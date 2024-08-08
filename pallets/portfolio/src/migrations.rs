use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v2 {
    use super::*;
    use polymesh_primitives::Ticker;

    decl_storage! {
        trait Store for Module<T: Config> as Portfolio {
            // This storage changed the Ticker key to AssetID.
            pub PortfolioAssetBalances get(fn portfolio_asset_balances):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

            // This storage changed the Ticker key to AssetID.
            pub PortfolioLockedAssets get(fn locked_assets):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

            // This storage changed the Ticker key to AssetID.
            pub PortfolioNFT get(fn portfolio_nft):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) (Ticker, NFTId) => bool;

            // This storage changed the Ticker key to AssetID.
            pub PortfolioLockedNFT get(fn portfolio_locked_nft):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) (Ticker, NFTId) => bool;

            // This storage changed the Ticker key to AssetID.
            pub PreApprovedPortfolios get(fn pre_approved_portfolios):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => bool;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v3<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    log::info!("Updating types for the PortfolioAssetBalances storage");
    v2::PortfolioAssetBalances::drain().for_each(|(portfolio, ticker, balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioAssetBalances::insert(portfolio, asset_id, balance);
    });

    log::info!("Updating types for the PortfolioLockedAssets storage");
    v2::PortfolioLockedAssets::drain().for_each(|(portfolio, ticker, balance)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioLockedAssets::insert(portfolio, asset_id, balance);
    });

    log::info!("Updating types for the PortfolioNFT storage");
    v2::PortfolioNFT::drain().for_each(|(portfolio, (ticker, nft_id), v)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioNFT::insert(portfolio, (asset_id, nft_id), v);
    });

    log::info!("Updating types for the PortfolioLockedNFT storage");
    v2::PortfolioLockedNFT::drain().for_each(|(portfolio, (ticker, nft_id), v)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioLockedNFT::insert(portfolio, (asset_id, nft_id), v);
    });

    log::info!("Updating types for the PreApprovedPortfolios storage");
    v2::PreApprovedPortfolios::drain().for_each(|(portfolio, ticker, v)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PreApprovedPortfolios::insert(portfolio, asset_id, v);
    });
}
