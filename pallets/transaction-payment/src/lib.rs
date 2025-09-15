// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
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

// Modified by Polymesh Association - 13rd March 2020
// - Charge fee from the identity in the signed extension
// - Tips have been removed.

//! # Transaction Payment Module
//!
//! This pallet provides the basic logic needed to pay the absolute minimum amount needed for a
//! transaction to be included. This includes:
//!   - _base fee_: This is the minimum amount a user pays for a transaction. It is declared
//!     as a base _weight_ in the runtime and converted to a fee using `WeightToFee`.
//!   - _weight fee_: A fee proportional to amount of weight a transaction consumes.
//!   - _length fee_: A fee proportional to the encoded length of the transaction.
//!   - _tip_: An optional tip. Tip increases the priority of the transaction, giving it a higher
//!     chance to be included by the transaction queue.
//!
//! The base fee and adjusted weight and length fees constitute the _inclusion fee_, which is
//! the minimum fee for a transaction to be included in a block.
//!
//! The formula of final fee:
//!   ```ignore
//!   inclusion_fee = base_fee + length_fee + [targeted_fee_adjustment * weight_fee];
//!   final_fee = inclusion_fee + tip;
//!   ```
//!
//!   - `targeted_fee_adjustment`: This is a multiplier that can tune the final fee based on
//!     the congestion of the network.
//!
//! Additionally, this pallet allows one to configure:
//!   - The mapping between one unit of weight to one unit of fee via [`Config::WeightToFee`].
//!   - A means of updating the fee for the next block, via defining a multiplier, based on the
//!     final state of the chain at the end of the previous block. This can be configured via
//!     [`Config::FeeMultiplierUpdate`]
//!   - How the fees are paid via [`Config::OnChargeTransaction`].

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::dispatch::{DispatchClass, DispatchInfo, DispatchResult};
use frame_support::dispatch::{GetDispatchInfo, Pays, PostDispatchInfo};
use frame_support::pallet_prelude::*;
use frame_support::traits::Get;
use frame_support::weights::{Weight, WeightToFee};
use frame_support::RuntimeDebugNoBound;
use frame_system::pallet_prelude::{BlockNumberFor, OriginFor};
use scale_info::TypeInfo;
use sp_runtime::traits::SaturatedConversion;
use sp_runtime::traits::{AsSystemOriginSigner, Saturating, TransactionExtension, Zero};
use sp_runtime::traits::{Convert, DispatchInfoOf, Dispatchable, PostDispatchInfoOf};
use sp_runtime::transaction_validity::{TransactionValidityError, ValidTransaction};
use sp_runtime::{FixedPointNumber, FixedPointOperand, FixedU128};
use sp_runtime::{Perbill, Perquintill, RuntimeDebug};

use polymesh_primitives::traits::group::GroupTrait;
use polymesh_primitives::traits::{CddAndFeeDetails, IdentityFnTrait, SubsidiserTrait};
use polymesh_primitives::TransactionError;

pub use pallet::*;
pub use payment::*;
pub use types::{FeeDetails, InclusionFee, RuntimeDispatchInfo};

mod payment;
mod types;

/// Fee multiplier.
pub type Multiplier = FixedU128;

type BalanceOf<T> = <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;

/// A struct to update the weight multiplier per block. It implements `Convert<Multiplier,
/// Multiplier>`, meaning that it can convert the previous multiplier to the next one. This should
/// be called on `on_finalize` of a block, prior to potentially cleaning the weight data from the
/// system pallet.
///
/// given:
/// 	s = previous block weight
/// 	s'= ideal block weight
/// 	m = maximum block weight
/// 		diff = (s - s')/m
/// 		v = 0.00001
/// 		t1 = (v * diff)
/// 		t2 = (v * diff)^2 / 2
/// 	then:
/// 	next_multiplier = prev_multiplier * (1 + t1 + t2)
///
/// Where `(s', v)` must be given as the `Get` implementation of the `T` generic type. Moreover, `M`
/// must provide the minimum allowed value for the multiplier. Note that a runtime should ensure
/// with tests that the combination of this `M` and `V` is not such that the multiplier can drop to
/// zero and never recover.
///
/// Note that `s'` is interpreted as a portion in the _normal transaction_ capacity of the block.
/// For example, given `s' == 0.25` and `AvailableBlockRatio = 0.75`, then the target fullness is
/// _0.25 of the normal capacity_ and _0.1875 of the entire block_.
///
/// Since block weight is multi-dimension, we use the scarcer resource, referred as limiting
/// dimension, for calculation of fees. We determine the limiting dimension by comparing the
/// dimensions using the ratio of `dimension_value / max_dimension_value` and selecting the largest
/// ratio. For instance, if a block is 30% full based on `ref_time` and 25% full based on
/// `proof_size`, we identify `ref_time` as the limiting dimension, indicating that the block is 30%
/// full.
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
/// For example, when all blocks are full and there are 28800 blocks per day (default in
/// `substrate-node`) and v == 0.00001, s' == 0.1875, we'd have:
///
/// p >= 0.00001 * 28800 * 0.8125
/// p >= 0.234
///
/// Meaning that fees can change by around ~23% per day, given extreme congestion.
///
/// More info can be found at:
/// <https://research.web3.foundation/Polkadot/overview/token-economics>
pub struct TargetedFeeAdjustment<T, S, V, M, X>(sp_std::marker::PhantomData<(T, S, V, M, X)>);

/// Something that can convert the current multiplier to the next one.
pub trait MultiplierUpdate: Convert<Multiplier, Multiplier> {
    /// Minimum multiplier. Any outcome of the `convert` function should be at least this.
    fn min() -> Multiplier;
    /// Maximum multiplier. Any outcome of the `convert` function should be less or equal this.
    fn max() -> Multiplier;
    /// Target block saturation level
    fn target() -> Perquintill;
    /// Variability factor
    fn variability() -> Multiplier;
}

impl MultiplierUpdate for () {
    fn min() -> Multiplier {
        Default::default()
    }
    fn max() -> Multiplier {
        <Multiplier as sp_runtime::traits::Bounded>::max_value()
    }
    fn target() -> Perquintill {
        Default::default()
    }
    fn variability() -> Multiplier {
        Default::default()
    }
}

impl<T, S, V, M, X> MultiplierUpdate for TargetedFeeAdjustment<T, S, V, M, X>
where
    T: frame_system::Config,
    S: Get<Perquintill>,
    V: Get<Multiplier>,
    M: Get<Multiplier>,
    X: Get<Multiplier>,
{
    fn min() -> Multiplier {
        M::get()
    }
    fn max() -> Multiplier {
        X::get()
    }
    fn target() -> Perquintill {
        S::get()
    }
    fn variability() -> Multiplier {
        V::get()
    }
}

impl<T, S, V, M, X> Convert<Multiplier, Multiplier> for TargetedFeeAdjustment<T, S, V, M, X>
where
    T: frame_system::Config,
    S: Get<Perquintill>,
    V: Get<Multiplier>,
    M: Get<Multiplier>,
    X: Get<Multiplier>,
{
    fn convert(previous: Multiplier) -> Multiplier {
        // Defensive only. The multiplier in storage should always be at most positive. Nonetheless
        // we recover here in case of errors, because any value below this would be stale and can
        // never change.
        let min_multiplier = M::get();
        let max_multiplier = X::get();
        let previous = previous.max(min_multiplier);

        let weights = T::BlockWeights::get();
        // the computed ratio is only among the normal class.
        let normal_max_weight = weights
            .get(DispatchClass::Normal)
            .max_total
            .unwrap_or(weights.max_block);
        let current_block_weight = frame_system::Pallet::<T>::block_weight();
        let normal_block_weight = current_block_weight
            .get(DispatchClass::Normal)
            .min(normal_max_weight);

        // Normalize dimensions so they can be compared. Ensure (defensive) max weight is non-zero.
        let normalized_ref_time = Perbill::from_rational(
            normal_block_weight.ref_time(),
            normal_max_weight.ref_time().max(1),
        );
        let normalized_proof_size = Perbill::from_rational(
            normal_block_weight.proof_size(),
            normal_max_weight.proof_size().max(1),
        );

        // Pick the limiting dimension. If the proof size is the limiting dimension, then the
        // multiplier is adjusted by the proof size. Otherwise, it is adjusted by the ref time.
        let (normal_limiting_dimension, max_limiting_dimension) =
            if normalized_ref_time < normalized_proof_size {
                (
                    normal_block_weight.proof_size(),
                    normal_max_weight.proof_size(),
                )
            } else {
                (normal_block_weight.ref_time(), normal_max_weight.ref_time())
            };

        let target_block_fullness = S::get();
        let adjustment_variable = V::get();

        let target_weight = (target_block_fullness * max_limiting_dimension) as u128;
        let block_weight = normal_limiting_dimension as u128;

        // determines if the first_term is positive
        let positive = block_weight >= target_weight;
        let diff_abs = block_weight.max(target_weight) - block_weight.min(target_weight);

        // defensive only, a test case assures that the maximum weight diff can fit in Multiplier
        // without any saturation.
        let diff = Multiplier::saturating_from_rational(diff_abs, max_limiting_dimension.max(1));
        let diff_squared = diff.saturating_mul(diff);

        let v_squared_2 = adjustment_variable.saturating_mul(adjustment_variable)
            / Multiplier::saturating_from_integer(2);

        let first_term = adjustment_variable.saturating_mul(diff);
        let second_term = v_squared_2.saturating_mul(diff_squared);

        if positive {
            let excess = first_term
                .saturating_add(second_term)
                .saturating_mul(previous);
            previous
                .saturating_add(excess)
                .clamp(min_multiplier, max_multiplier)
        } else {
            // Defensive-only: first_term > second_term. Safe subtraction.
            let negative = first_term
                .saturating_sub(second_term)
                .saturating_mul(previous);
            previous
                .saturating_sub(negative)
                .clamp(min_multiplier, max_multiplier)
        }
    }
}

/// A struct to make the fee multiplier a constant
pub struct ConstFeeMultiplier<M: Get<Multiplier>>(core::marker::PhantomData<M>);

impl<M: Get<Multiplier>> MultiplierUpdate for ConstFeeMultiplier<M> {
    fn min() -> Multiplier {
        M::get()
    }
    fn max() -> Multiplier {
        M::get()
    }
    fn target() -> Perquintill {
        Default::default()
    }
    fn variability() -> Multiplier {
        Default::default()
    }
}

impl<M> Convert<Multiplier, Multiplier> for ConstFeeMultiplier<M>
where
    M: Get<Multiplier>,
{
    fn convert(_previous: Multiplier) -> Multiplier {
        Self::min()
    }
}

/// Storage releases of the pallet.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Copy, Default, Eq, MaxEncodedLen, PartialEq, RuntimeDebug)]
pub enum Releases {
    /// Original version of the pallet.
    #[default]
    V1Ancient,
    /// One that bumps the usage to FixedU128 from FixedI128.
    V2,
}

const MULTIPLIER_DEFAULT_VALUE: Multiplier = Multiplier::from_u32(1);

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config: frame_system::Config + pallet_timestamp::Config {
        /// The overarching event type.
        type RuntimeEvent: From<Event<Self>> + IsType<<Self as frame_system::Config>::RuntimeEvent>;

        /// Handler for withdrawing, refunding and depositing the transaction fee.
        /// Transaction fees are withdrawn before the transaction is executed.
        /// After the transaction was executed the transaction weight can be
        /// adjusted, depending on the used resources by the transaction. If the
        /// transaction weight is lower than expected, parts of the transaction fee
        /// might be refunded. In the end the fees can be deposited.
        type OnChargeTransaction: OnChargeTransaction<Self>;

        /// Convert a weight value into a deductible fee based on the currency type.
        type WeightToFee: WeightToFee<Balance = BalanceOf<Self>>;

        /// Convert a length value into a deductible fee based on the currency type.
        type LengthToFee: WeightToFee<Balance = BalanceOf<Self>>;

        /// Update the multiplier of the next block, based on the previous block's weight.
        type FeeMultiplierUpdate: MultiplierUpdate;

        /// A fee multiplier for `Operational` extrinsics to compute "virtual tip" to boost their
        /// `priority`
        ///
        /// This value is multiplied by the `final_fee` to obtain a "virtual tip" that is later
        /// added to a tip component in regular `priority` calculations.
        /// It means that a `Normal` transaction can front-run a similarly-sized `Operational`
        /// extrinsic (with no tip), by including a tip value greater than the virtual tip.
        ///
        /// ```rust,ignore
        /// // For `Normal`
        /// let priority = priority_calc(tip);
        ///
        /// // For `Operational`
        /// let virtual_tip = (inclusion_fee + tip) * OperationalFeeMultiplier;
        /// let priority = priority_calc(tip + virtual_tip);
        /// ```
        ///
        /// Note that since we use `final_fee` the multiplier applies also to the regular `tip`
        /// sent with the transaction. So, not only does the transaction get a priority bump based
        /// on the `inclusion_fee`, but we also amplify the impact of tips applied to `Operational`
        /// transactions.
        #[pallet::constant]
        type OperationalFeeMultiplier: Get<u8>;

        /// The weight information of this pallet.
        type WeightInfo: WeightInfo;

        // Polymesh change
        // -----------------------------------------------------------------

        /// Fetch the signatory to charge fee from. Also sets fee payer and identity in context.
        type CddHandler: CddAndFeeDetails<Self::AccountId, Self::RuntimeCall>;

        /// Used to charge transaction fees to a subsidiser, instead of the payer.
        type Subsidiser: SubsidiserTrait<Self::AccountId, Self::RuntimeCall>;

        type CddProviders: GroupTrait<Self::Moment>;

        type GovernanceCommittee: GroupTrait<Self::Moment>;

        type Identity: IdentityFnTrait<Self::AccountId>;

        // -----------------------------------------------------------------
    }

    #[pallet::type_value]
    pub fn NextFeeMultiplierOnEmpty() -> Multiplier {
        MULTIPLIER_DEFAULT_VALUE
    }

    #[pallet::storage]
    #[pallet::whitelist_storage]
    pub type NextFeeMultiplier<T: Config> =
        StorageValue<_, Multiplier, ValueQuery, NextFeeMultiplierOnEmpty>;

    #[pallet::storage]
    pub type StorageVersion<T: Config> = StorageValue<_, Releases, ValueQuery>;

    // Polymesh change
    // -----------------------------------------------------------------
    #[cfg(feature = "disable_fees")]
    #[pallet::storage]
    pub type DisableFees<T: Config> = StorageValue<_, bool, ValueQuery>;
    // -----------------------------------------------------------------

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        pub multiplier: Multiplier,
        #[serde(skip)]
        pub _config: core::marker::PhantomData<T>,
        #[cfg(feature = "disable_fees")]
        pub disable_fees: bool,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                multiplier: MULTIPLIER_DEFAULT_VALUE,
                _config: Default::default(),
                #[cfg(feature = "disable_fees")]
                disable_fees: false,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Releases::V2);
            NextFeeMultiplier::<T>::put(self.multiplier);
            #[cfg(feature = "disable_fees")]
            DisableFees::<T>::put(self.disable_fees);
        }
    }

    #[pallet::event]
    #[pallet::generate_deposit(pub(super) fn deposit_event)]
    pub enum Event<T: Config> {
        /// A transaction fee `actual_fee`, of which `tip` was added to the minimum inclusion fee,
        /// has been paid by `who`.
        TransactionFeePaid {
            who: T::AccountId,
            actual_fee: BalanceOf<T>,
            tip: BalanceOf<T>,
        },
    }

    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
        #[cfg(feature = "std")]
        fn integrity_test() {
            // given weight == u64, we build multipliers from `diff` of two weight values, which can
            // at most be maximum block weight. Make sure that this can fit in a multiplier without
            // loss.
            assert!(
                <Multiplier as sp_runtime::traits::Bounded>::max_value()
                    >= Multiplier::checked_from_integer::<u128>(
                        T::BlockWeights::get()
                            .max_block
                            .ref_time()
                            .try_into()
                            .unwrap()
                    )
                    .unwrap(),
            );

            let target = T::FeeMultiplierUpdate::target()
                * T::BlockWeights::get()
                    .get(DispatchClass::Normal)
                    .max_total
                    .expect(
                        "Setting `max_total` for `Normal` dispatch class is not compatible with \
					`transaction-payment` pallet.",
                    );
            // add 1 percent;
            let addition = target / 100;
            if addition == Weight::zero() {
                // this is most likely because in a test setup we set everything to ()
                // or to `ConstFeeMultiplier`.
                return;
            }

            // This is the minimum value of the multiplier. Make sure that if we collapse to this
            // value, we can recover with a reasonable amount of traffic. For this test we assert
            // that if we collapse to minimum, the trend will be positive with a weight value which
            // is 1% more than the target.
            let min_value = T::FeeMultiplierUpdate::min();
            let target = target + addition;

            frame_system::Pallet::<T>::set_block_consumed_resources(target, 0);
            let next = T::FeeMultiplierUpdate::convert(min_value);
            assert!(
                next > min_value,
                "The minimum bound of the multiplier is too low. When \
				block saturation is more than target by 1% and multiplier is minimal then \
				the multiplier doesn't increase."
            );
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn set_disable_fees(origin: OriginFor<T>, _value: bool) -> DispatchResult {
            frame_system::ensure_root(origin)?;
            #[cfg(feature = "disable_fees")]
            DisableFees::<T>::put(_value);
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T>
where
    BalanceOf<T>: FixedPointOperand,
{
    /// Query the data that we know about the fee of a given `call`.
    ///
    /// This pallet is not and cannot be aware of the internals of a signed extension, for example
    /// a tip. It only interprets the extrinsic as some encoded value and accounts for its weight
    /// and length, the runtime's extrinsic base weight, and the current fee multiplier.
    ///
    /// All dispatchables must be annotated with weight and will have some fee info. This function
    /// always returns.
    pub fn query_info<Extrinsic: sp_runtime::traits::ExtrinsicLike + GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> RuntimeDispatchInfo<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    {
        // NOTE: we can actually make it understand `ChargeTransactionPayment`, but would be some
        // hassle for sure. We have to make it aware of the index of `ChargeTransactionPayment` in
        // `Extra`. Alternatively, we could actually execute the tx's per-dispatch and record the
        // balance of the sender before and after the pipeline.. but this is way too much hassle for
        // a very very little potential gain in the future.
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);

        let partial_fee = if unchecked_extrinsic.is_bare() {
            // Bare extrinsics have no partial fee.
            0u32.into()
        } else {
            Self::compute_fee(len, &dispatch_info, 0u32.into())
        };

        let DispatchInfo { class, .. } = dispatch_info;

        RuntimeDispatchInfo {
            weight: dispatch_info.total_weight(),
            class,
            partial_fee,
        }
    }

    /// Query the detailed fee of a given `call`.
    pub fn query_fee_details<Extrinsic: sp_runtime::traits::ExtrinsicLike + GetDispatchInfo>(
        unchecked_extrinsic: Extrinsic,
        len: u32,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    {
        let dispatch_info = <Extrinsic as GetDispatchInfo>::get_dispatch_info(&unchecked_extrinsic);

        let tip = 0u32.into();

        if unchecked_extrinsic.is_bare() {
            // Bare extrinsics have no inclusion fee.
            FeeDetails {
                inclusion_fee: None,
                tip,
            }
        } else {
            Self::compute_fee_details(len, &dispatch_info, tip)
        }
    }

    /// Query information of a dispatch class, weight, and fee of a given encoded `Call`.
    pub fn query_call_info(call: T::RuntimeCall, len: u32) -> RuntimeDispatchInfo<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo> + GetDispatchInfo,
    {
        let dispatch_info = <T::RuntimeCall as GetDispatchInfo>::get_dispatch_info(&call);
        let DispatchInfo { class, .. } = dispatch_info;

        RuntimeDispatchInfo {
            weight: dispatch_info.total_weight(),
            class,
            partial_fee: Self::compute_fee(len, &dispatch_info, 0u32.into()),
        }
    }

    /// Query fee details of a given encoded `Call`.
    pub fn query_call_fee_details(call: T::RuntimeCall, len: u32) -> FeeDetails<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo> + GetDispatchInfo,
    {
        let dispatch_info = <T::RuntimeCall as GetDispatchInfo>::get_dispatch_info(&call);
        let tip = 0u32.into();

        Self::compute_fee_details(len, &dispatch_info, tip)
    }

    /// Compute the final fee value for a particular transaction.
    pub fn compute_fee(
        len: u32,
        info: &DispatchInfoOf<T::RuntimeCall>,
        tip: BalanceOf<T>,
    ) -> BalanceOf<T>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    {
        Self::compute_fee_details(len, info, tip).final_fee()
    }

    /// Compute the fee details for a particular transaction.
    pub fn compute_fee_details(
        len: u32,
        info: &DispatchInfoOf<T::RuntimeCall>,
        tip: BalanceOf<T>,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo>,
    {
        Self::compute_fee_raw(len, info.total_weight(), tip, info.pays_fee, info.class)
    }

    /// Compute the actual post dispatch fee for a particular transaction.
    ///
    /// Identical to `compute_fee` with the only difference that the post dispatch corrected
    /// weight is used for the weight fee calculation.
    pub fn compute_actual_fee(
        len: u32,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        tip: BalanceOf<T>,
    ) -> BalanceOf<T>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    {
        Self::compute_actual_fee_details(len, info, post_info, tip).final_fee()
    }

    /// Compute the actual post dispatch fee details for a particular transaction.
    pub fn compute_actual_fee_details(
        len: u32,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &PostDispatchInfoOf<T::RuntimeCall>,
        tip: BalanceOf<T>,
    ) -> FeeDetails<BalanceOf<T>>
    where
        T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    {
        Self::compute_fee_raw(
            len,
            post_info.calc_actual_weight(info),
            tip,
            post_info.pays_fee(info),
            info.class,
        )
    }

    fn compute_fee_raw(
        len: u32,
        weight: Weight,
        tip: BalanceOf<T>,
        pays_fee: Pays,
        class: DispatchClass,
    ) -> FeeDetails<BalanceOf<T>> {
        #[cfg(feature = "disable_fees")]
        if DisableFees::<T>::get() {
            return FeeDetails {
                inclusion_fee: None,
                tip: 0u32.into(),
            };
        }

        if pays_fee == Pays::Yes {
            // the adjustable part of the fee.
            let unadjusted_weight_fee = Self::weight_to_fee(weight);
            let multiplier = NextFeeMultiplier::<T>::get();
            // final adjusted weight fee.
            let adjusted_weight_fee = multiplier.saturating_mul_int(unadjusted_weight_fee);

            // length fee. this is adjusted via `LengthToFee`.
            let len_fee = Self::length_to_fee(len);

            let base_fee = Self::weight_to_fee(T::BlockWeights::get().get(class).base_extrinsic);
            FeeDetails {
                inclusion_fee: Some(InclusionFee {
                    base_fee,
                    len_fee,
                    adjusted_weight_fee,
                }),
                tip,
            }
        } else {
            FeeDetails {
                inclusion_fee: None,
                tip,
            }
        }
    }

    /// Compute the length portion of a fee by invoking the configured `LengthToFee` impl.
    pub fn length_to_fee(length: u32) -> BalanceOf<T> {
        T::LengthToFee::weight_to_fee(&Weight::from_parts(length as u64, 0))
    }

    /// Compute the unadjusted portion of the weight fee by invoking the configured `WeightToFee`
    /// impl. Note that the input `weight` is capped by the maximum block weight before computation.
    pub fn weight_to_fee(weight: Weight) -> BalanceOf<T> {
        // cap the weight to the maximum defined in runtime, otherwise it will be the
        // `Bounded` maximum of its data type, which is not desired.
        let capped_weight = weight.min(T::BlockWeights::get().max_block);
        T::WeightToFee::weight_to_fee(&capped_weight)
    }

    /// Deposit the [`Event::TransactionFeePaid`] event.
    pub fn deposit_fee_paid_event(who: T::AccountId, actual_fee: BalanceOf<T>, tip: BalanceOf<T>) {
        Self::deposit_event(Event::TransactionFeePaid {
            who,
            actual_fee,
            tip,
        });
    }

    // Polymesh change
    // -----------------------------------------------------------------
    #[cfg(debug_assertions)]
    pub fn put_next_fee_multiplier(m: Multiplier) {
        NextFeeMultiplier::<T>::put(m)
    }
    // -----------------------------------------------------------------
}

impl<T> Convert<Weight, BalanceOf<T>> for Pallet<T>
where
    T: Config,
{
    /// Compute the fee for the specified weight.
    ///
    /// This fee is already adjusted by the per block fee adjustment factor and is therefore the
    /// share that the weight contributes to the overall fee of a transaction. It is mainly
    /// for informational purposes and not used in the actual fee calculation.
    fn convert(weight: Weight) -> BalanceOf<T> {
        NextFeeMultiplier::<T>::get().saturating_mul_int(Self::weight_to_fee(weight))
    }
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
///
/// # Transaction Validity
///
/// This extension sets the `priority` field of `TransactionValidity` depending on the amount
/// of tip being paid per weight unit.
///
/// Operational transactions will receive an additional priority bump, so that they are normally
/// considered before regular transactions.
#[derive(Encode, Decode, DecodeWithMemTracking, Clone, Eq, PartialEq, TypeInfo)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPayment<T: Config>(#[codec(compact)] BalanceOf<T>);

impl<T: Config> ChargeTransactionPayment<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync + Into<u128>,
{
    /// utility constructor. Used only in client/factory code.
    pub fn from(fee: BalanceOf<T>) -> Self {
        Self(fee)
    }

    /// Returns the tip as being chosen by the transaction sender.
    pub fn tip(&self) -> BalanceOf<T> {
        self.0
    }

    pub(crate) fn can_withdraw_fee(
        &self,
        who: &T::AccountId,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<
        (
            BalanceOf<T>,
            <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
            Option<T::AccountId>,
        ),
        TransactionValidityError,
    > {
        let tip = self.0;
        let fee = Pallet::<T>::compute_fee(len as u32, info, tip);

        // Polymesh change
        // -----------------------------------------------------------------

        if fee.is_zero() {
            return Ok((fee, Default::default(), None));
        }

        // Get the payer for this transaction.
        let payers_key =
            T::CddHandler::get_valid_payer(call, who)?.ok_or(InvalidTransaction::Payment)?;

        // Check if the payer is being subsidised.
        let subsidiser = T::Subsidiser::check_subsidy(&payers_key, fee.into(), Some(call))?;

        // key to pay the fee.
        let fee_key = subsidiser.as_ref().unwrap_or(&payers_key);

        let liq_info =
            <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::withdraw_fee(
                fee_key, call, info, fee, tip,
            )?;

        T::CddHandler::set_payer_context(Some(payers_key));
        Ok((fee, liq_info, subsidiser))

        // -----------------------------------------------------------------
    }

    // Polymesh change: Used to allow GC/CDD member to include a `tip`.
    // -----------------------------------------------------------------

    /// Returns `true` if `who` is member of `T::GovernanceCommittee` or `T::CddProviders`.
    fn is_gc_or_cdd_member(who: &T::AccountId) -> bool {
        T::Identity::get_identity(who)
            .map(|did| T::GovernanceCommittee::is_member(&did) || T::CddProviders::is_member(&did))
            .unwrap_or(false)
    }

    /// Ensures that the transaction tip is valid.
    ///
    /// Tipping is allowed for `DispatchClass::Operational` created by a Governance or CDD Provider member.
    /// Mandatory transactions are going to be included in the block, so adding a tip does not matter.
    pub(crate) fn ensure_valid_tip(
        &self,
        who: &T::AccountId,
        info: &DispatchInfoOf<T::RuntimeCall>,
    ) -> Result<BalanceOf<T>, TransactionValidityError> {
        match info.class {
            DispatchClass::Normal | DispatchClass::Operational => {
                if self.0.is_zero() {
                    return Ok(self.0);
                }

                if Self::is_gc_or_cdd_member(who) {
                    return Ok(self.0);
                }

                Err(TransactionValidityError::Invalid(
                    InvalidTransaction::Custom(TransactionError::ZeroTip as u8),
                ))
            }
            DispatchClass::Mandatory => Ok(self.0),
        }
    }
}

/// The info passed between the validate and prepare steps for the `ChargeAssetTxPayment` extension.
#[derive(RuntimeDebugNoBound)]
pub enum Val<T: Config> {
    Charge {
        tip: BalanceOf<T>,
        // who called the transaction
        who: T::AccountId,
        // transaction fee
        fee: BalanceOf<T>,
        // Polymesh Subsidiser account (who paid the fee)
        subsidiser: Option<T::AccountId>,
    },
    NoCharge,
}

/// The info passed between the prepare and post-dispatch steps for the `ChargeAssetTxPayment`
/// extension.
pub enum Pre<T: Config> {
    Charge {
        tip: BalanceOf<T>,
        // who paid the fee
        who: T::AccountId,
        // imbalance resulting from withdrawing the fee
        imbalance: <<T as Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
        // Polymesh Subsidiser account (who paid the fee)
        subsidiser: Option<T::AccountId>,
    },
    NoCharge {
        // weight initially estimated by the extension, to be refunded
        refund: Weight,
    },
}

impl<T: Config> sp_std::fmt::Debug for ChargeTransactionPayment<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ChargeTransactionPayment<{:?}>", self.0)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config> TransactionExtension<T::RuntimeCall> for ChargeTransactionPayment<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    <T::RuntimeCall as Dispatchable>::RuntimeOrigin: AsSystemOriginSigner<T::AccountId>,
    BalanceOf<T>: Send + Sync + Into<u128>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";
    type Implicit = ();
    type Val = Val<T>;
    type Pre = Pre<T>;

    fn weight(&self, _: &T::RuntimeCall) -> Weight {
        Weight::zero()
    }

    fn validate(
        &self,
        origin: <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
        _: (),
        _implication: &impl Encode,
        _source: TransactionSource,
    ) -> Result<
        (
            ValidTransaction,
            Self::Val,
            <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        let caller_acc = origin
            .as_system_origin_signer()
            .ok_or(InvalidTransaction::BadSigner)?;

        let tip = self.ensure_valid_tip(caller_acc, info)?;

        let (fee, _, subsidiser) = self.can_withdraw_fee(caller_acc, call, info, len)?;

        let valid_transaction = ValidTransaction {
            priority: tip.saturated_into::<TransactionPriority>(),
            ..Default::default()
        };

        let val = Val::Charge {
            tip,
            who: caller_acc.clone(),
            fee,
            subsidiser,
        };

        Ok((valid_transaction, val, origin))
    }

    fn prepare(
        self,
        _val: Self::Val,
        origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        let caller_acc = origin
            .as_system_origin_signer()
            .ok_or(InvalidTransaction::BadSigner)?;

        let tip = self.ensure_valid_tip(caller_acc, info)?;

        let (_, imbalance, subsidiser) = self.can_withdraw_fee(caller_acc, call, info, len)?;

        let pre = Pre::Charge {
            tip,
            who: caller_acc.clone(),
            imbalance,
            subsidiser,
        };

        Ok(pre)
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &mut PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        _result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        let (tip, who, imbalance, subsidiser) = {
            match pre {
                Pre::Charge {
                    tip,
                    who,
                    imbalance,
                    subsidiser,
                } => (tip, who, imbalance, subsidiser),
                Pre::NoCharge { .. } => return Ok(()),
            }
        };

        let actual_fee = Pallet::<T>::compute_actual_fee(len as u32, info, post_info, tip);

        // Fee returned to original payer.
        let payers_key = T::CddHandler::get_payer_from_context().unwrap_or(who.clone());

        let fee_key = {
            if let Some(subsidiser_acc) = subsidiser {
                T::Subsidiser::debit_subsidy(&payers_key, actual_fee.into())?;
                subsidiser_acc
            } else {
                payers_key
            }
        };

        T::OnChargeTransaction::correct_and_deposit_fee(
            &fee_key, info, post_info, actual_fee, tip, imbalance,
        )?;

        Pallet::<T>::deposit_event(Event::<T>::TransactionFeePaid {
            who: fee_key,
            actual_fee,
            tip,
        });

        // It clears the identity and payer in the context after transaction.
        T::CddHandler::clear_context();
        Ok(())
    }
}

/// Weight functions needed for `pallet_transaction_payment`.
pub trait WeightInfo {
    fn charge_transaction_payment() -> Weight;
}
