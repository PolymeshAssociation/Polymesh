use polymesh_runtime_common::{RocksDbWeight as DbWeight, Weight};

pub struct WeightInfo;

impl pallet_nft::WeightInfo for WeightInfo {
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: NFT CollectionTicker (r:1 w:1)
    // Storage: Asset AssetMetadataGlobalKeyToName (r:1 w:0)
    // Storage: Asset Tokens (r:1 w:1)
    // Storage: Asset TickerConfig (r:1 w:0)
    // Storage: Asset Tickers (r:1 w:1)
    // Storage: Identity DidRecords (r:1 w:1)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: ProtocolFee Coefficient (r:1 w:0)
    // Storage: ProtocolFee BaseFees (r:2 w:0)
    // Storage: Identity CurrentPayer (r:1 w:0)
    // Storage: ExternalAgents NumFullAgents (r:1 w:1)
    // Storage: NFT NextCollectionId (r:1 w:1)
    // Storage: NFT Collection (r:0 w:1)
    // Storage: NFT CollectionKeys (r:0 w:1)
    // Storage: Asset FundingRound (r:0 w:1)
    // Storage: Asset AssetOwnershipRelations (r:0 w:1)
    // Storage: Asset AssetNames (r:0 w:1)
    // Storage: Asset ClassicTickers (r:0 w:1)
    // Storage: Asset DisableInvestorUniqueness (r:0 w:1)
    // Storage: Asset Identifiers (r:0 w:1)
    // Storage: ExternalAgents AgentOf (r:0 w:1)
    // Storage: ExternalAgents GroupOfAgent (r:0 w:1)
    fn create_nft_collection(n: u32) -> Weight {
        (207_235_000 as Weight)
            .saturating_add(DbWeight::get().writes(16 as Weight))
            .saturating_add(DbWeight::get().reads(13 as Weight))
            .saturating_add(DbWeight::get().reads(1 as Weight))
            .saturating_mul(n as Weight)
    }

    // Storage: NFT CollectionTicker (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: NFT CollectionKeys (r:1 w:0)
    // Storage: NFT NextNFTId (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:1)
    // Storage: Portfolio PortfolioNFT (r:0 w:1)
    // Storage: NFT MetadataValue (r:0 w:1)
    fn mint_nft(n: u32) -> Weight {
        (207_235_000 as Weight)
            .saturating_add(DbWeight::get().reads(9 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
            .saturating_add(DbWeight::get().writes(1 as Weight))
            .saturating_mul(n as Weight)
    }

    // Storage: NFT CollectionTicker (r:1 w:0)
    // Storage: Identity KeyRecords (r:1 w:0)
    // Storage: ExternalAgents GroupOfAgent (r:1 w:0)
    // Storage: Permissions CurrentPalletName (r:1 w:0)
    // Storage: Permissions CurrentDispatchableName (r:1 w:0)
    // Storage: Portfolio PortfolioCustodian (r:1 w:0)
    // Storage: Portfolio PortfolioNFT (r:1 w:1)
    // Storage: Asset BalanceOf (r:1 w:1)
    // Storage: NFT MetadataValue (r:0 w:1)
    fn burn_nft() -> Weight {
        (207_235_000 as Weight)
            .saturating_add(DbWeight::get().reads(8 as Weight))
            .saturating_add(DbWeight::get().writes(3 as Weight))
    }
}
