use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v0 {
    use super::*;
    use polymesh_primitives::Ticker;

    decl_storage! {
        trait Store for Module<T: Config> as Sto {
            // This storage changed the Ticker key to AssetID.
            pub(crate) Fundraisers get(fn fundraisers):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) FundraiserId => Option<Fundraiser<T::Moment>>;

            // This storage changed the Ticker key to AssetID.
            pub(crate) FundraiserCount get(fn fundraiser_count):
                map hasher(blake2_128_concat) Ticker => FundraiserId;

            // This storage changed the Ticker key to AssetID.
            pub(crate) FundraiserNames get(fn fundraiser_name):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) FundraiserId => Option<FundraiserName>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    log::info!("Updating types for the Fundraisers storage");
    v0::Fundraisers::<T>::drain().for_each(|(ticker, id, fundraiser)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        Fundraisers::<T>::insert(asset_id, id, fundraiser);
    });

    log::info!("Updating types for the FundraiserCount storage");
    v0::FundraiserCount::drain().for_each(|(ticker, id)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        FundraiserCount::insert(asset_id, id);
    });

    log::info!("Updating types for the FundraiserNames storage");
    v0::FundraiserNames::drain().for_each(|(ticker, id, name)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        FundraiserNames::insert(asset_id, id, name);
    });
}
