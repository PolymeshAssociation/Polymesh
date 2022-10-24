use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::nft::{NFTCollectionId, NFTId};
use polymesh_primitives::ticker::Ticker;

use crate::asset::AssetFnTrait;
use crate::base;

pub trait Config: frame_system::Config + base::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;

    type MaxNumberOfCollectionKeys: Get<u8>;

    type Asset: AssetFnTrait<Self::AccountId, Self::Origin>;
}

decl_event!(
    pub enum Event {
        /// Emitted when a new nft collection is created
        NftCollectionCreated(Ticker, NFTCollectionId),
        /// Emitted when a new nft is created
        MintedNft(NFTCollectionId, NFTId),
    }
);

pub trait WeightInfo {
    fn create_nft_collection() -> Weight;
    fn mint_nft() -> Weight;
}
