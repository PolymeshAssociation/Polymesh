use frame_support::storage::migration::move_prefix;
use frame_support::BoundedBTreeSet;
use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v2 {
    use scale_info::TypeInfo;

    use super::*;
    use polymesh_primitives::{ClaimType, Ticker};

    #[derive(Decode, Encode, TypeInfo)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub enum AssetScope {
        Ticker(Ticker),
    }

    #[derive(Decode, Encode, TypeInfo)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
    pub struct StatType {
        pub op: StatOpType,
        pub claim_issuer: Option<(ClaimType, IdentityId)>,
    }

    #[derive(Decode, Encode, TypeInfo)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct Stat1stKey {
        pub asset: AssetScope,
        pub stat_type: StatType,
    }

    #[derive(Decode, Encode, TypeInfo)]
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub struct TransferConditionExemptKey {
        pub asset: AssetScope,
        pub op: StatOpType,
        pub claim_type: Option<ClaimType>,
    }

    decl_storage! {
        trait Store for Module<T: Config> as Statistics {
            // This storage changed the AssetScope type.
            pub OldActiveAssetStats get(fn active_asset_stats):
                map hasher(blake2_128_concat) AssetScope => BoundedBTreeSet<StatType, T::MaxStatsPerAsset>;

            // This storage changed the Stat1stKey type.
            pub OldAssetStats get(fn asset_stats):
              double_map hasher(blake2_128_concat) Stat1stKey, hasher(blake2_128_concat) Stat2ndKey => u128;

            // This storage changed the AssetScope type.
            pub OldAssetTransferCompliances get(fn asset_transfer_compliance):
                map hasher(blake2_128_concat) AssetScope => AssetTransferCompliance<T::MaxTransferConditionsPerAsset>;

            // This storage changed the TransferConditionExemptKey type.
            pub OldTransferConditionExemptEntities get(fn transfer_condition_exempt_entities):
                double_map hasher(blake2_128_concat) TransferConditionExemptKey, hasher(blake2_128_concat) IdentityId => bool;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v2::AssetScope> for AssetID {
    fn from(v2_asset_scope: v2::AssetScope) -> AssetID {
        match v2_asset_scope {
            v2::AssetScope::Ticker(ticker) => ticker.into(),
        }
    }
}

impl From<v2::StatType> for StatType {
    fn from(v2_stat_type: v2::StatType) -> StatType {
        StatType {
            operation_type: v2_stat_type.op,
            claim_issuer: v2_stat_type.claim_issuer,
        }
    }
}

impl From<v2::Stat1stKey> for Stat1stKey {
    fn from(v2_stat1key: v2::Stat1stKey) -> Stat1stKey {
        Stat1stKey {
            asset_id: v2_stat1key.asset.into(),
            stat_type: v2_stat1key.stat_type.into(),
        }
    }
}

impl From<v2::TransferConditionExemptKey> for TransferConditionExemptKey {
    fn from(v2_exempt_key: v2::TransferConditionExemptKey) -> TransferConditionExemptKey {
        TransferConditionExemptKey {
            asset_id: v2_exempt_key.asset.into(),
            op: v2_exempt_key.op,
            claim_type: v2_exempt_key.claim_type,
        }
    }
}

pub(crate) fn migrate_to_v3<T: Config>() {
    RuntimeLogger::init();

    // Removes all elements in the old storage and inserts it in the new storage

    let mut count = 0;
    log::info!("Updating types for the ActiveAssetStats storage");
    move_prefix(
        &ActiveAssetStats::<T>::final_prefix(),
        &v2::OldActiveAssetStats::<T>::final_prefix(),
    );
    v2::OldActiveAssetStats::<T>::drain().for_each(|(scope, set)| {
        count += 1;
        let set: BTreeSet<StatType> = set.into_iter().map(|v| v.into()).collect();
        let bounded_set = BoundedBTreeSet::try_from(set).unwrap_or_default();
        ActiveAssetStats::<T>::insert(AssetID::from(scope), bounded_set);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetStats storage");
    move_prefix(
        &AssetStats::final_prefix(),
        &v2::OldAssetStats::final_prefix(),
    );
    v2::OldAssetStats::drain().for_each(|(stat1key, stat2key, v)| {
        count += 1;
        AssetStats::insert(Stat1stKey::from(stat1key), stat2key, v);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the AssetTransferCompliances storage");
    move_prefix(
        &AssetTransferCompliances::<T>::final_prefix(),
        &v2::OldAssetTransferCompliances::<T>::final_prefix(),
    );
    v2::OldAssetTransferCompliances::<T>::drain().for_each(|(scope, compliance)| {
        count += 1;
        AssetTransferCompliances::<T>::insert(AssetID::from(scope), compliance);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the TransferConditionExemptEntities storage");
    move_prefix(
        &TransferConditionExemptEntities::final_prefix(),
        &v2::OldTransferConditionExemptEntities::final_prefix(),
    );
    v2::OldTransferConditionExemptEntities::drain().for_each(|(exemption_key, did, exempt)| {
        count += 1;
        TransferConditionExemptEntities::insert(
            TransferConditionExemptKey::from(exemption_key),
            did,
            exempt,
        );
    });
    log::info!("{:?} items migrated", count);
}
