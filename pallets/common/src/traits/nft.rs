use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::nft::{NFTCollectionId, NFTId};
use polymesh_primitives::ticker::Ticker;
use polymesh_primitives::IdentityId;

use crate::asset;
use crate::base;
use crate::compliance_manager;
use crate::identity;
use crate::portfolio;

pub trait Config:
    frame_system::Config + base::Config + asset::Config + identity::Config + portfolio::Config
{
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;

    type Compliance: compliance_manager::Config;

    type MaxNumberOfCollectionKeys: Get<u8>;
}

decl_event!(
    pub enum Event {
        /// Emitted when a new nft collection is created.
        NftCollectionCreated(IdentityId, Ticker, NFTCollectionId),
        /// Emitted when a new nft is minted.
        MintedNft(IdentityId, NFTCollectionId, NFTId),
        /// Emitted when an NFT is burned.
        BurnedNFT(IdentityId, Ticker, NFTId),
    }
);

pub trait WeightInfo {
    fn create_nft_collection(n: u32) -> Weight;
    fn mint_nft(n: u32) -> Weight;
    fn burn_nft() -> Weight;
}
