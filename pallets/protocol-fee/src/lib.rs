// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Protocol Fee Module
//!
//! This module stores the fee of each protocol operation, and a common coefficient which is applied on
//! fee computation.
//!
//! It also provides helper functions to calculate and charge fees on each protocol operation.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - [change_coefficient](Module::change_coefficient) - It changes the fee coefficient.
//! - [change_base_fee](Module::change_base_fee) - It changes the base fee.
//!
//! ### Public Functions
//!
//! - [compute_fee](Module::compute_fee) - It computes the fee of the operation.
//! - [charge_fee](Module::charge_fee) - It calculates the fee and charges it.
//! - [batch_charge_fee](Module::batch_charge_fee) - It calculates the fee and charges it on a batch operation.
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReasons},
    weights::Weight,
};
use frame_system::ensure_root;
use polymesh_common_utilities::{
    identity::Config as IdentityConfig,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::relayer::SubsidiserTrait,
    transaction_payment::CddAndFeeDetails,
    GC_DID,
};
use polymesh_primitives::{Balance, IdentityId, PosRatio};
use sp_runtime::{traits::Zero, Perbill};
use sp_std::vec;

type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;
/// Either an imbalance or an error.
type WithdrawFeeResult<T> = sp_std::result::Result<NegativeImbalanceOf<T>, DispatchError>;

pub trait WeightInfo {
    fn change_coefficient() -> Weight;
    fn change_base_fee() -> Weight;
}

pub trait Config: frame_system::Config + IdentityConfig {
    type RuntimeEvent: From<Event<Self>> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId, Balance = Balance> + Send + Sync;
    /// Handler for the unbalanced reduction when taking protocol fees.
    type OnProtocolFeePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;
    /// Weight calaculation.
    type WeightInfo: WeightInfo;
    /// Connection to the `Relayer` pallet.
    /// Used to charge protocol fees to a subsidiser, if any, instead of the payer.
    type Subsidiser: SubsidiserTrait<Self::AccountId>;
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// Insufficient account balance to pay the fee.
        InsufficientAccountBalance,
        /// Not able to handled the imbalances
        UnHandledImbalances,
        /// Insufficient subsidy balance to pay the fee.
        InsufficientSubsidyBalance,
    }
}

decl_storage! {
    trait Store for Module<T: Config> as ProtocolFee {
        /// The mapping of operation names to the base fees of those operations.
        pub BaseFees get(fn base_fees) config(): map hasher(twox_64_concat) ProtocolOp => Balance;
        /// The fee coefficient as a positive rational (numerator, denominator).
        pub Coefficient get(fn coefficient) config() build(|config: &GenesisConfig| {
            if config.coefficient.1 == 0 {
                PosRatio(1, 1)
            } else {
                config.coefficient
            }
        }): PosRatio;
    }
}

decl_event! {
    pub enum Event<T> where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// The protocol fee of an operation.
        FeeSet(IdentityId, Balance),
        /// The fee coefficient.
        CoefficientSet(IdentityId, PosRatio),
        /// Fee charged.
        FeeCharged(AccountId, Balance),
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Changes the fee coefficient for the root origin.
        ///
        /// # Errors
        /// * `BadOrigin` - Only root allowed.
        #[weight = <T as Config>::WeightInfo::change_coefficient()]
        pub fn change_coefficient(origin, coefficient: PosRatio) {
            ensure_root(origin)?;
            Coefficient::put(&coefficient);
            Self::deposit_event(Event::<T>::CoefficientSet(GC_DID, coefficient));
        }

        /// Changes the a base fee for the root origin.
        ///
        /// # Errors
        /// * `BadOrigin` - Only root allowed.
        #[weight = <T as Config>::WeightInfo::change_base_fee()]
        pub fn change_base_fee(origin, op: ProtocolOp, base_fee: Balance) {
            ensure_root(origin)?;
            BaseFees::insert(op, &base_fee);
            Self::deposit_event(Event::<T>::FeeSet(GC_DID, base_fee));
        }
    }
}

impl<T: Config> Module<T> {
    /// Computes the fee of the operation as `(base_fee * coefficient.0) / coefficient.1`.
    pub fn compute_fee(ops: &[ProtocolOp]) -> Balance {
        let coefficient = Self::coefficient();
        let ratio = Perbill::from_rational(coefficient.0, coefficient.1);
        let base = ops
            .iter()
            .fold(Zero::zero(), |a: Balance, e| a + Self::base_fees(e));
        ratio * base
    }

    /// Computes the fee of the operations and charges it to the current payer. The fee is then
    /// credited to the intended recipients according to the implementation of
    /// `OnProtocolFeePayment`.
    pub fn charge_fees(ops: &[ProtocolOp]) -> DispatchResult {
        if ops.is_empty() {
            return Ok(());
        }
        let fee = Self::compute_fee(ops);
        if fee.is_zero() {
            return Ok(());
        }
        Self::withdraw_from_payer(fee)
    }

    /// Computes the fee for `count` similar operations, and charges that fee to the current payer.
    pub fn batch_charge_fee(op: ProtocolOp, count: usize) -> DispatchResult {
        let fee = Self::compute_fee(&[op]).saturating_mul(Balance::from(count as u32));
        if fee.is_zero() {
            return Ok(());
        }
        Self::withdraw_from_payer(fee)
    }

    /// Withdraws a precomputed fee from the current payer if it is defined or from the current
    /// identity otherwise.
    fn withdraw_fee(account: T::AccountId, fee: Balance) -> WithdrawFeeResult<T> {
        // Check if the `account` is being subsidised.
        let subsidiser = T::Subsidiser::check_subsidy(&account, fee, None)
            .map_err(|_| Error::<T>::InsufficientSubsidyBalance)?;

        // `fee_key` is either a subsidiser or the original payer.
        let fee_key = if let Some(subsidiser_key) = subsidiser {
            // Debit the protocol `fee` from the subsidy if there was a subsidiser.
            // This shouldn't fail, since the subsidy was already checked.
            T::Subsidiser::debit_subsidy(&account, fee)
                .map_err(|_| Error::<T>::InsufficientSubsidyBalance)?;
            subsidiser_key
        } else {
            // No subsidy.
            account
        };

        // Withdraw protocol `fee` from the payer `fee_key.
        let ret = T::Currency::withdraw(
            &fee_key,
            fee,
            WithdrawReasons::FEE,
            ExistenceRequirement::KeepAlive,
        )
        .map_err(|_| Error::<T>::InsufficientAccountBalance)?;

        Self::deposit_event(RawEvent::FeeCharged(fee_key, fee));
        Ok(ret)
    }

    fn withdraw_from_payer(fee: Balance) -> DispatchResult {
        if let Some(payer) = T::CddHandler::get_payer_from_context() {
            let imbalance = Self::withdraw_fee(payer, fee)?;
            T::OnProtocolFeePayment::on_unbalanced(imbalance);
        }
        Ok(())
    }
}

impl<T: Config> ChargeProtocolFee<T::AccountId> for Module<T> {
    fn charge_fee(op: ProtocolOp) -> DispatchResult {
        Self::charge_fees(&[op])
    }

    fn charge_fees(ops: &[ProtocolOp]) -> DispatchResult {
        Self::charge_fees(ops)
    }

    fn batch_charge_fee(op: ProtocolOp, count: usize) -> DispatchResult {
        Self::batch_charge_fee(op, count)
    }
}
