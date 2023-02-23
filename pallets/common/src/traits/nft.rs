use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::nft::{NFTCollectionId, NFTId};
use polymesh_primitives::ticker::Ticker;
use polymesh_primitives::IdentityId;

use crate::compliance_manager::Config as ComplianceManagerConfig;
use crate::{asset, base, identity, portfolio};

pub trait Config:
    frame_system::Config + base::Config + asset::Config + identity::Config + portfolio::Config
{
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;

    type Compliance: ComplianceManagerConfig;

    type MaxNumberOfCollectionKeys: Get<u8>;
}

decl_event!(
    pub enum Event {
        /// Emitted when a new nft collection is created.
        NftCollectionCreated(IdentityId, Ticker, NFTCollectionId),
        /// Emitted when a new nft is issued.
        IssuedNFT(IdentityId, NFTCollectionId, NFTId),
        /// Emitted when an NFT is redeemed.
        RedeemedNFT(IdentityId, Ticker, NFTId),
    }
);

pub trait WeightInfo {
    fn create_nft_collection(n: u32) -> Weight;
    fn issue_nft(n: u32) -> Weight;
    fn redeem_nft(n: u32) -> Weight;
}
