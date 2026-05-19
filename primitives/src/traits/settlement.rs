use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::DispatchError;
use frame_support::weights::Weight;
use frame_system::{pallet_prelude::OriginFor, Config};

#[cfg(feature = "runtime-benchmarks")]
use crate::settlement::AffirmationRequirement;
use crate::{
    asset::AssetHolder, portfolio::Fund, settlement::AssetCount, settlement::InstructionId,
    IdentityId, WeightMeter,
};

/// Trait for querying affirmation settings stored in the settlement pallet.
pub trait AffirmationFnTrait {
    /// Returns `true` if the given identity has opted in to mandatory receiver affirmation.
    fn identity_requires_affirmation(did: &IdentityId) -> bool;

    /// Sets the mandatory receiver affirmation requirement for benchmarks.
    #[cfg(feature = "runtime-benchmarks")]
    fn set_mandatory_receiver_affirmation(did: IdentityId, requirement: AffirmationRequirement);
}

/// Supertrait config for pallets that need affirmation queries.
pub trait AffirmationFnConfig: frame_system::Config {
    type AffirmationFn: AffirmationFnTrait;
}

/// Trait defining settlement functions for transferring assets.
pub trait SettlementFnTrait<T: Config> {
    /// Receiver affirms the transfer of fungible or non-fungible assets and attempts to execute it.
    fn receiver_affirm_transfer_and_try_execute(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResultWithPostInfo;

    /// Reject a transfer instruction.
    fn reject_transfer(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        asset_count: AssetCount,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo;

    /// Get the try execute weight based on the asset count.
    fn try_execute_weight(asset_count: AssetCount) -> Weight;

    /// Get the receiver affirm transfer weight based on the asset count.
    fn receiver_affirm_transfer_weight(asset_count: AssetCount) -> Weight;

    /// Get the receiver affirm transfer and try execute weight based on the asset count.
    fn receiver_affirm_transfer_and_try_execute_weight_meter(
        base: Weight,
        asset_count: AssetCount,
    ) -> WeightMeter {
        let minimum_charge =
            Self::receiver_affirm_transfer_weight(asset_count).saturating_add(base);
        let limit = minimum_charge.saturating_add(Self::try_execute_weight(asset_count));
        WeightMeter::from_limit_unchecked(minimum_charge, limit)
    }

    /// Get the reject transfer weight meter.
    fn reject_transfer_weight_meter(asset_count: AssetCount) -> WeightMeter;

    /// Worst-case weight limit for `transfer_funds`.
    ///
    /// Includes base benchmark + execute/compliance + spender allowance (only when
    /// `from` is `Some(Account)`) + receiver affirmation for the spender-is-receiver path.
    fn transfer_funds_weight_limit(from: Option<&AssetHolder>, fund: &Fund) -> Weight;

    /// Routes a transfer: same-identity direct, cross-identity settlement.
    /// Returns the settlement instruction ID (None for same-identity).
    fn transfer_funds(
        origin: OriginFor<T>,
        from: Option<AssetHolder>,
        to: AssetHolder,
        fund: Fund,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchError>;
}
