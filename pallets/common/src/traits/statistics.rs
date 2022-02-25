use crate::asset::AssetFnTrait;
use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::{
    statistics::{AssetScope, StatType, StatUpdate},
    transfer_compliance::{TransferCondition, TransferConditionExemptKey},
    IdentityId, ScopeId,
};
use sp_std::vec::Vec;

/// The main trait for statistics module
pub trait Config:
    frame_system::Config + crate::traits::identity::Config + crate::traits::external_agents::Config
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
    /// Asset module.
    type Asset: AssetFnTrait<Self::AccountId, Self::Origin>;
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
}

decl_event!(
    pub enum Event {
        /// Stat types added to asset.
        StatTypesAdded(IdentityId, AssetScope, Vec<StatType>),
        /// Stat types removed from asset.
        StatTypesRemoved(IdentityId, AssetScope, Vec<StatType>),
        /// Asset stats updated.
        AssetStatsUpdated(IdentityId, AssetScope, StatType, Vec<StatUpdate>),
        /// Set Transfer compliance rules for asset.
        SetAssetTransferCompliance(IdentityId, AssetScope, Vec<TransferCondition>),
        /// Add `ScopeId`s exempt for transfer conditions matching exempt key.
        TransferConditionExemptionsAdded(IdentityId, TransferConditionExemptKey, Vec<ScopeId>),
        /// Remove `ScopeId`s exempt for transfer conditions matching exempt key.
        TransferConditionExemptionsRemoved(IdentityId, TransferConditionExemptKey, Vec<ScopeId>),
    }
);
