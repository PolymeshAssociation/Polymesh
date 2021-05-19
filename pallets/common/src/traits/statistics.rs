use crate::asset::AssetFnTrait;
use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::statistics::TransferManager;
use polymesh_primitives::{IdentityId, ScopeId, Ticker};
use sp_std::vec::Vec;

/// The main trait for statistics module
pub trait Trait:
    frame_system::Config + crate::traits::identity::Trait + crate::traits::external_agents::Trait
{
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
    /// Asset module
    type Asset: AssetFnTrait<Self::Balance, Self::AccountId, Self::Origin>;
    /// Maximum transfer managers that can be enabled for an Asset
    type MaxTransferManagersPerAsset: Get<u32>;
    /// Weights for extrinsics
    type WeightInfo: WeightInfo;
}

/// Weight info for extrinsics
pub trait WeightInfo {
    fn add_transfer_manager() -> Weight;
    fn remove_transfer_manager() -> Weight;
    fn add_exempted_entities(i: u32) -> Weight;
    fn remove_exempted_entities(i: u32) -> Weight;
}

decl_event!(
    pub enum Event {
        /// A new transfer manager was added.
        TransferManagerAdded(IdentityId, Ticker, TransferManager),
        /// An existing transfer manager was removed.
        TransferManagerRemoved(IdentityId, Ticker, TransferManager),
        /// `ScopeId`s were added to the exemption list.
        ExemptionsAdded(IdentityId, Ticker, TransferManager, Vec<ScopeId>),
        /// `ScopeId`s were removed from the exemption list.
        ExemptionsRemoved(IdentityId, Ticker, TransferManager, Vec<ScopeId>),
    }
);
