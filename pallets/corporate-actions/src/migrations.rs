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
            pub DefaultTargetIdentities get(fn default_target_identities):
                map hasher(blake2_128_concat) Ticker => TargetIdentities;

            // This storage changed the Ticker key to AssetID.
            pub DefaultWithholdingTax get(fn default_withholding_tax):
                map hasher(blake2_128_concat) Ticker => Tax;

            // This storage changed the Ticker key to AssetID.
            pub DidWithholdingTax get(fn did_withholding_tax):
                map hasher(blake2_128_concat) Ticker => Vec<(IdentityId, Tax)>;

            // This storage changed the Ticker key to AssetID.
            pub CAIdSequence get(fn ca_id_sequence):
                map hasher(blake2_128_concat) Ticker => LocalCAId;

            // This storage changed the Ticker key to AssetID.
            pub CorporateActions get(fn corporate_actions):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) LocalCAId => Option<CorporateAction>;

            // The CAId type has been updated.
            pub CADocLink get(fn ca_doc_link):
                map hasher(blake2_128_concat) CAId => Vec<DocumentId>;

            // The CAId type has been updated.
            pub Details get(fn details):
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

    log::info!("Updating types for the DefaultTargetIdentities storage");
    v0::DefaultTargetIdentities::drain().for_each(|(ticker, target_identities)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DefaultTargetIdentities::insert(asset_id, target_identities);
    });

    log::info!("Updating types for the DefaultWithholdingTax storage");
    v0::DefaultWithholdingTax::drain().for_each(|(ticker, tax)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DefaultWithholdingTax::insert(asset_id, tax);
    });

    log::info!("Updating types for the DidWithholdingTax storage");
    v0::DidWithholdingTax::drain().for_each(|(ticker, id_tax)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        DidWithholdingTax::insert(asset_id, id_tax);
    });

    log::info!("Updating types for the CAIdSequence storage");
    v0::CAIdSequence::drain().for_each(|(ticker, id_tax)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CAIdSequence::insert(asset_id, id_tax);
    });

    log::info!("Updating types for the CorporateActions storage");
    v0::CorporateActions::drain().for_each(|(ticker, local_id, ca)| {
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        CorporateActions::insert(asset_id, local_id, ca);
    });

    log::info!("Updating types for the CADocLink storage");
    v0::CADocLink::drain().for_each(|(ca_id, docs)| {
        CADocLink::insert(CAId::from(ca_id), docs);
    });

    log::info!("Updating types for the Details storage");
    v0::Details::drain().for_each(|(ca_id, details)| {
        Details::insert(CAId::from(ca_id), details);
    });
}
