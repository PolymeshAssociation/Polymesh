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
//! - [change_coefficient](Pallet::change_coefficient) - It changes the fee coefficient.
//! - [change_base_fee](Pallet::change_base_fee) - It changes the base fee.
//!
//! ### Public Functions
//!
//! - [compute_fee](Pallet::compute_fee) - It computes the fee of the operation.
//! - [charge_fee](Pallet::charge_fee) - It calculates the fee and charges it.
//! - [batch_charge_fee](Pallet::batch_charge_fee) - It calculates the fee and charges it on a batch operation.
//!
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::dispatch::DispatchResult;
use frame_support::pallet_prelude::DispatchError;
use frame_support::traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReasons};
use frame_support::weights::Weight;
use sp_runtime::{traits::Zero, Perbill};
use sp_std::vec::Vec;

use frame_system::ensure_root;
use pallet_identity::Config as IdentityConfig;
use polymesh_common_utilities::protocol_fee::{ChargeProtocolFee, ProtocolOp};
use polymesh_primitives::traits::{CddAndFeeDetails, SubsidiserTrait};
use polymesh_primitives::{Balance, IdentityId, PosRatio, GC_DID};

type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;
/// Either an imbalance or an error.
type WithdrawFeeResult<T> = sp_std::result::Result<NegativeImbalanceOf<T>, DispatchError>;

pub trait WeightInfo {
    fn change_coefficient() -> Weight;
    fn change_base_fee() -> Weight;
}

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;
    use frame_system::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config + IdentityConfig {
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;
        /// The currency type in which fees will be paid.
        type Currency: Currency<Self::AccountId, Balance = Balance> + Send + Sync;
        /// Handler for the unbalanced reduction when taking protocol fees.
        type OnProtocolFeePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;
        /// Weight calaculation.
        type WeightInfo: WeightInfo;
        /// Connection to the `Relayer` pallet.
        /// Used to charge protocol fees to a subsidiser, if any, instead of the payer.
        type Subsidiser: SubsidiserTrait<Self::AccountId, Self::RuntimeCall>;
    }

    #[pallet::error]
    pub enum Error<T> {
        /// Insufficient account balance to pay the fee.
        InsufficientAccountBalance,
        /// Not able to handled the imbalances
        UnHandledImbalances,
        /// Insufficient subsidy balance to pay the fee.
        InsufficientSubsidyBalance,
    }

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The mapping of operation names to the base fees of those operations.
    #[pallet::storage]
    pub type BaseFees<T: Config> = StorageMap<_, Twox64Concat, ProtocolOp, Balance, ValueQuery>;

    /// The fee coefficient as a positive rational (numerator, denominator).
    #[pallet::storage]
    pub type Coefficient<T: Config> = StorageValue<_, PosRatio, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        pub base_fees: Vec<(ProtocolOp, Balance)>,
        pub coefficient: PosRatio,
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            // Set initial base fees
            for (op, fee) in &self.base_fees {
                BaseFees::<T>::insert(op, fee);
            }

            // Set initial coefficient
            if self.coefficient.1 == 0 {
                Coefficient::<T>::put(&PosRatio(1, 1));
            } else {
                Coefficient::<T>::put(&self.coefficient);
            }
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// The protocol fee of an operation.
        FeeSet(IdentityId, Balance),
        /// The fee coefficient.
        CoefficientSet(IdentityId, PosRatio),
        /// Fee charged.
        FeeCharged(T::AccountId, Balance),
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        /// Changes the fee coefficient for the root origin.
        ///
        /// # Errors
        /// * `BadOrigin` - Only root allowed.
        #[pallet::weight(<T as Config>::WeightInfo::change_coefficient())]
        #[pallet::call_index(0)]
        pub fn change_coefficient(
            origin: OriginFor<T>,
            coefficient: PosRatio,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            Coefficient::<T>::put(&coefficient);
            Self::deposit_event(Event::<T>::CoefficientSet(GC_DID, coefficient));
            Ok(().into())
        }

        /// Changes the a base fee for the root origin.
        ///
        /// # Errors
        /// * `BadOrigin` - Only root allowed.
        #[pallet::weight(<T as Config>::WeightInfo::change_base_fee())]
        #[pallet::call_index(1)]
        pub fn change_base_fee(
            origin: OriginFor<T>,
            op: ProtocolOp,
            base_fee: Balance,
        ) -> DispatchResultWithPostInfo {
            ensure_root(origin)?;
            BaseFees::<T>::insert(op, &base_fee);
            Self::deposit_event(Event::<T>::FeeSet(GC_DID, base_fee));
            Ok(().into())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Computes the fee of the operation as `(base_fee * coefficient.0) / coefficient.1`.
    pub fn compute_fee(ops: &[ProtocolOp]) -> Balance {
        let coefficient = Coefficient::<T>::get();
        let ratio = Perbill::from_rational(coefficient.0, coefficient.1);
        let base = ops
            .iter()
            .fold(Zero::zero(), |a: Balance, e| a + BaseFees::<T>::get(e));
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

        Self::deposit_event(Event::<T>::FeeCharged(fee_key, fee));
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

impl<T: Config> ChargeProtocolFee<T::AccountId> for Pallet<T> {
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
