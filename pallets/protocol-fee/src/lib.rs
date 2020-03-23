//! # Protocol Fee Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced, WithdrawReason},
};
use frame_system::{self as system, ensure_root};
use polymesh_runtime_common::protocol_fee::{ChargeProtocolFee, OperationName};
use primitives::{traits::IdentityCurrency, Signatory};
use sp_runtime::traits::{CheckedDiv, Saturating};
//use sp_std::prelude::*;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
/// Either the computed fee or an error.
pub type ComputeFeeResult<T> = sp_std::result::Result<BalanceOf<T>, DispatchError>;
/// A positive rational number: a pair of a numerator and a denominator.
pub type PosRational = (u32, u32);
/// Either an imbalance or an error.
type WithdrawFeeResult<T> = sp_std::result::Result<NegativeImbalanceOf<T>, DispatchError>;

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
    trait Store for Module<T: Trait> as ProtocolFee {
        /// The mapping of operation names to the base fees of those operations.
        pub BaseFees get(base_fees) config(): map OperationName => BalanceOf<T>;
        /// The fee coefficient as a positive rational (numerator, denominator).
        pub Coefficient get(coefficient) config() build(|config: &GenesisConfig<T>| {
            if config.coefficient.1 == 0 {
                (1, 1)
            } else {
                config.coefficient
            }
        }): PosRational;
    }
}

decl_event! {
    pub enum Event<T> where Balance = BalanceOf<T> {
        /// The protocol fee of an operation.
        Fee(Balance),
        /// The fee coefficient.
        Coefficient(PosRational),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Changes the fee coefficient for the root origin.
        pub fn change_coefficient(origin, coefficient: (u32, u32)) -> DispatchResult {
            ensure_root(origin)?;
            <Coefficient>::put(coefficient);
            Ok(())
        }

        /// Changes the a base fee for the root origin.
        pub fn change_base_fee(origin, name: OperationName, base_fee: BalanceOf<T>) ->
            DispatchResult
        {
            ensure_root(origin)?;
            <BaseFees<T>>::insert(name, base_fee);
            Ok(())
        }

        /// Emits an event with the fee of the operation.
        pub fn get_fee(_origin, name: OperationName) -> DispatchResult {
            let fee = Self::compute_fee(&name)?;
            Self::deposit_event(RawEvent::Fee(fee));
            Ok(())
        }

        /// Emits an event with the fee coefficient.
        pub fn get_coefficient(_origin) -> DispatchResult {
            Self::deposit_event(RawEvent::Coefficient(Self::coefficient()));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Computes the fee of the operation as `(base_fee * coefficient.0) / coefficient.1`.
    pub fn compute_fee(name: &OperationName) -> ComputeFeeResult<T> {
        let (numerator, denominator) = Self::coefficient();
        if let Some(fee) = Self::base_fees(name)
            .saturating_mul(<BalanceOf<T>>::from(numerator))
            .checked_div(&<BalanceOf<T>>::from(denominator))
        {
            Ok(fee)
        } else {
            Err(Error::<T>::ComputeFee.into())
        }
    }

    /// Computes the fee of the operation and charges it to the given signatory.
    pub fn charge_fee(signatory: &Signatory, name: &OperationName) -> DispatchResult {
        let fee = Self::compute_fee(name)?;
        let imbalance = Self::withdraw_fee(signatory, fee)?;
        T::OnProtocolFeePayment::on_unbalanced(imbalance);
        Ok(())
    }

    /// Computes the fee for `count` similar operations, and charges that fee to the given
    /// signatory.
    pub fn charge_fee_batch(
        signatory: &Signatory,
        name: &OperationName,
        count: usize,
    ) -> DispatchResult {
        let fee = Self::compute_fee(name)?.saturating_mul(<BalanceOf<T>>::from(count as u32));
        let imbalance = Self::withdraw_fee(signatory, fee)?;
        T::OnProtocolFeePayment::on_unbalanced(imbalance);
        Ok(())
    }

    /// Computes the fee of the operation, charges that fee to `signatory`, and pays it out
    /// collectively to `recipients` in equal parts.
    pub fn charge_fee_equal_parts(
        signatory: &Signatory,
        name: &OperationName,
        recipients: &[<T as frame_system::Trait>::AccountId],
    ) -> DispatchResult {
        let fee = Self::compute_fee(name)?;
        let mut imbalance = Self::withdraw_fee(signatory, fee)?;
        let num_recipients = u32::max(1, recipients.len() as u32);
        let fee_part = imbalance.peek() / num_recipients.into();
        for account_id in recipients {
            let (part, rest) = imbalance.split(fee_part);
            imbalance = rest;
            T::Currency::resolve_creating(account_id, part);
        }
        // Burn the remainder of division by not resolving it.
        Ok(())
    }

    /// Withdraws a precomputed fee.
    fn withdraw_fee(signatory: &Signatory, fee: BalanceOf<T>) -> WithdrawFeeResult<T> {
        match signatory {
            Signatory::Identity(did) => T::Currency::withdraw_identity_balance(did, fee)
                .map_err(|_| Error::<T>::InsufficientBalance.into()),
            Signatory::AccountKey(account) => T::Currency::withdraw(
                &T::AccountId::decode(&mut &account.encode()[..])
                    .map_err(|_| Error::<T>::AccountIdDecode)?,
                fee,
                WithdrawReason::Fee.into(),
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientBalance.into()),
        }
    }
}

impl<T: Trait> ChargeProtocolFee<T::AccountId> for Module<T> {
    fn charge_fee(signatory: &Signatory, name: &OperationName) -> DispatchResult {
        Self::charge_fee(signatory, name)
    }

    fn charge_fee_batch(
        signatory: &Signatory,
        name: &OperationName,
        count: usize,
    ) -> DispatchResult {
        Self::charge_fee_batch(signatory, name, count)
    }

    fn charge_fee_equal_parts(
        signatory: &Signatory,
        name: &OperationName,
        recipients: &[T::AccountId],
    ) -> DispatchResult {
        Self::charge_fee_equal_parts(signatory, name, recipients)
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
    use sp_core::{crypto::key_types, H256};
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        transaction_validity::{TransactionValidity, ValidTransaction},
        AnySignature, KeyTypeId, Perbill,
    };
    //    use test_client::{self, AccountKeyring};

    type AccountId = <AnySignature as Verify>::Signer;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type Balances = balances::Module<Test>;
    type BlockNumber = u64;
    type Identity = identity::Module<Test>;
    type OffChainSignature = AnySignature;
    type ProtocolFee = super::Module<Test>;
    type SessionIndex = u32;
    type System = frame_system::Module<Test>;

    pub struct TestOnSessionEnding;
    impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
        const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}

        fn on_before_session_ending() {}
    }

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    impl_outer_dispatch! {
        pub enum Call for Test where origin: Origin {
            frame_system::System,
            identity::Identity,
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Test;

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
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Call = Call;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
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

    impl pallet_session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call;
        type AddSignerMultiSigTarget = Test;
        type CddServiceProviders = Test;
        type Balances = balances::Module<Test>;
        type ChargeTxFeeTarget = Test;
        type Public = AccountId;
        type OffChainSignature = OffChainSignature;
        type ProtocolFee = super::Module<Test>;
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
        base_fees: Vec<(OperationName, u128)>,
        coefficient: PosRational,
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                base_fees: vec![
                    (OperationName::from(b"10_k_test"), 10_000),
                    (OperationName::from(b"99_k_test"), 99_000),
                ],
                coefficient: (1, 1),
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
                coefficient: self.coefficient,
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
                ProtocolFee::compute_fee(&OperationName::from(b"10_k_test")),
                Ok(10_000)
            );
            assert_eq!(
                ProtocolFee::compute_fee(&OperationName::from(b"99_k_test")),
                Ok(99_000)
            );
        });
    }
}
