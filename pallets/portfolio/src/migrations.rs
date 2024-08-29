use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v2 {
    use super::*;
    use polymesh_primitives::Ticker;

    decl_storage! {
        trait Store for Module<T: Config> as Portfolio {
            // This storage changed the Ticker key to AssetID.
            pub OldPortfolioAssetBalances get(fn portfolio_asset_balances):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

            // This storage changed the Ticker key to AssetID.
            pub OldPortfolioLockedAssets get(fn locked_assets):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) Ticker => Balance;

            // This storage changed the Ticker key to AssetID.
            pub OldPortfolioNFT get(fn portfolio_nft):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) (Ticker, NFTId) => bool;

            // This storage changed the Ticker key to AssetID.
            pub OldPortfolioLockedNFT get(fn portfolio_locked_nft):
                double_map hasher(twox_64_concat) PortfolioId, hasher(blake2_128_concat) (Ticker, NFTId) => bool;

            // This storage changed the Ticker key to AssetID.
            pub OldPreApprovedPortfolios get(fn pre_approved_portfolios):
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

    let mut count = 0;
    log::info!("Updating types for the PortfolioAssetBalances storage");
    move_prefix(
        &PortfolioAssetBalances::final_prefix(),
        &v2::OldPortfolioAssetBalances::final_prefix(),
    );
    v2::OldPortfolioAssetBalances::drain().for_each(|(portfolio, ticker, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioAssetBalances::insert(portfolio, asset_id, balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the PortfolioLockedAssets storage");
    move_prefix(
        &PortfolioLockedAssets::final_prefix(),
        &v2::OldPortfolioLockedAssets::final_prefix(),
    );
    v2::OldPortfolioLockedAssets::drain().for_each(|(portfolio, ticker, balance)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioLockedAssets::insert(portfolio, asset_id, balance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the PortfolioNFT storage");
    move_prefix(
        &PortfolioNFT::final_prefix(),
        &v2::OldPortfolioNFT::final_prefix(),
    );
    v2::OldPortfolioNFT::drain().for_each(|(portfolio, (ticker, nft_id), v)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioNFT::insert(portfolio, (asset_id, nft_id), v);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the PortfolioLockedNFT storage");
    move_prefix(
        &PortfolioLockedNFT::final_prefix(),
        &v2::OldPortfolioLockedNFT::final_prefix(),
    );
    v2::OldPortfolioLockedNFT::drain().for_each(|(portfolio, (ticker, nft_id), v)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PortfolioLockedNFT::insert(portfolio, (asset_id, nft_id), v);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the PreApprovedPortfolios storage");
    move_prefix(
        &PortfolioLockedNFT::final_prefix(),
        &v2::OldPortfolioLockedNFT::final_prefix(),
    );
    v2::OldPreApprovedPortfolios::drain().for_each(|(portfolio, ticker, v)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        PreApprovedPortfolios::insert(portfolio, asset_id, v);
    });
    log::info!("{:?} items migrated", count);
}
