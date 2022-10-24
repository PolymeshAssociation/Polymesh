use frame_support::decl_event;
use frame_support::traits::Get;
use frame_support::weights::Weight;

use crate::asset::AssetFnTrait;
use crate::base;

pub trait Config: frame_system::Config + base::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;

    type WeightInfo: WeightInfo;

    type Asset: AssetFnTrait<Self::AccountId, Self::Origin>;

    type MaxNumberOfCollectionKeys: Get<u8>;
}

decl_event!(
    pub enum Event {
        NftCollectionCreated,
        MintedNft,
    }
);

pub trait WeightInfo {
    fn create_nft_collection() -> Weight;
}
