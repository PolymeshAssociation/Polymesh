use frame_support::dispatch::{DispatchErrorWithPostInfo, DispatchResultWithPostInfo};
use frame_support::weights::Weight;
use frame_system::{pallet_prelude::OriginFor, Config};

use crate::{asset::AssetId, settlement::InstructionId, Balance, IdentityId, Memo, NFTId, WeightMeter};

/// Trait for querying affirmation settings stored in the settlement pallet.
pub trait AffirmationFnTrait {
    /// Returns `true` if the given identity has opted in to mandatory receiver affirmation.
    fn identity_requires_affirmation(did: &IdentityId) -> bool;

    /// Sets the mandatory receiver affirmation flag for benchmarks.
    #[cfg(feature = "runtime-benchmarks")]
    fn set_mandatory_receiver_affirmation(did: IdentityId, value: bool);
}

/// Supertrait config for pallets that need affirmation queries.
pub trait AffirmationFnConfig: frame_system::Config {
    type AffirmationFn: AffirmationFnTrait;
}

/// Enum representing either a fungible asset or a non-fungible token.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum AssetOrNft {
    Asset { asset_id: AssetId, amount: Balance },
    Nft { asset_id: AssetId, nft_id: NFTId },
}

/// Trait defining settlement functions for transferring assets.
pub trait SettlementFnTrait<T: Config> {
    /// Creates a transfer instruction for a fungible asset and attempts to execute it (if no pending affirmations).
    fn transfer_asset_and_try_execute(
        origin: OriginFor<T>,
        to: T::AccountId,
        asset_id: AssetId,
        amount: Balance,
        memo: Option<Memo>,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchErrorWithPostInfo> {
        Self::transfer_and_try_execute(
            origin,
            to,
            AssetOrNft::Asset { asset_id, amount },
            memo,
            weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )
    }

    /// Creates a transfer instruction for a non-fungible asset and attempts to execute it (if no pending affirmations).
    fn transfer_nft_and_try_execute(
        origin: OriginFor<T>,
        to: T::AccountId,
        asset_id: AssetId,
        nft_id: NFTId,
        memo: Option<Memo>,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchErrorWithPostInfo> {
        Self::transfer_and_try_execute(
            origin,
            to,
            AssetOrNft::Nft { asset_id, nft_id },
            memo,
            weight_meter,
            #[cfg(feature = "runtime-benchmarks")]
            bench_base_weight,
        )
    }

    /// Creates a transfer instruction for fungible or non-fungible assets and attempts to execute it (if no pending affirmations).
    fn transfer_and_try_execute(
        origin: OriginFor<T>,
        to: T::AccountId,
        asset_or_nft: AssetOrNft,
        memo: Option<Memo>,
        weight_meter: &mut WeightMeter,
        #[cfg(feature = "runtime-benchmarks")] bench_base_weight: bool,
    ) -> Result<Option<InstructionId>, DispatchErrorWithPostInfo>;

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

    /// Get the transfer weight based on the type of asset.
    fn transfer_weight(is_fungible: bool) -> Weight;

    /// Get the try execute weight based on the type of asset.
    fn try_execute_weight(is_fungible: bool) -> Weight;

    /// Get the transfer and try execute weight based on the type of asset.
    fn transfer_and_try_execute_weight_meter(base: Weight, is_fungible: bool) -> WeightMeter {
        let minimum_charge = Self::transfer_weight(is_fungible).saturating_add(base);
        let limit = minimum_charge.saturating_add(Self::try_execute_weight(is_fungible));
        WeightMeter::from_limit_unchecked(minimum_charge, limit)
    }

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
}
