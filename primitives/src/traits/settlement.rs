use frame_support::dispatch::DispatchResultWithPostInfo;
use frame_support::pallet_prelude::DispatchError;
use frame_support::weights::Weight;
use frame_system::{pallet_prelude::OriginFor, Config};

#[cfg(feature = "runtime-benchmarks")]
use crate::settlement::AffirmationRequirement;
use crate::{
    asset::AssetHolder, portfolio::Fund, settlement::InstructionId, IdentityId, WeightMeter,
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
        is_fungible: bool,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> DispatchResultWithPostInfo;

    /// Reject a transfer instruction.
    fn reject_transfer(
        origin: OriginFor<T>,
        instruction_id: InstructionId,
        is_fungible: bool,
        weight_meter: &mut WeightMeter,
    ) -> DispatchResultWithPostInfo;

    /// Get the try execute weight based on the type of asset.
    fn try_execute_weight(is_fungible: bool) -> Weight;

    /// Get the receiver affirm transfer weight based on the type of asset.
    fn receiver_affirm_transfer_weight(is_fungible: bool) -> Weight;

    /// Get the receiver affirm transfer and try execute weight based on the type of asset.
    fn receiver_affirm_transfer_and_try_execute_weight_meter(
        base: Weight,
        is_fungible: bool,
    ) -> WeightMeter {
        let minimum_charge =
            Self::receiver_affirm_transfer_weight(is_fungible).saturating_add(base);
        let limit = minimum_charge.saturating_add(Self::try_execute_weight(is_fungible));
        WeightMeter::from_limit_unchecked(minimum_charge, limit)
    }

    /// Get the reject transfer weight meter.
    fn reject_transfer_weight_meter(is_fungible: bool) -> WeightMeter;

    /// Get the worst-case weight for `transfer_funds` / `base_transfer_funds`.
    fn transfer_funds_weight() -> Weight;

    /// Worst-case weight for `transfer_funds` when sender and receiver are accounts (no portfolios).
    /// Used by `transfer_asset` where both ends are always `AccountId`.
    fn transfer_funds_account_weight() -> Weight;

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
