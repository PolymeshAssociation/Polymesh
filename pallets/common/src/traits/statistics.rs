use crate::asset::AssetFnTrait;
use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::{
    statistics::{AssetScope, StatType, StatUpdate},
    transfer_compliance::{TransferCondition, TransferConditionExemptKey},
    IdentityId,
};
use sp_std::vec::Vec;

/// The main trait for statistics module
pub trait Config:
    frame_system::Config + crate::traits::identity::Config + crate::traits::external_agents::Config
{
    /// The overarching event type.
    type RuntimeEvent: From<Event> + Into<<Self as frame_system::Config>::RuntimeEvent>;
    /// Asset module.
    type Asset: AssetFnTrait<Self::AccountId, Self::RuntimeOrigin>;
    /// Maximum stats that can be enabled for an Asset.
    type MaxStatsPerAsset: Get<u32>;
    /// Maximum transfer conditions that can be enabled for an Asset.
    type MaxTransferConditionsPerAsset: Get<u32>;
    /// Weights for extrinsics.
    type WeightInfo: WeightInfo;
}

/// Weight info for extrinsics
pub trait WeightInfo {
    fn set_active_asset_stats(i: u32) -> Weight;
    fn batch_update_asset_stats(i: u32) -> Weight;
    fn set_asset_transfer_compliance(i: u32) -> Weight;
    fn set_entities_exempt(i: u32) -> Weight;
    fn max_investor_count_restriction(a: u32) -> Weight;
    fn max_investor_ownership_restriction() -> Weight;
    fn claim_count_restriction_no_stats(c: u32) -> Weight;
    fn claim_count_restriction_with_stats() -> Weight;
    fn claim_ownership_restriction(a: u32) -> Weight;
    fn update_asset_count_stats(a: u32) -> Weight;
    fn update_asset_balance_stats(a: u32) -> Weight;
    fn active_asset_statistics_load(_a: u32) -> Weight;
    fn is_exempt() -> Weight;
    fn verify_requirements(i: u32) -> Weight;
    fn verify_requirements_loop(i: u32) -> Weight {
        Self::verify_requirements(i)
            .saturating_sub(Self::max_investor_count_restriction(0).saturating_mul(i.into()))
    }
}

decl_event!(
    pub enum Event {
        /// Stat types added to asset.
        ///
        /// (Caller DID, Asset, Stat types)
        StatTypesAdded(IdentityId, AssetScope, Vec<StatType>),
        /// Stat types removed from asset.
        ///
        /// (Caller DID, Asset, Stat types)
        StatTypesRemoved(IdentityId, AssetScope, Vec<StatType>),
        /// Asset stats updated.
        ///
        /// (Caller DID, Asset, Stat type, Updates)
        AssetStatsUpdated(IdentityId, AssetScope, StatType, Vec<StatUpdate>),
        /// Set Transfer compliance rules for asset.
        ///
        /// (Caller DID, Asset, Transfer conditions)
        SetAssetTransferCompliance(IdentityId, AssetScope, Vec<TransferCondition>),
        /// Add `IdentityId`s exempt for transfer conditions matching exempt key.
        ///
        /// (Caller DID, Exempt key, Entities)
        TransferConditionExemptionsAdded(IdentityId, TransferConditionExemptKey, Vec<IdentityId>),
        /// Remove `IdentityId`s exempt for transfer conditions matching exempt key.
        ///
        /// (Caller DID, Exempt key, Entities)
        TransferConditionExemptionsRemoved(IdentityId, TransferConditionExemptKey, Vec<IdentityId>),
    }
);
