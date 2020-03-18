//! # Protocol Fee Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReason},
};
use frame_system::{self as system, ensure_root};
use primitives::{traits::IdentityCurrency, Signatory};
use sp_runtime::traits::{CheckedDiv, Saturating};
use sp_std::{fmt::Debug, prelude::*};

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
/// Either the computed fee or an error.
pub type ComputeFeeResult<T> = sp_std::result::Result<BalanceOf<T>, DispatchError>;

/// A wrapper for a dispatchable function name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtrinsicName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for ExtrinsicName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        ExtrinsicName(v)
    }
}

pub trait Trait: frame_system::Trait {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId> + Send + Sync + IdentityCurrency<Self::AccountId>;
    /// Handler for the unbalanced reduction when taking protocol fees.
    type OnProtocolFeePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Insufficient balance to pay the fee.
        InsufficientBalance,
        /// Account ID decoding failed.
        AccountIdDecode,
        /// Division in `compute_fee` failed.
        ComputeFee,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Balances {
        /// The mapping of extrinsic names to the base fees of those extrinsics.
        BaseFees get(base_fees) config(): map ExtrinsicName => BalanceOf<T>;
        /// The fee multiplier as a positive rational (numerator, denominator).
        Multiplier get(multiplier) config() build(|config: &GenesisConfig<T>| {
            if config.multiplier.1 == 0 {
                (1, 1)
            } else {
                config.multiplier
            }
        }): (u32, u32);
    }
}

decl_event! {
    pub enum Event<T> where Balance = BalanceOf<T> {
        /// The protocol fee of an extrinsic.
        Fee(Balance),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Changes the fee multiplier for the root origin.
        pub fn change_multiplier(origin, multiplier: (u32, u32)) -> DispatchResult {
            ensure_root(origin)?;
            <Multiplier>::put(multiplier);
            Ok(())
        }

        /// Changes the a base fee for the root origin.
        pub fn change_base_fee(origin, name: ExtrinsicName, base_fee: BalanceOf<T>) ->
            DispatchResult
        {
            ensure_root(origin)?;
            <BaseFees<T>>::insert(name, base_fee);
            Ok(())
        }

        /// Emits an event with the fee of the extrinsic.
        pub fn get_fee(_origin, name: ExtrinsicName) -> DispatchResult {
            let fee = Self::compute_fee(name)?;
            Self::deposit_event(RawEvent::Fee(fee));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Computes the fee of the extrinsic.
    pub fn compute_fee(name: ExtrinsicName) -> ComputeFeeResult<T> {
        let (numerator, denominator) = Self::multiplier();
        if let Some(fee) = Self::base_fees(name)
            .saturating_mul(<BalanceOf<T>>::from(numerator))
            .checked_div(&<BalanceOf<T>>::from(denominator))
        {
            Ok(fee)
        } else {
            Err(Error::<T>::ComputeFee.into())
        }
    }

    /// Computes the fee of the extrinsic and charges it to the given signatory.
    pub fn charge_fee(signatory: Signatory, name: ExtrinsicName) -> DispatchResult {
        let fee = Self::compute_fee(name)?;
        let imbalance = match signatory {
            Signatory::Identity(did) => T::Currency::withdraw_identity_balance(&did, fee)
                .map_err(|_| Error::<T>::InsufficientBalance),
            Signatory::AccountKey(account) => T::Currency::withdraw(
                &T::AccountId::decode(&mut &account.encode()[..])
                    .map_err(|_| Error::<T>::AccountIdDecode)?,
                fee,
                WithdrawReason::Fee.into(),
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientBalance),
        }?;
        T::OnProtocolFeePayment::on_unbalanced(imbalance);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use frame_support::{
        impl_outer_dispatch, impl_outer_origin, parameter_types,
        weights::{DispatchInfo, Weight},
    };
    use frame_system as system;
    use polymesh_runtime_balances as balances;
    use polymesh_runtime_common::traits::{
        asset::AcceptTransfer, group::GroupTrait, multisig::AddSignerMultiSig, CommonTrait,
    };
    use polymesh_runtime_identity as identity;
    use primitives::IdentityId;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
        transaction_validity::{TransactionValidity, ValidTransaction},
        Perbill,
    };
    use test_client::{self, AccountKeyring};

    type Balances = balances::Module<Test>;
    type ProtocolFee = super::Module<Test>;
    type System = frame_system::Module<Test>;

    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {
            frame_system::System,
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Test;

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl AcceptTransfer for Test {
        fn accept_ticker_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }

        fn accept_token_ownership_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }
    }

    impl GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!();
        }
    }

    impl AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const ExistentialDeposit: u64 = 0;
        pub const MinimumPeriod: u64 = 3;
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = Call;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
    }

    impl pallet_transaction_payment::ChargeTxFee for Test {
        fn charge_fee(_who: Signatory, _len: u32, _info: DispatchInfo) -> TransactionValidity {
            Ok(ValidTransaction::default())
        }
    }

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call;
        type AddSignerMultiSigTarget = Test;
        type CddServiceProviders = Test;
        type Balances = balances::Module<Test>;
        type ChargeTxFeeTarget = Test;
    }

    impl CommonTrait for Test {
        type Balance = u128;
        type CreationFee = CreationFee;
        type AcceptTransferTarget = Test;
        type BlockRewardsReserve = balances::Module<Test>;
    }

    impl balances::Trait for Test {
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type Identity = identity::Module<Test>;
    }

    impl Trait for Test {
        type Event = ();
        type Currency = Balances;
        type OnProtocolFeePayment = ();
    }

    pub struct ExtBuilder {
        base_fees: Vec<(ExtrinsicName, u128)>,
        multiplier: (u32, u32),
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                base_fees: vec![
                    (ExtrinsicName::from(b"10_k_test"), 10_000),
                    (ExtrinsicName::from(b"99_k_test"), 99_000),
                ],
                multiplier: (1, 1),
            }
        }
    }

    impl ExtBuilder {
        fn build(self) -> sp_io::TestExternalities {
            let mut storage = frame_system::GenesisConfig::default()
                .build_storage::<Test>()
                .unwrap();
            GenesisConfig::<Test> {
                base_fees: self.base_fees,
                multiplier: self.multiplier,
            }
            .assimilate_storage(&mut storage)
            .unwrap();
            storage.into()
        }
    }

    #[test]
    fn can_compute_fee() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(
                ProtocolFee::compute_fee(ExtrinsicName::from(b"10_k_test")),
                Ok(10_000)
            );
            assert_eq!(
                ProtocolFee::compute_fee(ExtrinsicName::from(b"99_k_test")),
                Ok(99_000)
            );
        });
    }
}
