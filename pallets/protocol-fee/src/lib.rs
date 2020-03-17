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
        weights::{DispatchClass, DispatchInfo, GetDispatchInfo, Weight},
    };
    use frame_system as system;
    use sp_core::H256;
    use sp_runtime::{
        testing::{Header, TestXt},
        traits::{BlakeTwo256, Extrinsic, IdentityLookup},
        Perbill,
    };

    type ProtocolFee = super::Module<Runtime>;
    type System = frame_system::Module<Runtime>;

    impl_outer_dispatch! {
        pub enum Call for Runtime where origin: Origin {
            frame_system::System,
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Runtime;

    impl_outer_origin! {
        pub enum Origin for Runtime {}
    }

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    impl frame_system::Trait for Runtime {
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

    impl Trait for Runtime {
        type Event = ();
        type Currency = polymesh_runtime_balances::Module<Runtime>;
        type OnProtocolFeePayment = ();
    }

    pub struct ExtBuilder {
        base_fees: Vec<(Vec<u8>, u64)>,
        multiplier: (u32, u32),
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                base_fees: vec![
                    (b"10_k_test".to_vec(), 10_000),
                    (b"99_k_test".to_vec(), 99_000),
                ],
                multiplier: (1, 1),
            }
        }
    }

    impl ExtBuilder {
        fn build(self) -> sp_io::TestExternalities {
            let storage = frame_system::GenesisConfig::default()
                .build_storage::<Runtime>()
                .unwrap();
            storage.into()
        }
    }

    #[test]
    fn can_compute_fee() {
        ExtBuilder::default().build().execute_with(|| {
            assert_eq!(
                ProtocolFee::compute_fee(ExtrinsicName::from(b"10_k_test")),
                10_000
            );
            assert_eq!(
                ProtocolFee::compute_fee(ExtrinsicName::from(b"99_k_test")),
                99_000
            );
        });
    }
}
