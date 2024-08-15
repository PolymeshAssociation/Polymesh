use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v0 {
    use super::*;
    use polymesh_primitives::{v6, Ticker};

    decl_storage! {
        trait Store for Module<T: Config> as ExternalAgents {
            // This storage changed the Ticker key to AssetID.
            pub AGIdSequence get(fn agent_group_id_sequence):
                map hasher(blake2_128_concat) Ticker => AGId;

            // This storage changed the Ticker key to AssetID.
            pub AgentOf get(fn agent_of):
                double_map hasher(blake2_128_concat) IdentityId, hasher(blake2_128_concat) Ticker => ();

            // This storage changed the Ticker key to AssetID.
            pub GroupOfAgent get(fn agents):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) IdentityId => Option<AgentGroup>;

            // This storage changed the Ticker key to AssetID.
            pub NumFullAgents get(fn num_full_agents):
                map hasher(blake2_128_concat) Ticker => u32;

            // This storage changed the Ticker key to AssetID.
            pub GroupPermissions get(fn permissions):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) AGId => Option<v6::ExtrinsicPermissions>;
        }

    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    let mut count = 0;
    log::info!("Updating types for the AGIdSequence storage");
    v0::AGIdSequence::drain().for_each(|(ticker, ag_id)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AGIdSequence::insert(asset_id, ag_id);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AgentOf storage");
    v0::AgentOf::drain().for_each(|(did, ticker, empty)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        AgentOf::insert(did, asset_id, empty);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the GroupOfAgent storage");
    v0::GroupOfAgent::drain().for_each(|(ticker, did, group)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        GroupOfAgent::insert(asset_id, did, group);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the NumFullAgents storage");
    v0::NumFullAgents::drain().for_each(|(ticker, n)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        NumFullAgents::insert(asset_id, n);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the GroupPermissions storage");
    v0::GroupPermissions::drain().for_each(|(ticker, ag_id, ext_perms)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        GroupPermissions::insert(asset_id, ag_id, ExtrinsicPermissions::from(ext_perms));
    });
    log::info!("{:?} items migrated", count);
}
