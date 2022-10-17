use frame_support::decl_event;
use frame_support::weights::Weight;

pub trait Config: frame_system::Config {
    type Event: From<Event> + Into<<Self as frame_system::Config>::Event>;
    type WeightInfo: WeightInfo;
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
