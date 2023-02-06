use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::asset_metadata::AssetMetadataKey;
use polymesh_primitives::nft::{NFTCollectionId, NFTId};
use polymesh_primitives::ticker::Ticker;
use polymesh_primitives::IdentityId;

use crate::compliance_manager::Config as ComplianceManagerConfig;
use crate::{asset, base, identity};

pub trait Config: frame_system::Config + base::Config + asset::Config + identity::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;

    type Compliance: ComplianceManagerConfig;

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
    fn burn_nft(n: u32) -> Weight;
}

pub trait NFTTrait {
    /// Returns true if the given `metadata_key` is a mandatory key for the ticker's NFT collection.
    fn is_collection_key(ticker: &Ticker, metadata_key: &AssetMetadataKey) -> bool;
}
