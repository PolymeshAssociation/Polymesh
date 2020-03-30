//! # Protocol Fee Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReason},
};
use frame_system::{self as system, ensure_root};
use polymesh_runtime_common::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use primitives::{traits::IdentityCurrency, PosRatio, Signatory};
use sp_runtime::{
    traits::{Saturating, Zero},
    Perbill,
};

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
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
        /// Insufficient identity balance to pay the fee.
        InsufficientIdentityBalance,
        /// Insufficient account balance to pay the fee.
        InsufficientAccountBalance,
        /// Account ID decoding failed.
        AccountIdDecode,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ProtocolFee {
        /// The mapping of operation names to the base fees of those operations.
        pub BaseFees get(base_fees) config(): map ProtocolOp => BalanceOf<T>;
        /// The fee coefficient as a positive rational (numerator, denominator).
        pub Coefficient get(coefficient) config() build(|config: &GenesisConfig<T>| {
            if config.coefficient.1 == 0 {
                PosRatio(1, 1)
            } else {
                config.coefficient
            }
        }): PosRatio;
    }
}

decl_event! {
    pub enum Event<T> where Balance = BalanceOf<T> {
        /// The protocol fee of an operation.
        Fee(Balance),
        /// The fee coefficient.
        Coefficient(PosRatio),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Changes the fee coefficient for the root origin.
        pub fn change_coefficient(origin, coefficient: PosRatio) -> DispatchResult {
            ensure_root(origin)?;
            <Coefficient>::put(coefficient);
            Ok(())
        }

        /// Changes the a base fee for the root origin.
        pub fn change_base_fee(origin, op: ProtocolOp, base_fee: BalanceOf<T>) ->
            DispatchResult
        {
            ensure_root(origin)?;
            <BaseFees<T>>::insert(op, base_fee);
            Ok(())
        }

        /// Emits an event with the fee of the operation.
        pub fn get_fee(_origin, op: ProtocolOp) -> DispatchResult {
            let fee = Self::compute_fee(op);
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
    pub fn compute_fee(op: ProtocolOp) -> BalanceOf<T> {
        let coefficient = Self::coefficient();
        let ratio = Perbill::from_rational_approximation(coefficient.0, coefficient.1);
        ratio * Self::base_fees(op)
    }

    /// Computes the fee of the operation and charges it to the given signatory. The fee is then
    /// credited to the intended recipients according to the implementation of
    /// `OnProtocolFeePayment`.
    pub fn charge_fee(signatory: &Signatory, op: ProtocolOp) -> DispatchResult {
        let fee = Self::compute_fee(op);
        if fee.is_zero() {
            return Ok(());
        }
        let imbalance = Self::withdraw_fee(signatory, fee)?;
        // Pay the fee to the intended recipients depending on the implementation of
        // `OnProtocolFeePayment`.
        T::OnProtocolFeePayment::on_unbalanced(imbalance);
        Ok(())
    }

    /// Computes the fee for `count` similar operations, and charges that fee to the given
    /// signatory.
    pub fn charge_fee_batch(signatory: &Signatory, op: ProtocolOp, count: usize) -> DispatchResult {
        let fee = Self::compute_fee(op).saturating_mul(<BalanceOf<T>>::from(count as u32));
        let imbalance = Self::withdraw_fee(signatory, fee)?;
        T::OnProtocolFeePayment::on_unbalanced(imbalance);
        Ok(())
    }

    /// Withdraws a precomputed fee.
    fn withdraw_fee(signatory: &Signatory, fee: BalanceOf<T>) -> WithdrawFeeResult<T> {
        match signatory {
            Signatory::Identity(did) => T::Currency::withdraw_identity_balance(did, fee)
                .map_err(|_| Error::<T>::InsufficientIdentityBalance.into()),
            Signatory::AccountKey(account) => T::Currency::withdraw(
                &T::AccountId::decode(&mut &account.encode()[..])
                    .map_err(|_| Error::<T>::AccountIdDecode)?,
                fee,
                WithdrawReason::Fee.into(),
                ExistenceRequirement::KeepAlive,
            )
            .map_err(|_| Error::<T>::InsufficientAccountBalance.into()),
        }
    }
}

impl<T: Trait> ChargeProtocolFee<T::AccountId> for Module<T> {
    fn charge_fee(signatory: &Signatory, op: ProtocolOp) -> DispatchResult {
        Self::charge_fee(signatory, op)
    }

    fn charge_fee_batch(signatory: &Signatory, op: ProtocolOp, count: usize) -> DispatchResult {
        Self::charge_fee_batch(signatory, op, count)
    }
}
