// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

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

#![cfg_attr(not(feature = "std"), no_std)]

use pallet_identity as identity;

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReason},
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_root};
use polymesh_common_utilities::{
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    Context, SystematicIssuers,
};
use primitives::{traits::IdentityCurrency, IdentityId, PosRatio, Signatory};
use sp_runtime::{
    traits::{Saturating, Zero},
    PerThing, Perbill,
};

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;
/// Either an imbalance or an error.
type WithdrawFeeResult<T> = sp_std::result::Result<NegativeImbalanceOf<T>, DispatchError>;
type Identity<T> = identity::Module<T>;

pub trait Trait: frame_system::Trait + IdentityTrait {
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
        /// Missing current DID
        MissingCurrentIdentity,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as ProtocolFee {
        /// The mapping of operation names to the base fees of those operations.
        pub BaseFees get(base_fees) config(): map hasher(twox_64_concat) ProtocolOp => BalanceOf<T>;
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
        FeeSet(IdentityId, Balance),
        /// The fee coefficient.
        CoefficientSet(IdentityId, PosRatio),
        /// Fee charged.
        FeeCharged(IdentityId, Balance),
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Changes the fee coefficient for the root origin.
        #[weight = SimpleDispatchInfo::FixedOperational(500_000)]
        pub fn change_coefficient(origin, coefficient: PosRatio) -> DispatchResult {
            ensure_root(origin)?;
            let id = Context::current_identity::<Identity<T>>().unwrap_or(SystematicIssuers::Committee.as_id());

            <Coefficient>::put(&coefficient);
            Self::deposit_event(RawEvent::CoefficientSet(id, coefficient));
            Ok(())
        }

        /// Changes the a base fee for the root origin.
        #[weight = SimpleDispatchInfo::FixedOperational(500_000)]
        pub fn change_base_fee(origin, op: ProtocolOp, base_fee: BalanceOf<T>) ->
            DispatchResult
        {
            ensure_root(origin)?;
            let id = Context::current_identity::<Identity<T>>().unwrap_or(SystematicIssuers::Committee.as_id());

            <BaseFees<T>>::insert(op, &base_fee);
            Self::deposit_event(RawEvent::FeeSet(id, base_fee));
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
        let id = Context::current_identity::<Identity<T>>()
            .ok_or_else(|| Error::<T>::MissingCurrentIdentity)?;
        Self::deposit_event(RawEvent::FeeCharged(id, fee));
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
