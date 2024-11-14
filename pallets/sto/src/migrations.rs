use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v0 {
    use super::*;
    use polymesh_primitives::Ticker;

    #[derive(Clone, Decode, Default, Encode, PartialEq, TypeInfo)]
    pub struct Fundraiser<Moment> {
        pub creator: IdentityId,
        pub offering_portfolio: PortfolioId,
        pub offering_asset: Ticker,
        pub raising_portfolio: PortfolioId,
        pub raising_asset: Ticker,
        pub tiers: Vec<FundraiserTier>,
        pub venue_id: VenueId,
        pub start: Moment,
        pub end: Option<Moment>,
        pub status: FundraiserStatus,
        pub minimum_investment: Balance,
    }

    decl_storage! {
        trait Store for Module<T: Config> as Sto {
            // This storage changed the Ticker key to AssetId.
            pub(crate) OldFundraisers get(fn fundraisers):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) FundraiserId => Option<Fundraiser<T::Moment>>;

            // This storage changed the Ticker key to AssetId.
            pub(crate) OldFundraiserCount get(fn fundraiser_count):
                map hasher(blake2_128_concat) Ticker => FundraiserId;

            // This storage changed the Ticker key to AssetId.
            pub(crate) OldFundraiserNames get(fn fundraiser_name):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) FundraiserId => Option<FundraiserName>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl<T> From<v0::Fundraiser<T>> for Fundraiser<T> {
    fn from(v0_fundraiser: v0::Fundraiser<T>) -> Fundraiser<T> {
        Fundraiser {
            creator: v0_fundraiser.creator,
            offering_portfolio: v0_fundraiser.offering_portfolio,
            offering_asset: AssetId::from(v0_fundraiser.offering_asset),
            raising_portfolio: v0_fundraiser.raising_portfolio,
            raising_asset: AssetId::from(v0_fundraiser.raising_asset),
            tiers: v0_fundraiser.tiers,
            venue_id: v0_fundraiser.venue_id,
            start: v0_fundraiser.start,
            end: v0_fundraiser.end,
            status: v0_fundraiser.status,
            minimum_investment: v0_fundraiser.minimum_investment,
        }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    let mut count = 0;
    log::info!("Updating types for the Fundraisers storage");
    move_prefix(
        &Fundraisers::<T>::final_prefix(),
        &v0::OldFundraisers::<T>::final_prefix(),
    );
    v0::OldFundraisers::<T>::drain().for_each(|(ticker, id, fundraiser)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        Fundraisers::<T>::insert(asset_id, id, Fundraiser::from(fundraiser));
    });
    log::info!("Migrated {:?} Sto.Fundraiser entries.", count);

    let mut count = 0;
    log::info!("Updating types for the FundraiserCount storage");
    move_prefix(
        &FundraiserCount::final_prefix(),
        &v0::OldFundraiserCount::final_prefix(),
    );
    v0::OldFundraiserCount::drain().for_each(|(ticker, id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        FundraiserCount::insert(asset_id, id);
    });
    log::info!("Migrated {:?} Sto.FundraiserCount entries.", count);

    let mut count = 0;
    log::info!("Updating types for the FundraiserNames storage");
    move_prefix(
        &FundraiserNames::final_prefix(),
        &v0::OldFundraiserNames::final_prefix(),
    );
    v0::OldFundraiserNames::drain().for_each(|(ticker, id, name)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetId::from(ticker));
        FundraiserNames::insert(asset_id, id, name);
    });
    log::info!("Migrated {:?} Sto.FundraiserNames entries.", count);
}
