use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::nft::{NFTCollectionId, NFTs};
use polymesh_primitives::traits::ComplianceFnConfig;
use polymesh_primitives::{IdentityId, PortfolioId, PortfolioUpdateReason};

use crate::{asset, portfolio};

pub trait Config:
    frame_system::Config + asset::Config + pallet_identity::Config + portfolio::Config
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
