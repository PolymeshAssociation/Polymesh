#[cfg(feature = "runtime-benchmarks")]
use frame_support::dispatch::DispatchResult;
#[cfg(feature = "runtime-benchmarks")]
use polymesh_primitives::asset::NonFungibleType;
#[cfg(feature = "runtime-benchmarks")]
use polymesh_primitives::nft::NFTCollectionKeys;

use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::asset_metadata::AssetMetadataKey;
use polymesh_primitives::nft::{NFTCollectionId, NFTs};
use polymesh_primitives::{IdentityId, NFTId, PortfolioId, PortfolioUpdateReason};

use crate::compliance_manager::ComplianceFnConfig;
use crate::{asset, base, identity, portfolio};

pub trait Config:
    frame_system::Config + base::Config + asset::Config + identity::Config + portfolio::Config
{
    type RuntimeEvent: From<Event> + Into<<Self as frame_system::Config>::RuntimeEvent>;

    type WeightInfo: WeightInfo;

    type Compliance: ComplianceFnConfig;

    type MaxNumberOfCollectionKeys: Get<u8>;

    type MaxNumberOfNFTsCount: Get<u32>;
}

decl_event!(
    pub enum Event {
        /// Emitted when a new nft collection is created.
        NftCollectionCreated(IdentityId, AssetId, NFTCollectionId),
        /// Emitted when NFTs were issued, redeemed or transferred.
        /// Contains the [`IdentityId`] of the receiver/issuer/redeemer, the [`NFTs`], the [`PortfolioId`] of the source, the [`PortfolioId`]
        /// of the destination and the [`PortfolioUpdateReason`].
        NFTPortfolioUpdated(
            IdentityId,
            NFTs,
            Option<PortfolioId>,
            Option<PortfolioId>,
            PortfolioUpdateReason,
        ),
    }
);

pub trait WeightInfo {
    fn create_nft_collection(n: u32) -> Weight;
    fn issue_nft(n: u32) -> Weight;
    fn redeem_nft(n: u32) -> Weight;
    fn base_nft_transfer(n: u32) -> Weight;
    fn controller_transfer(n: u32) -> Weight;
}

pub trait NFTTrait<Origin> {
    /// Returns `true` if the given `metadata_key` is a mandatory key for the `asset_id` NFT collection.
    fn is_collection_key(asset_id: &AssetId, metadata_key: &AssetMetadataKey) -> bool;
    /// Updates the NFTOwner storage after moving funds.
    fn move_portfolio_owner(asset_id: AssetId, nft_id: NFTId, new_owner_portfolio: PortfolioId);

    #[cfg(feature = "runtime-benchmarks")]
    fn create_nft_collection(
        origin: Origin,
        asset_id: Option<AssetId>,
        nft_type: Option<NonFungibleType>,
        collection_keys: NFTCollectionKeys,
    ) -> DispatchResult;
}
