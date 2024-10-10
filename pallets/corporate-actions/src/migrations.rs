use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

pub(crate) mod v0 {
    use super::*;
    use polymesh_primitives::Ticker;

    #[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
    pub struct CAId {
        pub ticker: Ticker,
        pub local_id: LocalCAId,
    }

    decl_storage! {
        trait Store for Module<T: Config> as CorporateAction {
            // This storage changed the Ticker key to AssetID.
            pub OldDefaultTargetIdentities get(fn default_target_identities):
                map hasher(blake2_128_concat) Ticker => TargetIdentities;

            // This storage changed the Ticker key to AssetID.
            pub OldDefaultWithholdingTax get(fn default_withholding_tax):
                map hasher(blake2_128_concat) Ticker => Tax;

            // This storage changed the Ticker key to AssetID.
            pub OldDidWithholdingTax get(fn did_withholding_tax):
                map hasher(blake2_128_concat) Ticker => Vec<(IdentityId, Tax)>;

            // This storage changed the Ticker key to AssetID.
            pub OldCAIdSequence get(fn ca_id_sequence):
                map hasher(blake2_128_concat) Ticker => LocalCAId;

            // This storage changed the Ticker key to AssetID.
            pub OldCorporateActions get(fn corporate_actions):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) LocalCAId => Option<CorporateAction>;

            // The CAId type has been updated.
            pub OldCADocLink get(fn ca_doc_link):
                map hasher(blake2_128_concat) CAId => Vec<DocumentId>;

            // The CAId type has been updated.
            pub OldDetails get(fn details):
                map hasher(blake2_128_concat) CAId => CADetails;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v0::CAId> for CAId {
    fn from(v0_ca_id: v0::CAId) -> Self {
        Self {
            asset_id: AssetID::from(v0_ca_id.ticker),
            local_id: v0_ca_id.local_id,
        }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    let mut count = 0;
    log::info!("Updating types for the DefaultTargetIdentities storage");
    move_prefix(
        &DefaultTargetIdentities::final_prefix(),
        &v0::OldDefaultTargetIdentities::final_prefix(),
    );
    v0::OldDefaultTargetIdentities::drain().for_each(|(ticker, target_identities)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DefaultTargetIdentities::insert(asset_id, target_identities);
    });
    log::info!("Migrated {:?} CA.DefaultTargetIdentities entries.", count);

    let mut count = 0;
    log::info!("Updating types for the DefaultWithholdingTax storage");
    move_prefix(
        &DefaultWithholdingTax::final_prefix(),
        &v0::OldDefaultWithholdingTax::final_prefix(),
    );
    v0::OldDefaultWithholdingTax::drain().for_each(|(ticker, tax)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DefaultWithholdingTax::insert(asset_id, tax);
    });
    log::info!("Migrated {:?} CA.DefaultWithholdingTax entries.", count);

    let mut count = 0;
    log::info!("Updating types for the DidWithholdingTax storage");
    move_prefix(
        &DidWithholdingTax::final_prefix(),
        &v0::OldDidWithholdingTax::final_prefix(),
    );
    v0::OldDidWithholdingTax::drain().for_each(|(ticker, id_tax)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DidWithholdingTax::insert(asset_id, id_tax);
    });
    log::info!("Migrated {:?} CA.DidWithholdingTax entries.", count);

    let mut count = 0;
    log::info!("Updating types for the CAIdSequence storage");
    move_prefix(
        &CAIdSequence::final_prefix(),
        &v0::OldCAIdSequence::final_prefix(),
    );
    v0::OldCAIdSequence::drain().for_each(|(ticker, id_tax)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CAIdSequence::insert(asset_id, id_tax);
    });
    log::info!("Migrated {:?} CA.CAIdSequence entries.", count);

    let mut count = 0;
    log::info!("Updating types for the CorporateActions storage");
    move_prefix(
        &CorporateActions::final_prefix(),
        &v0::OldCorporateActions::final_prefix(),
    );
    v0::OldCorporateActions::drain().for_each(|(ticker, local_id, ca)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CorporateActions::insert(asset_id, local_id, ca);
    });
    log::info!("Migrated {:?} CA.CorporateActions entries.", count);

    let mut count = 0;
    log::info!("Updating types for the CADocLink storage");
    move_prefix(
        &CADocLink::final_prefix(),
        &v0::OldCADocLink::final_prefix(),
    );
    v0::OldCADocLink::drain().for_each(|(ca_id, docs)| {
        count += 1;
        CADocLink::insert(CAId::from(ca_id), docs);
    });
    log::info!("Migrated {:?} CA.CADocLink entries.", count);

    let mut count = 0;
    log::info!("Updating types for the Details storage");
    move_prefix(&Details::final_prefix(), &v0::OldDetails::final_prefix());
    v0::OldDetails::drain().for_each(|(ca_id, details)| {
        count += 1;
        Details::insert(CAId::from(ca_id), details);
    });
    log::info!("Migrated {:?} CA.Details entries.", count);
}
