use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;

impl pallet_nft::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:0)
    // Storage: Asset AssetMetadataLocalKeyToName (r:1 w:0)
    // Storage: NFT NextCollectionId (r:1 w:1)
    // Storage: NFT Collection (r:0 w:1)
    // Storage: NFT CollectionKeys (r:0 w:1)
    fn create_nft_collection(n: u32) -> Weight {
        (207_235_000 as Weight)
            .saturating_add(DbWeight::get().reads(3 as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_mul(n as Weight)
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }

    // Storage: NFT Collection (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: NFT CollectionKeys (r:1 w:0)
    // Storage: NFT NextNFTId (r:1 w:1)
    // Storage: Portfolio PortfolioNFT (r:0 w:1)
    // Storage: NFT MetadataValue (r:0 w:1)
    fn mint_nft(n: u32) -> Weight {
        (207_235_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(2 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_mul(n as Weight)
    }

    fn burn_nft() -> Weight {
        unimplemented!()
    }
}
