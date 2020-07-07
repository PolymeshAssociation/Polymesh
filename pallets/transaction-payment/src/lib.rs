// Copyright 2019 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

// Modified by Polymath Inc - 13rd March 2020
// - Charge fee from the identity in the signed extension
// - Introduce `ChargeTxFee` trait to compute and charge transaction fee for Multisig.

//! # Transaction Payment Module
//!
//! This module provides the basic logic needed to pay the absolute minimum amount needed for a
//! transaction to be included. This includes:
//!   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
//!   - _length fee_: A fee proportional to the encoded length of the transaction.
//!   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
//!     chance to be included by the transaction queue.
//!
//! Additionally, this module allows one to configure:
//!   - The mapping between one unit of weight to one unit of fee via [`WeightToFee`].
//!   - A means of updating the fee for the next block, via defining a multiplier, based on the
//!     final state of the chain at the end of the previous block. This can be configured via
//!     [`FeeMultiplierUpdate`]

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_module, decl_storage,
    dispatch::DispatchResult,
    traits::{Currency, ExistenceRequirement, Get, Imbalance, OnUnbalanced, WithdrawReason},
    weights::{
        DispatchInfo, GetDispatchInfo, Pays, PostDispatchInfo, Weight, WeightToFeeCoefficient,
        WeightToFeePolynomial,
    },
};
use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
use primitives::{traits::IdentityCurrency, IdentityId, Signatory, TransactionError};
use sp_runtime::{
    traits::{
        Convert, DispatchInfoOf, Dispatchable, PostDispatchInfoOf, SaturatedConversion, Saturating,
        SignedExtension, UniqueSaturatedFrom, UniqueSaturatedInto, Zero,
    },
    transaction_validity::{
        InvalidTransaction, TransactionPriority, TransactionValidity, TransactionValidityError,
        ValidTransaction,
    },
    FixedI128, FixedPointNumber, FixedPointOperand, Perquintill,
};
use sp_std::prelude::*;

/// Fee multiplier.
pub type Multiplier = FixedI128;

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// A struct to update the weight multiplier per block. It implements `Convert<Multiplier,
/// Multiplier>`, meaning that it can convert the previous multiplier to the next one. This should
/// be called on `on_finalize` of a block, prior to potentially cleaning the weight data from the
/// system module.
///
/// given:
/// 	s = previous block weight
/// 	s'= ideal block weight
/// 	m = maximum block weight
///		diff = (s - s')/m
///		v = 0.00001
///		t1 = (v * diff)
///		t2 = (v * diff)^2 / 2
///	then:
/// 	next_multiplier = prev_multiplier * (1 + t1 + t2)
///
/// Where `(s', v)` must be given as the `Get` implementation of the `T` generic type. Moreover, `M`
/// must provide the minimum allowed value for the multiplier. Note that a runtime should ensure
/// with tests that the combination of this `M` and `V` is not such that the multiplier can drop to
/// zero and never recover.
///
/// note that `s'` is interpreted as a portion in the _normal transaction_ capacity of the block.
/// For example, given `s' == 0.25` and `AvailableBlockRatio = 0.75`, then the target fullness is
/// _0.25 of the normal capacity_ and _0.1875 of the entire block_.
///
/// This implementation implies the bound:
/// - `v ≤ p / k * (s − s')`
/// - or, solving for `p`: `p >= v * k * (s - s')`
///
/// where `p` is the amount of change over `k` blocks.
///
/// Hence:
/// - in a fully congested chain: `p >= v * k * (1 - s')`.
/// - in an empty chain: `p >= v * k * (-s')`.
///
/// For example, when all blocks are full and there are 28800 blocks per day (default in `substrate-node`)
/// and v == 0.00001, s' == 0.1875, we'd have:
///
/// p >= 0.00001 * 28800 * 0.8125
/// p >= 0.234
///
/// Meaning that fees can change by around ~23% per day, given extreme congestion.
///
/// More info can be found at:
/// https://w3f-research.readthedocs.io/en/latest/polkadot/Token%20Economics.html
pub struct TargetedFeeAdjustment<T, S, V, M>(sp_std::marker::PhantomData<(T, S, V, M)>);

impl<T, S, V, M> Convert<Multiplier, Multiplier> for TargetedFeeAdjustment<T, S, V, M>
where
    T: frame_system::Trait,
    S: Get<Perquintill>,
    V: Get<Multiplier>,
    M: Get<Multiplier>,
{
    fn convert(previous: Multiplier) -> Multiplier {
        // Defensive only. The multiplier in storage should always be at most positive. Nonetheless
        // we recover here in case of errors, because any value below this would be stale and can
        // never change.
        let min_multiplier = M::get();
        let previous = previous.max(min_multiplier);

        // the computed ratio is only among the normal class.
        let normal_max_weight = <T as frame_system::Trait>::AvailableBlockRatio::get()
            * <T as frame_system::Trait>::MaximumBlockWeight::get();
        let normal_block_weight = <frame_system::Module<T>>::block_weight()
            .get(frame_support::weights::DispatchClass::Normal)
            .min(normal_max_weight);

        let s = S::get();
        let v = V::get();

        let target_weight = (s * normal_max_weight) as u128;
        let block_weight = normal_block_weight as u128;

        // determines if the first_term is positive
        let positive = block_weight >= target_weight;
        let diff_abs = block_weight.max(target_weight) - block_weight.min(target_weight);

        // defensive only, a test case assures that the maximum weight diff can fit in Multiplier
        // without any saturation.
        let diff = Multiplier::saturating_from_rational(diff_abs, normal_max_weight.max(1));
        let diff_squared = diff.saturating_mul(diff);

        let v_squared_2 = v.saturating_mul(v) / Multiplier::saturating_from_integer(2);

        let first_term = v.saturating_mul(diff);
        let second_term = v_squared_2.saturating_mul(diff_squared);

        if positive {
            let excess = first_term
                .saturating_add(second_term)
                .saturating_mul(previous);
            previous.saturating_add(excess).max(min_multiplier)
        } else {
            // Defensive-only: first_term > second_term. Safe subtraction.
            let negative = first_term
                .saturating_sub(second_term)
                .saturating_mul(previous);
            previous.saturating_sub(negative).max(min_multiplier)
        }
    }
}

pub trait Trait: frame_system::Trait {
    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId> + Send + Sync + IdentityCurrency<Self::AccountId>;

    /// Handler for the unbalanced reduction when taking transaction fees.
    type OnTransactionPayment: OnUnbalanced<NegativeImbalanceOf<Self>>;

    /// The fee to be paid for making a transaction; the per-byte portion.
    type TransactionByteFee: Get<BalanceOf<Self>>;

    /// Convert a weight value into a deductible fee based on the currency type.
    type WeightToFee: WeightToFeePolynomial<Balance = BalanceOf<Self>>;

    /// Update the multiplier of the next block, based on the previous block's weight.
    type FeeMultiplierUpdate: Convert<Multiplier, Multiplier>;

    // Polymesh note: This was specifically added for Polymesh
    /// Fetch the signatory to charge fee from. Also sets fee payer and identity in context.
    type CddHandler: CddAndFeeDetails<Self::AccountId, Self::Call>;
}

decl_storage! {
    trait Store for Module<T: Trait> as TransactionPayment {
        pub NextFeeMultiplier get(fn next_fee_multiplier): Multiplier = Multiplier::saturating_from_integer(1);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// The fee to be paid for making a transaction; the per-byte portion.
        const TransactionByteFee: BalanceOf<T> = T::TransactionByteFee::get();

        /// The polynomial that is applied in order to derive fee from weight.
        const WeightToFee: Vec<WeightToFeeCoefficient<BalanceOf<T>>> =
            T::WeightToFee::polynomial().to_vec();

        fn on_finalize() {
            NextFeeMultiplier::mutate(|fm| {
                *fm = T::FeeMultiplierUpdate::convert(*fm);
            });
        }

        fn integrity_test() {
            // given weight == u64, we build multipliers from `diff` of two weight values, which can
            // at most be MaximumBlockWeight. Make sure that this can fit in a multiplier without
            // loss.
            use sp_std::convert::TryInto;
            assert!(
                <Multiplier as sp_runtime::traits::Bounded>::max_value() >=
                Multiplier::checked_from_integer(
                    <T as frame_system::Trait>::MaximumBlockWeight::get().try_into().unwrap()
                ).unwrap(),
            );
        }
    }
}

impl<T: Trait> Module<T>
where
    BalanceOf<T>: FixedPointOperand,
{
    /// Query the data that we know about the fee of a given `call`.
    ///
    /// As this module is not and cannot be aware of the internals of a signed extension, It only
    /// interprets the extrinsic as some encoded value and accounts for its weight
    /// and length, the runtime's extrinsic base weight, and the current fee multiplier.
    ///
    /// All dispatchables must be annotated with weight and will have some fee info. This function
    /// always returns.
    pub fn query_info<Extrinsic: GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> RuntimeDispatchInfo<BalanceOf<T>>
    where
        T: Send + Sync,
        BalanceOf<T>: Send + Sync,
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        // NOTE: we can actually make it understand `ChargeTransactionPayment`, but would be some hassle
        // for sure. We have to make it aware of the index of `ChargeTransactionPayment` in `Extra`.
        // Alternatively, we could actually execute the tx's per-dispatch and record the balance of the
        // sender before and after the pipeline.. but this is way too much hassle for a very very little
        // potential gain in the future.
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);

        let partial_fee = Self::compute_fee(len, &dispatch_info, 0u32.into());
        let DispatchInfo { weight, class, .. } = dispatch_info;

        RuntimeDispatchInfo {
            weight,
            class,
            partial_fee,
        }
    }

    /// Compute the final fee value for a particular transaction.
    ///
    /// The final fee is composed of:
    ///   - `base_weight`: This is the minimum amount a user pays for a transaction. It is declared
    ///     as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
    ///   - `len_fee`: The length fee, the amount paid for the encoded length (in bytes) of the
    ///     transaction.
    ///   - `weight_fee`: This amount is computed based on the weight of the transaction. Weight
    ///     accounts for the execution time of a transaction.
    ///   - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on
    ///     the congestion of the network.
    ///   - (Optional) `tip`: If included in the transaction, the tip will be added on top. Only
    ///     signed transactions can have a tip. Although it will always be zero in Polymesh, keeping
    ///     the tip as parameter to reduce the change in the apis.
    ///
    /// The base fee and adjusted weight and length fees constitute the _inclusion fee,_ which is
    /// the minimum fee for a transaction to be included in a block.
    ///
    /// ```ignore
    /// inclusion_fee = base_weight + targeted_fee_adjustment * (len_fee + weight_fee);
    /// final_fee = inclusion_fee + tip;
    /// ```
    pub fn compute_fee(len: u32, info: &DispatchInfoOf<T::Call>, tip: BalanceOf<T>) -> BalanceOf<T>
    where
        T::Call: Dispatchable<Info = DispatchInfo>,
    {
        Self::compute_fee_raw(len, info.weight, tip, info.pays_fee)
    }

    /// Compute the actual post dispatch fee for a particular transaction.
    ///
    /// Identical to `compute_fee` with the only difference that the post dispatch corrected
    /// weight is used for the weight fee calculation.
    pub fn compute_actual_fee(
        len: u32,
        info: &DispatchInfoOf<T::Call>,
        post_info: &PostDispatchInfoOf<T::Call>,
        tip: BalanceOf<T>,
    ) -> BalanceOf<T>
    where
        T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    {
        Self::compute_fee_raw(len, post_info.calc_actual_weight(info), tip, info.pays_fee)
    }

    fn compute_fee_raw(
        len: u32,
        weight: Weight,
        tip: BalanceOf<T>,
        pays_fee: Pays,
    ) -> BalanceOf<T> {
        if pays_fee == Pays::Yes {
            let len = <BalanceOf<T>>::from(len);
            let per_byte = T::TransactionByteFee::get();

            // length fee. this is not adjusted.
            let fixed_len_fee = per_byte.saturating_mul(len);

            // the adjustable part of the fee.
            let unadjusted_weight_fee = Self::weight_to_fee(weight);
            let multiplier = Self::next_fee_multiplier();
            // final adjusted weight fee.
            let adjusted_weight_fee = multiplier.saturating_mul_int(unadjusted_weight_fee);

            let base_weight = Self::weight_to_fee(T::ExtrinsicBaseWeight::get());
            base_weight
                .saturating_add(fixed_len_fee)
                .saturating_add(adjusted_weight_fee)
                .saturating_add(tip)
        } else {
            tip
        }
    }
}

impl<T: Trait> Module<T> {
    /// Compute the fee for the specified weight.
    ///
    /// This fee is already adjusted by the per block fee adjustment factor and is therefore
    /// the share that the weight contributes to the overall fee of a transaction.
    ///
    /// This function is generic in order to supply the contracts module with a way
    /// to calculate the gas price. The contracts module is not able to put the necessary
    /// `BalanceOf<T>` contraints on its trait. This function is not to be used by this module.
    pub fn weight_to_fee_with_adjustment<Balance>(weight: Weight) -> Balance
    where
        Balance: UniqueSaturatedFrom<u128>,
    {
        let fee: u128 = Self::weight_to_fee(weight).unique_saturated_into();
        Balance::unique_saturated_from(NextFeeMultiplier::get().saturating_mul_acc_int(fee))
    }

    fn weight_to_fee(weight: Weight) -> BalanceOf<T> {
        // cap the weight to the maximum defined in runtime, otherwise it will be the
        // `Bounded` maximum of its data type, which is not desired.
        let capped_weight = weight.min(<T as frame_system::Trait>::MaximumBlockWeight::get());
        T::WeightToFee::calc(&capped_weight)
    }
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
#[derive(Encode, Decode, Clone, Eq, PartialEq)]
pub struct ChargeTransactionPayment<T: Trait + Send + Sync>(#[codec(compact)] BalanceOf<T>);

impl<T: Trait + Send + Sync> ChargeTransactionPayment<T>
where
    T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync + FixedPointOperand,
{
    /// utility constructor. Used only in client/factory code.
    pub fn from(fee: BalanceOf<T>) -> Self {
        Self(fee)
    }

    fn withdraw_fee(
        &self,
        call: &T::Call,
        who: &T::AccountId,
        info: &DispatchInfoOf<T::Call>,
        len: usize,
    ) -> Result<(BalanceOf<T>, Option<NegativeImbalanceOf<T>>), TransactionValidityError> {
        let fee = Module::<T>::compute_fee(len as u32, info, 0u32.into());

        // Only mess with balances if fee is not zero.
        if fee.is_zero() {
            return Ok((fee, None));
        }

        if let Some(payer) = T::CddHandler::get_valid_payer(call, &Signatory::Account(who.clone()))?
        {
            let imbalance;
            match payer.clone() {
                Signatory::Account(payer_key) => {
                    imbalance = T::Currency::withdraw(
                        &payer_key,
                        fee,
                        WithdrawReason::TransactionPayment.into(),
                        ExistenceRequirement::KeepAlive,
                    )
                    .map_err(|_| InvalidTransaction::Payment)?;
                }
                Signatory::Identity(did) => {
                    imbalance = T::Currency::withdraw_identity_balance(&did, fee)
                        .map_err(|_| InvalidTransaction::Payment)?;
                }
            }
            T::CddHandler::set_payer_context(Some(payer));
            return Ok((fee, Some(imbalance)));
        } else {
            Err(InvalidTransaction::Payment.into())
        }
    }
}

impl<T: Trait + Send + Sync> sp_std::fmt::Debug for ChargeTransactionPayment<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ChargeTransactionPayment<{:?}>", self.0)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Trait + Send + Sync> SignedExtension for ChargeTransactionPayment<T>
where
    BalanceOf<T>: Send + Sync + From<u64> + FixedPointOperand,
    T::Call: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";
    type AccountId = T::AccountId;
    type Call = T::Call;
    type AdditionalSigned = ();
    type Pre = (
        BalanceOf<T>,
        Self::AccountId,
        Option<NegativeImbalanceOf<T>>,
        BalanceOf<T>,
    );
    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }
    // Polymesh note: Almost all of this function was re written to enforce zero tip and charge fee to proper payer.
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> TransactionValidity {
        if self.0 != Zero::zero() {
            // Tip must be set to zero.
            // This is enforced to curb front running.
            return InvalidTransaction::Custom(TransactionError::ZeroTip as u8).into();
        }
        let (fee, _) = self.withdraw_fee(call, who, info, len)?;
        let mut r = ValidTransaction::default();
        // NOTE: we probably want to maximize the _fee (of any type) per weight unit_ here, which
        // will be a bit more than setting the priority to tip. For now, this is enough.
        r.priority = fee.saturated_into::<TransactionPriority>();
        Ok(r)
    }

    fn pre_dispatch(
        self,
        who: &Self::AccountId,
        call: &Self::Call,
        info: &DispatchInfoOf<Self::Call>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        if self.0 != Zero::zero() {
            // Tip must be set to zero.
            // This is enforced to curb front running.
            return Err(TransactionValidityError::Invalid(
                InvalidTransaction::Custom(TransactionError::ZeroTip as u8),
            ));
        }
        let (fee, imbalance) = self.withdraw_fee(call, who, info, len)?;
        Ok((Zero::zero(), who.clone(), imbalance, fee))
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<Self::Call>,
        post_info: &PostDispatchInfoOf<Self::Call>,
        len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        let (tip, who, imbalance, fee) = pre;
        if let Some(payed) = imbalance {
            let actual_fee = Module::<T>::compute_actual_fee(len as u32, info, post_info, tip);
            let refund = fee.saturating_sub(actual_fee);
            if let Some(payer) = T::CddHandler::get_payer_from_context() {
                let actual_payment;
                match payer {
                    Signatory::Account(payer_account) => {
                        actual_payment =
                            match T::Currency::deposit_into_existing(&payer_account, refund) {
                                Ok(refund_imbalance) => {
                                    // The refund cannot be larger than the up front payed max weight.
                                    // `PostDispatchInfo::calc_unspent` guards against such a case.
                                    match payed.offset(refund_imbalance) {
                                        Ok(actual_payment) => actual_payment,
                                        Err(_) => return Err(InvalidTransaction::Payment.into()),
                                    }
                                }
                                // We do not recreate the account using the refund. The up front payment
                                // is gone in that case.
                                Err(_) => payed,
                            };
                    }
                    Signatory::Identity(id) => {
                        actual_payment =
                            match T::Currency::deposit_into_existing_identity(&id, refund) {
                                Ok(refund_imbalance) => {
                                    // The refund cannot be larger than the up front payed max weight.
                                    // `PostDispatchInfo::calc_unspent` guards against such a case.
                                    match payed.offset(refund_imbalance) {
                                        Ok(actual_payment) => actual_payment,
                                        Err(_) => return Err(InvalidTransaction::Payment.into()),
                                    }
                                }
                                // We do not recreate the account using the refund. The up front payment
                                // is gone in that case.
                                Err(_) => payed,
                            };
                    }
                }
                T::OnTransactionPayment::on_unbalanced(actual_payment);
            }
        }
        // It clears the identity and payer in the context after transaction.
        T::CddHandler::clear_context();
        Ok(())
    }
}

// Polymesh note: This was specifically added for Polymesh
pub trait CddAndFeeDetails<AccountId, Call> {
    fn get_valid_payer(
        call: &Call,
        caller: &Signatory<AccountId>,
    ) -> Result<Option<Signatory<AccountId>>, InvalidTransaction>;
    fn clear_context();
    fn set_payer_context(payer: Option<Signatory<AccountId>>);
    fn get_payer_from_context() -> Option<Signatory<AccountId>>;
    fn set_current_identity(did: &IdentityId);
}

// Polymesh note: This was specifically added for Polymesh
pub trait ChargeTxFee {
    fn charge_fee(len: u32, info: DispatchInfo) -> TransactionValidity;
}

// Polymesh note: This was specifically added for Polymesh
impl<T: Trait> ChargeTxFee for Module<T>
where
    BalanceOf<T>: FixedPointOperand,
    T::Call: Dispatchable<Info = DispatchInfo>,
{
    fn charge_fee(len: u32, info: DispatchInfoOf<T::Call>) -> TransactionValidity {
        let fee = Self::compute_fee(len as u32, &info, 0u32.into());
        if let Some(who) = T::CddHandler::get_payer_from_context() {
            let imbalance = match who {
                Signatory::Identity(did) => T::Currency::withdraw_identity_balance(&did, fee)
                    .map_err(|_| InvalidTransaction::Payment),
                Signatory::Account(account) => T::Currency::withdraw(
                    &account,
                    fee,
                    WithdrawReason::TransactionPayment.into(),
                    ExistenceRequirement::KeepAlive,
                )
                .map_err(|_| InvalidTransaction::Payment),
            }?;
            T::OnTransactionPayment::on_unbalanced(imbalance);
        }
        Ok(ValidTransaction::default())
    }
}

#[cfg(test)]
mod tests {
    use super::{ChargeTxFee, *};
    use codec::Encode;
    use frame_support::{
        impl_outer_dispatch, impl_outer_event, impl_outer_origin, parameter_types,
        weights::{
            DispatchClass, DispatchInfo, GetDispatchInfo, PostDispatchInfo, Weight,
            WeightToFeeCoefficient, WeightToFeeCoefficients, WeightToFeePolynomial,
        },
    };
    use pallet_balances::Call as BalancesCall;
    use pallet_transaction_payment_rpc_runtime_api::RuntimeDispatchInfo;
    use polymesh_common_utilities::{
        traits::{
            asset::AcceptTransfer,
            balances::{self, AccountData, CheckCdd},
            identity::IdentityTrait,
            CommonTrait,
        },
        SystematicIssuers,
    };
    use primitives::{IdentityId, Permission};
    use smallvec::smallvec;
    use sp_core::H256;
    use sp_runtime::{
        testing::{Header, TestXt},
        traits::{BlakeTwo256, IdentityLookup},
        Perbill,
    };
    use std::cell::RefCell;

    pub type AccountId = u64;
    pub type Balance = u128;

    const CALL: &<Runtime as frame_system::Trait>::Call =
        &Call::Balances(BalancesCall::transfer(2, 69));

    impl_outer_dispatch! {
        pub enum Call for Runtime where origin: Origin {
            pallet_balances::Balances,
            frame_system::System,
        }
    }

    impl_outer_event! {
        pub enum Event for Runtime {
            system<T>,
            pallet_balances<T>,
        }
    }

    #[derive(Clone, PartialEq, Eq, Debug)]
    pub struct Runtime;

    use frame_system as system;
    impl_outer_origin! {
        pub enum Origin for Runtime {}
    }

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u64 = 4096;
        pub const MaximumBlockLength: u32 = 4096;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
        pub const MaximumExtrinsicWeight: u64 = 2800;
        pub const BlockExecutionWeight: u64 = 10;
    }

    thread_local! {
        static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
    }

    pub struct ExtrinsicBaseWeight;
    impl Get<u64> for ExtrinsicBaseWeight {
        fn get() -> u64 {
            EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow())
        }
    }

    impl frame_system::Trait for Runtime {
        /// The basic call filter to use in dispatchable.
        type BaseCallFilter = ();
        /// The identifier used to distinguish between accounts.
        type AccountId = AccountId;
        /// The aggregated dispatch type that is available for extrinsics.
        type Call = Call;
        /// The lookup mechanism to get account ID from whatever is passed in dispatchers.
        type Lookup = IdentityLookup<Self::AccountId>;
        /// The index type for storing how many extrinsics an account has signed.
        type Index = u64;
        /// The index type for blocks.
        type BlockNumber = u64;
        /// The type for hashing blocks and tries.
        type Hash = H256;
        /// The hashing algorithm used.
        type Hashing = BlakeTwo256;
        /// The header type.
        type Header = Header;
        /// The ubiquitous event type.
        type Event = Event;
        /// The ubiquitous origin type.
        type Origin = Origin;
        /// Maximum number of block number to block hash mappings to keep (oldest pruned first).
        type BlockHashCount = BlockHashCount;
        /// Maximum weight of each block.
        type MaximumBlockWeight = MaximumBlockWeight;
        /// Maximum size of all encoded transactions (in bytes) that are allowed in one block.
        type MaximumBlockLength = MaximumBlockLength;
        /// Portion of the block weight that is available to all normal transactions.
        type AvailableBlockRatio = AvailableBlockRatio;
        /// Version of the runtime.
        type Version = ();
        /// Converts a module to the index of the module in `construct_runtime!`.
        ///
        /// This type is being generated by `construct_runtime!`.
        type ModuleToIndex = ();
        /// What to do if a new account is created.
        type OnNewAccount = ();
        /// What to do if an account is fully reaped from the system.
        type OnKilledAccount = ();
        /// The data to be stored in an account.
        type AccountData = AccountData<Balance>;
        /// The weight of database operations that the runtime can invoke.
        type DbWeight = ();
        /// The weight of the overhead invoked on the block import process, independent of the
        /// extrinsics included in that block.
        type BlockExecutionWeight = BlockExecutionWeight;
        /// The base weight of any extrinsic processed by the runtime, independent of the
        /// logic of that extrinsic. (Signature verification, nonce increment, fee, etc...)
        type ExtrinsicBaseWeight = ExtrinsicBaseWeight;
        /// The maximum weight that a single extrinsic of `Normal` dispatch class can have,
        /// independent of the logic of that extrinsics. (Roughly max block weight - average on
        /// initialize cost).
        type MaximumExtrinsicWeight = MaximumExtrinsicWeight;
    }

    parameter_types! {
        pub const ExistentialDeposit: u128 = 1;
    }

    impl CommonTrait for Runtime {
        type Balance = Balance;
        type AcceptTransferTarget = Runtime;
        type BlockRewardsReserve = pallet_balances::Module<Runtime>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl pallet_balances::Trait for Runtime {
        type DustRemoval = ();
        type Event = Event;
        type ExistentialDeposit = ExistentialDeposit;
        type AccountStore = frame_system::Module<Runtime>;
        type Identity = Runtime;
        type CddChecker = Runtime;
    }

    impl CheckCdd<AccountId> for Runtime {
        fn check_key_cdd(_key: &AccountId) -> bool {
            true
        }

        fn get_key_cdd_did(_key: &AccountId) -> Option<IdentityId> {
            None
        }
    }

    thread_local! {
        static TRANSACTION_BYTE_FEE: RefCell<u128> = RefCell::new(1);
        static WEIGHT_TO_FEE: RefCell<u128> = RefCell::new(1);
    }

    impl CddAndFeeDetails<AccountId, Call> for Runtime {
        fn get_valid_payer(
            _: &Call,
            caller: &Signatory<AccountId>,
        ) -> Result<Option<Signatory<AccountId>>, InvalidTransaction> {
            Ok(Some(*caller))
        }
        fn clear_context() {}
        fn set_payer_context(_: Option<Signatory<AccountId>>) {}
        fn get_payer_from_context() -> Option<Signatory<AccountId>> {
            Some(Signatory::Account(2))
        }
        fn set_current_identity(_: &IdentityId) {}
    }

    impl IdentityTrait<AccountId> for Runtime {
        fn get_identity(_key: &AccountId) -> Option<IdentityId> {
            unimplemented!()
        }
        fn current_payer() -> Option<Signatory<AccountId>> {
            None
        }
        fn current_identity() -> Option<IdentityId> {
            unimplemented!()
        }
        fn set_current_identity(_id: Option<IdentityId>) {
            unimplemented!()
        }
        fn set_current_payer(_payer: Option<Signatory<AccountId>>) {}
        fn is_signer_authorized(_did: IdentityId, _signer: &Signatory<AccountId>) -> bool {
            unimplemented!()
        }
        fn is_signer_authorized_with_permissions(
            _did: IdentityId,
            _signer: &Signatory<AccountId>,
            _permissions: Vec<Permission>,
        ) -> bool {
            unimplemented!()
        }
        fn is_master_key(_did: IdentityId, _key: &AccountId) -> bool {
            unimplemented!()
        }

        fn unsafe_add_systematic_cdd_claims(_targets: &[IdentityId], _issuer: SystematicIssuers) {}
        fn unsafe_revoke_systematic_cdd_claims(
            _targets: &[IdentityId],
            _issuer: SystematicIssuers,
        ) {
        }
        fn has_valid_cdd(_target_did: IdentityId) -> bool {
            unimplemented!()
        }
    }

    impl AcceptTransfer for Runtime {
        fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
            Ok(())
        }
        fn accept_asset_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
            Ok(())
        }
    }
    pub struct TransactionByteFee;
    impl Get<u128> for TransactionByteFee {
        fn get() -> u128 {
            TRANSACTION_BYTE_FEE.with(|v| *v.borrow())
        }
    }
    pub struct WeightToFee;
    impl WeightToFeePolynomial for WeightToFee {
        type Balance = Balance;

        fn polynomial() -> WeightToFeeCoefficients<Self::Balance> {
            smallvec![WeightToFeeCoefficient {
                degree: 1,
                coeff_frac: Perbill::zero(),
                coeff_integer: WEIGHT_TO_FEE.with(|v| *v.borrow()),
                negative: false,
            }]
        }
    }

    impl Trait for Runtime {
        type Currency = pallet_balances::Module<Runtime>;
        type OnTransactionPayment = ();
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = WeightToFee;
        type FeeMultiplierUpdate = ();
        type CddHandler = Runtime;
    }

    impl ChargeTxFee for Runtime {
        fn charge_fee(_len: u32, _info: DispatchInfo) -> TransactionValidity {
            Ok(ValidTransaction::default())
        }
    }

    type Balances = pallet_balances::Module<Runtime>;
    type System = frame_system::Module<Runtime>;
    type TransactionPayment = Module<Runtime>;

    pub struct ExtBuilder {
        balance_factor: u128,
        base_weight: u64,
        byte_fee: u128,
        weight_to_fee: u128,
    }

    impl Default for ExtBuilder {
        fn default() -> Self {
            Self {
                balance_factor: 1,
                base_weight: 0,
                byte_fee: 1,
                weight_to_fee: 1,
            }
        }
    }

    impl ExtBuilder {
        pub fn base_weight(mut self, base_weight: u64) -> Self {
            self.base_weight = base_weight;
            self
        }
        pub fn byte_fee(mut self, byte_fee: u128) -> Self {
            self.byte_fee = byte_fee;
            self
        }
        pub fn weight_fee(mut self, weight_to_fee: u128) -> Self {
            self.weight_to_fee = weight_to_fee;
            self
        }
        pub fn balance_factor(mut self, factor: u128) -> Self {
            self.balance_factor = factor;
            self
        }
        fn set_constants(&self) {
            EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.base_weight);
            TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.byte_fee);
            WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
        }
        pub fn build(self) -> sp_io::TestExternalities {
            self.set_constants();
            let mut t = frame_system::GenesisConfig::default()
                .build_storage::<Runtime>()
                .unwrap();
            pallet_balances::GenesisConfig::<Runtime> {
                balances: if self.balance_factor > 0 {
                    vec![
                        (1, 10 * self.balance_factor),
                        (2, 20 * self.balance_factor),
                        (3, 30 * self.balance_factor),
                        (4, 40 * self.balance_factor),
                        (5, 50 * self.balance_factor),
                        (6, 60 * self.balance_factor),
                    ]
                } else {
                    vec![]
                },
                identity_balances: vec![],
            }
            .assimilate_storage(&mut t)
            .unwrap();
            t.into()
        }
    }

    /// create a transaction info struct from weight. Handy to avoid building the whole struct.
    pub fn info_from_weight(w: Weight) -> DispatchInfo {
        // pays_fee: Pays::Yes -- class: DispatchClass::Normal
        DispatchInfo {
            weight: w,
            ..Default::default()
        }
    }

    fn post_info_from_weight(w: Weight) -> PostDispatchInfo {
        PostDispatchInfo {
            actual_weight: Some(w),
        }
    }

    fn default_post_info() -> PostDispatchInfo {
        PostDispatchInfo {
            actual_weight: None,
        }
    }

    #[test]
    fn signed_extension_transaction_payment_work() {
        ExtBuilder::default()
            .balance_factor(10)
            .base_weight(5)
            .build()
            .execute_with(|| {
                let len = 10;
                let pre = ChargeTransactionPayment::<Runtime>::from(0)
                    .pre_dispatch(&1, CALL, &info_from_weight(5), len)
                    .unwrap();
                assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

                assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &info_from_weight(5),
                    &default_post_info(),
                    len,
                    &Ok(())
                )
                .is_ok());
                assert_eq!(Balances::free_balance(1), 100 - 5 - 5 - 10);

                let pre = ChargeTransactionPayment::<Runtime>::from(0 /* tipped */)
                    .pre_dispatch(&2, CALL, &info_from_weight(100), len)
                    .unwrap();
                assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 100 - 0);

                assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &info_from_weight(100),
                    &post_info_from_weight(50),
                    len,
                    &Ok(())
                )
                .is_ok());
                assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 50 - 0);
            });
    }

    #[test]
    fn signed_extension_transaction_payment_multiplied_refund_works() {
        ExtBuilder::default()
            .balance_factor(10)
            .base_weight(5)
            .build()
            .execute_with(|| {
                let len = 10;
                NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

                let pre = ChargeTransactionPayment::<Runtime>::from(0 /* tipped */)
                    .pre_dispatch(&2, CALL, &info_from_weight(100), len)
                    .unwrap();
                // 5 base fee, 10 byte fee, 3/2 * 100 weight fee, 5 tip
                assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 150 - 0);

                assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &info_from_weight(100),
                    &post_info_from_weight(50),
                    len,
                    &Ok(())
                )
                .is_ok());
                // 75 (3/2 of the returned 50 units of weight) is refunded
                assert_eq!(Balances::free_balance(2), 200 - 5 - 10 - 75 - 0);
            });
    }

    #[test]
    fn signed_extension_transaction_payment_is_bounded() {
        ExtBuilder::default()
            .balance_factor(1000)
            .byte_fee(0)
            .build()
            .execute_with(|| {
                // maximum weight possible
                assert!(ChargeTransactionPayment::<Runtime>::from(0)
                    .pre_dispatch(&1, CALL, &info_from_weight(Weight::max_value()), 10)
                    .is_ok());
                // fee will be proportional to what is the actual maximum weight in the runtime.
                assert_eq!(
                    Balances::free_balance(&1),
                    (10000 - <Runtime as frame_system::Trait>::MaximumBlockWeight::get()) as u128
                );
            });
    }

    #[test]
    fn signed_extension_allows_free_transactions() {
        ExtBuilder::default()
            .base_weight(100)
            .balance_factor(0)
            .build()
            .execute_with(|| {
                // 1 ain't have a penny.
                assert_eq!(Balances::free_balance(1), 0);

                let len = 100;

                // This is a completely free (and thus wholly insecure/DoS-ridden) transaction.
                let operational_transaction = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::No,
                };
                assert!(ChargeTransactionPayment::<Runtime>::from(0)
                    .validate(&1, CALL, &operational_transaction, len)
                    .is_ok());

                // like a InsecureFreeNormal
                let free_transaction = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Normal,
                    pays_fee: Pays::Yes,
                };
                assert!(ChargeTransactionPayment::<Runtime>::from(0)
                    .validate(&1, CALL, &free_transaction, len)
                    .is_err());
            });
    }

    #[test]
    fn signed_ext_length_fee_is_also_updated_per_congestion() {
        ExtBuilder::default()
            .base_weight(5)
            .balance_factor(10)
            .build()
            .execute_with(|| {
                // all fees should be x1.5
                NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
                let len = 10;

                assert!(ChargeTransactionPayment::<Runtime>::from(0) // tipped
                    .pre_dispatch(&1, CALL, &info_from_weight(3), len)
                    .is_ok());
                assert_eq!(
                    Balances::free_balance(1),
                    100 // original
				- 0 // tip
				- 5 // base
				- 10 // len
				- (3 * 3 / 2) // adjusted weight
                );
            })
    }

    #[test]
    fn query_info_works() {
        let call = Call::Balances(BalancesCall::transfer(2, 69));
        let origin = 111111;
        let extra = ();
        let xt = TestXt::new(call, Some((origin, extra)));
        let info = xt.get_dispatch_info();
        let ext = xt.encode();
        let len = ext.len() as u32;
        ExtBuilder::default()
            .base_weight(5)
            .weight_fee(2)
            .build()
            .execute_with(|| {
                // all fees should be x1.5
                NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));

                assert_eq!(
                    TransactionPayment::query_info(xt, len),
                    RuntimeDispatchInfo {
                        weight: info.weight,
                        class: info.class,
                        partial_fee: 5 * 2 /* base * weight_fee */
						+ len as u128  /* len * 1 */
						+ info.weight.min(MaximumBlockWeight::get()) as u128 * 2 * 3 / 2 /* weight */
                    },
                );
            });
    }

    #[test]
    fn compute_fee_works_without_multiplier() {
        ExtBuilder::default()
            .base_weight(100)
            .byte_fee(10)
            .balance_factor(0)
            .build()
            .execute_with(|| {
                // Next fee multiplier is zero
                assert_eq!(NextFeeMultiplier::get(), Multiplier::one());

                // Tip only, no fees works
                let dispatch_info = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::No,
                };
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 10), 10);
                // No tip, only base fee works
                let dispatch_info = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);
                // Tip + base fee works
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 69), 169);
                // Len (byte fee) + base fee works
                assert_eq!(Module::<Runtime>::compute_fee(42, &dispatch_info, 0), 520);
                // Weight fee + base fee works
                let dispatch_info = DispatchInfo {
                    weight: 1000,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 1100);
            });
    }

    #[test]
    fn compute_fee_works_with_multiplier() {
        ExtBuilder::default()
            .base_weight(100)
            .byte_fee(10)
            .balance_factor(0)
            .build()
            .execute_with(|| {
                // Add a next fee multiplier. Fees will be x3/2.
                NextFeeMultiplier::put(Multiplier::saturating_from_rational(3, 2));
                // Base fee is unaffected by multiplier
                let dispatch_info = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

                // Everything works together :)
                let dispatch_info = DispatchInfo {
                    weight: 123,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                // 123 weight, 456 length, 100 base
                assert_eq!(
                    Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
                    100 + (3 * 123 / 2) + 4560 + 789,
                );
            });
    }

    #[test]
    fn compute_fee_works_with_negative_multiplier() {
        ExtBuilder::default()
            .base_weight(100)
            .byte_fee(10)
            .balance_factor(0)
            .build()
            .execute_with(|| {
                // Add a next fee multiplier. All fees will be x1/2.
                NextFeeMultiplier::put(Multiplier::saturating_from_rational(1, 2));

                // Base fee is unaffected by multiplier.
                let dispatch_info = DispatchInfo {
                    weight: 0,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                assert_eq!(Module::<Runtime>::compute_fee(0, &dispatch_info, 0), 100);

                // Everything works together.
                let dispatch_info = DispatchInfo {
                    weight: 123,
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                // 123 weight, 456 length, 100 base
                assert_eq!(
                    Module::<Runtime>::compute_fee(456, &dispatch_info, 789),
                    100 + (123 / 2) + 4560 + 789,
                );
            });
    }

    #[test]
    fn compute_fee_does_not_overflow() {
        ExtBuilder::default()
            .base_weight(100)
            .byte_fee(10)
            .balance_factor(0)
            .build()
            .execute_with(|| {
                // Overflow is handled
                let dispatch_info = DispatchInfo {
                    weight: Weight::max_value(),
                    class: DispatchClass::Operational,
                    pays_fee: Pays::Yes,
                };
                assert_eq!(
                    Module::<Runtime>::compute_fee(
                        <u32>::max_value(),
                        &dispatch_info,
                        <u128>::max_value()
                    ),
                    <u128>::max_value()
                );
            });
    }

    #[test]
    fn actual_weight_higher_than_max_refunds_nothing() {
        ExtBuilder::default()
            .balance_factor(10)
            .base_weight(5)
            .build()
            .execute_with(|| {
                let len = 10;
                let pre = ChargeTransactionPayment::<Runtime>::from(0 /* tipped */)
                    .pre_dispatch(&2, CALL, &info_from_weight(100), len)
                    .unwrap();
                assert_eq!(Balances::free_balance(2), 200 - 0 - 10 - 100 - 5);

                assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &info_from_weight(100),
                    &post_info_from_weight(101),
                    len,
                    &Ok(())
                )
                .is_ok());
                assert_eq!(Balances::free_balance(2), 200 - 0 - 10 - 100 - 5);
            });
    }

    #[test]
    fn zero_transfer_on_free_transaction() {
        ExtBuilder::default()
            .balance_factor(10)
            .base_weight(5)
            .build()
            .execute_with(|| {
                // So events are emitted
                System::set_block_number(10);
                let len = 10;
                let dispatch_info = DispatchInfo {
                    weight: 100,
                    pays_fee: Pays::No,
                    class: DispatchClass::Normal,
                };
                let user = 69;
                let pre = ChargeTransactionPayment::<Runtime>::from(0)
                    .pre_dispatch(&user, CALL, &dispatch_info, len)
                    .unwrap();
                assert_eq!(Balances::total_balance(&user), 0);
                assert!(ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &dispatch_info,
                    &default_post_info(),
                    len,
                    &Ok(())
                )
                .is_ok());
                assert_eq!(Balances::total_balance(&user), 0);
                // No events for such a scenario
                assert_eq!(System::events().len(), 0);
            });
    }

    #[test]
    fn refund_consistent_with_actual_weight() {
        ExtBuilder::default()
            .balance_factor(10)
            .base_weight(7)
            .build()
            .execute_with(|| {
                let info = info_from_weight(100);
                let post_info = post_info_from_weight(33);
                let prev_balance = Balances::free_balance(2);
                let len = 10;
                let tip = 0;

                NextFeeMultiplier::put(Multiplier::saturating_from_rational(5, 4));

                let pre = ChargeTransactionPayment::<Runtime>::from(tip)
                    .pre_dispatch(&2, CALL, &info, len)
                    .unwrap();

                ChargeTransactionPayment::<Runtime>::post_dispatch(
                    pre,
                    &info,
                    &post_info,
                    len,
                    &Ok(()),
                )
                .unwrap();

                let refund_based_fee = prev_balance - Balances::free_balance(2);
                let actual_fee =
                    Module::<Runtime>::compute_actual_fee(len as u32, &info, &post_info, tip);

                // 33 weight, 10 length, 7 base, 5 tip
                assert_eq!(actual_fee, 7 + 10 + (33 * 5 / 4) + tip);
                assert_eq!(refund_based_fee, actual_fee);
            });
    }
}
