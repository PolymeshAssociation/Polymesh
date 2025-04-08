use frame_benchmarking::benchmarks;
use sp_runtime::Permill;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use pallet_identity::benchmarking::{User, UserBuilder};
use pallet_identity::{Claim1stKey, Claim2ndKey};
use polymesh_primitives::bench::create_and_issue_sample_asset;
use polymesh_primitives::constants::currency::ONE_UNIT;
use polymesh_primitives::jurisdiction::*;
use polymesh_primitives::statistics::*;
use polymesh_primitives::{Claim, ClaimType, IdentityClaim, Scope};

use crate::*;

const STAT_TYPES: &[(StatOpType, Option<ClaimType>)] = &[
    (StatOpType::Count, None),
    (StatOpType::Balance, None),
    (StatOpType::Count, Some(ClaimType::Accredited)),
    (StatOpType::Balance, Some(ClaimType::Accredited)),
    (StatOpType::Count, Some(ClaimType::Affiliate)),
    (StatOpType::Balance, Some(ClaimType::Affiliate)),
    (StatOpType::Count, Some(ClaimType::Jurisdiction)),
    (StatOpType::Balance, Some(ClaimType::Jurisdiction)),
];

fn make_stats(count: u32) -> BTreeSet<StatType> {
    (0..count as usize)
        .into_iter()
        .map(|idx| {
            let (op, claim_type) = STAT_TYPES[idx % STAT_TYPES.len()];
            StatType {
                operation_type: op,
                claim_issuer: claim_type.map(|ct| (ct, IdentityId::from(idx as u128))),
            }
        })
        .collect()
}

fn make_jur_stat_updates(count: u32, value: Option<u128>) -> BTreeSet<StatUpdate> {
    (0..count as usize)
        .into_iter()
        .map(|idx| StatUpdate {
            key2: Stat2ndKey::Claim(StatClaim::Jurisdiction(Some(
                COUNTRY_CODES[idx % COUNTRY_CODES.len()],
            ))),
            value,
        })
        .collect()
}

fn claim_type_to_stat_claim(claim_type: ClaimType) -> Option<StatClaim> {
    match claim_type {
        ClaimType::Accredited => Some(StatClaim::Accredited(true)),
        ClaimType::Affiliate => Some(StatClaim::Affiliate(true)),
        ClaimType::Jurisdiction => Some(StatClaim::Jurisdiction(None)),
        _ => None,
    }
}

fn make_transfer_conditions(stats: &BTreeSet<StatType>, count: u32) -> BTreeSet<TransferCondition> {
    let p0 = sp_arithmetic::Permill::from_rational(0u32, 100u32);
    let p40 = sp_arithmetic::Permill::from_rational(40u32, 100u32);
    (0..count as usize)
        .into_iter()
        .zip(stats.iter())
        .map(
            |(_idx, stat)| match (stat.operation_type, stat.claim_issuer) {
                (StatOpType::Count, None) => TransferCondition::MaxInvestorCount(10),
                (StatOpType::Balance, None) => TransferCondition::MaxInvestorOwnership(p40),
                (StatOpType::Count, Some((claim_type, issuer))) => {
                    let claim =
                        claim_type_to_stat_claim(claim_type).expect("Unsupported ClaimType");
                    TransferCondition::ClaimCount(claim, issuer, 0, Some(10))
                }
                (StatOpType::Balance, Some((claim_type, issuer))) => {
                    let claim =
                        claim_type_to_stat_claim(claim_type).expect("Unsupported ClaimType");
                    TransferCondition::ClaimOwnership(claim, issuer, p0, p40)
                }
            },
        )
        .collect()
}

fn init_asset<T: Config>() -> (User<T>, AssetId) {
    let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
    let asset_id =
        create_and_issue_sample_asset::<T>(owner.account(), true, None, b"MyAsset", true);
    (owner, asset_id)
}

fn init_transfer_conditions<T: Config>(
    count_stats: u32,
    count_conditions: u32,
) -> (
    User<T>,
    AssetId,
    BTreeSet<StatType>,
    BTreeSet<TransferCondition>,
) {
    let (owner, asset_id) = init_asset::<T>();
    let stats = make_stats(count_stats);
    let conditions = make_transfer_conditions(&stats, count_conditions);
    (owner, asset_id, stats, conditions)
}

fn init_exempts<T: Config>(
    count: u32,
) -> (User<T>, TransferConditionExemptKey, BTreeSet<IdentityId>) {
    let (owner, asset_id) = init_asset::<T>();
    let scope_ids = (0..count as u128).map(IdentityId::from).collect();
    let exempt_key = TransferConditionExemptKey {
        asset_id,
        op: StatOpType::Count,
        claim_type: Some(ClaimType::Accredited),
    };
    (owner, exempt_key, scope_ids)
}

/// Exempts `exempt_user_id` to follow a transfer condition of claim type `Accredited` for `ticker`.
pub fn set_transfer_exception<T: Config>(
    origin: T::RuntimeOrigin,
    asset_id: AssetId,
    exempt_user_id: IdentityId,
) {
    let transfer_exception = TransferConditionExemptKey {
        asset_id,
        op: StatOpType::Balance,
        claim_type: Some(ClaimType::Accredited),
    };
    Pallet::<T>::set_entities_exempt(
        origin.clone(),
        true,
        transfer_exception,
        [exempt_user_id].into(),
    )
    .unwrap();
}

/// Adds `claim` issued by `issuer_id` to `id`.
pub fn add_identity_claim<T: Config>(id: IdentityId, claim: Claim, issuer_id: IdentityId) {
    pallet_identity::Pallet::<T>::unverified_add_claim_with_scope(
        id,
        claim.clone(),
        claim.as_scope().cloned(),
        issuer_id,
        None,
    );
}

/// Adds the maximum number of active statistics, adds `n` transfer restrictions and if `pause_restrictions` is true,
/// pauses analyzing the restrictions
pub fn setup_transfer_restrictions<T: Config>(
    origin: T::RuntimeOrigin,
    sender_id: IdentityId,
    asset_id: AssetId,
    n: u32,
    pause_restrictions: bool,
) {
    // Adds the maximum number of active statistics
    let active_stats = (0..10)
        .map(|i| StatType {
            operation_type: StatOpType::Count,
            claim_issuer: Some((ClaimType::Accredited, IdentityId::from(i as u128))),
        })
        .collect();
    Pallet::<T>::set_active_asset_stats(origin.clone(), asset_id, active_stats).unwrap();

    let transfer_conditions: BTreeSet<TransferCondition> = (0..n)
        .map(|i| {
            let issuer_id = IdentityId::from(i as u128);
            add_identity_claim::<T>(
                sender_id,
                Claim::Accredited(Scope::Asset(asset_id)),
                issuer_id,
            );
            TransferCondition::ClaimCount(StatClaim::Accredited(true), issuer_id, 0, Some(1))
        })
        .collect();
    Pallet::<T>::set_asset_transfer_compliance(origin.clone(), asset_id, transfer_conditions)
        .unwrap();
    if pause_restrictions {
        ActiveAssetStats::<T>::remove(&asset_id);
        AssetTransferCompliances::<T>::mutate(asset_id, |atc| {
            atc.paused = true;
        });
        return;
    }
}

#[cfg(feature = "running-ci")]
mod limits {
    pub const MAX_EXEMPTED_IDENTITIES: u32 = 10;
}

#[cfg(not(feature = "running-ci"))]
mod limits {
    pub const MAX_EXEMPTED_IDENTITIES: u32 = 1000;
}

benchmarks! {
    set_active_asset_stats {
        let i in 1..T::MaxStatsPerAsset::get().saturating_sub(1);

        let (owner, ticker, stats, _) = init_transfer_conditions::<T>(i, 0);

    }: _(owner.origin, ticker.into(), stats)

    batch_update_asset_stats {
        let i in 1..COUNTRY_CODES.len() as u32;

        let max_stats = T::MaxStatsPerAsset::get().saturating_sub(1);
        let (owner, ticker, stats, _) = init_transfer_conditions::<T>(max_stats, 0);

        // Get a Jurisdiction stat type.
        let stat_type = stats.iter().find(|s| match s.claim_issuer {
            Some((ClaimType::Jurisdiction, _)) => true,
            _ => false,
        }).cloned().unwrap();

        // Set active stats.
        Pallet::<T>::set_active_asset_stats(owner.origin.clone().into(), ticker.into(), stats)?;

        // Generate updates.
        let updates = make_jur_stat_updates(i, Some(1000u128));
    }: _(owner.origin, ticker.into(), stat_type, updates)

    set_asset_transfer_compliance {
        let i in 1..T::MaxTransferConditionsPerAsset::get().saturating_sub(1);

        let max_stats = T::MaxStatsPerAsset::get().saturating_sub(1);
        let (owner, ticker, stats, conditions) = init_transfer_conditions::<T>(max_stats, i);

        // Set active stats.
        Pallet::<T>::set_active_asset_stats(owner.origin.clone().into(), ticker.into(), stats)?;

    }: _(owner.origin, ticker.into(), conditions)

    set_entities_exempt {
        // Number of exempt entities being added.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, exempt_key, scope_ids) = init_exempts::<T>(i);
    }: set_entities_exempt(owner.origin, true, exempt_key, scope_ids)

    update_asset_count_stats {
        // Number of times `AssetStats` is read/written
        let a in 0..2;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let issuer_id = IdentityId::from(0);
        let stat_type = StatType{
            operation_type: StatOpType::Count,
            claim_issuer: Some((ClaimType::Accredited, issuer_id))
        };
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let asset_id = create_and_issue_sample_asset::<T>(alice.account(), true, None, b"MyAsset", true);
        let key1 = Stat1stKey { asset_id, stat_type };

        let changes = {
            if a == 0 {
                (true, true)
            } else if a == 1 {
                (false, true)
            } else {
                add_identity_claim::<T>(
                    alice.did(),
                    Claim::Accredited(Scope::Asset(asset_id)),
                    issuer_id,
                );
                (true, true)
            }
        };
    }: {
        let from_key2 = Pallet::<T>::fetch_claim_as_key(Some(&alice.did()), &key1);
        let to_key2 = Pallet::<T>::fetch_claim_as_key(Some(&bob.did()), &key1);
        Pallet::<T>::update_asset_count_stats(
            key1,
            from_key2,
            to_key2,
            changes,
            &mut weight_meter
        )
        .unwrap();
    }

    update_asset_balance_stats {
        // Number of times `AssetStats` is read/written
        let a in 0..2;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let issuer_id = IdentityId::from(0);
        let stat_type = StatType{operation_type: StatOpType::Balance, claim_issuer: Some((ClaimType::Accredited, issuer_id))};
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let asset_id = create_and_issue_sample_asset::<T>(alice.account(), true, None, b"MyAsset", true);
        let key1 = Stat1stKey { asset_id, stat_type };
        let (from_balance, to_balance) = {
            if a == 0 {
                (Some(ONE_UNIT), Some(ONE_UNIT))
            } else {
                add_identity_claim::<T>(
                    alice.did(),
                    Claim::Accredited(Scope::Asset(asset_id)),
                    issuer_id,
                );
                if a == 1 {
                    (Some(ONE_UNIT), None)
                } else {
                    (Some(ONE_UNIT), Some(ONE_UNIT))
                }
            }
        };
    }: {
        let from_key2 = Pallet::<T>::fetch_claim_as_key(Some(&alice.did()), &key1);
        let to_key2 = Pallet::<T>::fetch_claim_as_key(Some(&bob.did()), &key1);
        Pallet::<T>::update_asset_balance_stats(
            key1,
            from_key2,
            to_key2,
            from_balance,
            to_balance,
            ONE_UNIT,
            &mut weight_meter
        )
        .unwrap();
    }

    active_asset_statistics_load {
        let a in 1..T::MaxStatsPerAsset::get();

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = AssetId::new([a as u8; 16]);

        let statistics: BTreeSet<StatType> = (0..a)
            .map(|a| StatType {
                operation_type: StatOpType::Count,
                claim_issuer: Some((ClaimType::Accredited, alice.did())),
            })
            .collect();
        let statistics: BoundedBTreeSet<StatType, T::MaxStatsPerAsset> = statistics.try_into().unwrap();
        ActiveAssetStats::<T>::insert(&asset_id, statistics);
    }: {
        ActiveAssetStats::<T>::get(asset_id).into_iter();
    }

    ensure_valid_statistics_common {
        // The maximum number of fungible assets
        let n in 1..10;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_and_issue_sample_asset::<T>(alice.account(), true, None, b"MyAsset", true);

        let transfer_conditions = (0..T::MaxTransferConditionsPerAsset::get())
            .map(|i| TransferCondition::MaxInvestorCount((100+i).into()))
            .collect::<BTreeSet<_>>();
        let transfer_conditions = transfer_conditions.try_into().unwrap();
        let asset_transfer_compliance = AssetTransferCompliance::new(false, transfer_conditions);
        AssetTransferCompliances::<T>::insert(asset_id.clone(), asset_transfer_compliance);

        let total_rcv_per_did = (0..n)
            .map(|i| (IdentityId::from(i as u128), 0))
            .collect::<BTreeMap<_, _>>();
        let total_sent_per_did = (0..n)
            .map(|i| (IdentityId::from((i + n) as u128), 0))
            .collect::<BTreeMap<_, _>>();
    }: {
        let investors_update = Pallet::<T>::calculate_investors_balance(
            &asset_id,
            &total_rcv_per_did,
            &total_sent_per_did,
        )
        .unwrap();

        let asset_total_supply = T::AssetFn::asset_total_supply(&asset_id).unwrap();
    }

    is_exempt_from_condition {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = AssetId::new([0 as u8; 16]);
        let transfer_condition = TransferCondition::ClaimOwnership(
            StatClaim::Jurisdiction(Some(CountryCode::BR)),
            alice.did(),
            Permill::zero(),
            Permill::zero(),
        );
        TransferConditionExemptEntities::<T>::insert(
            transfer_condition.get_exempt_key(asset_id.clone()),
            alice.did(),
            true,
        );
    }: {
        assert!(
            Pallet::<T>::is_exempt_from_condition(
                &alice.did(),
                &asset_id,
                StatOpType::Balance,
                Some(ClaimType::Jurisdiction),
                &mut WeightMeter::max_limit_no_minimum()
            )
            .unwrap()
        );
    }

    has_matching_claim {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let issuer = UserBuilder::<T>::default().generate_did().build("issuer");
        let asset_id = AssetId::new([0 as u8; 16]);
        let claim = Claim::Jurisdiction(CountryCode::BR, Scope::Asset(asset_id));
        pallet_identity::Claims::<T>::insert(
            Claim1stKey::new(alice.did(), claim.claim_type()),
            Claim2ndKey::new(issuer.did(), Some(Scope::Asset(asset_id))),
            IdentityClaim::from(claim.clone()),
        );
    }: {
        assert!(
            Pallet::<T>::has_matching_claim(
                &alice.did(),
                &Stat1stKey::new(asset_id, StatType::new(StatOpType::Count, Some((claim.claim_type(), issuer.did())))),
                &Stat2ndKey::new_from(&claim.claim_type(), Some(claim)),
                &mut WeightMeter::max_limit_no_minimum()
            )
            .unwrap()
        );
    }

    asset_stats_read {
        let asset_id = AssetId::new([0 as u8; 16]);
        let stat_first_key = Stat1stKey::investor_count(asset_id.clone());
        let stat_second_key = Stat2ndKey::NoClaimStat;
        AssetStats::<T>::insert(stat_first_key, stat_second_key.clone(), 100);
    }: {
        assert_eq!(AssetStats::<T>::get(stat_first_key, stat_second_key), 100);
    }

    transfer_compliance_read {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_and_issue_sample_asset::<T>(alice.account(), true, None, b"MyAsset", true);

        let transfer_conditions = (0..T::MaxTransferConditionsPerAsset::get())
            .map(|i| TransferCondition::MaxInvestorCount((100+i).into()))
            .collect::<BTreeSet<_>>();
        let transfer_conditions = transfer_conditions.try_into().unwrap();
        let asset_transfer_compliance = AssetTransferCompliance::new(false, transfer_conditions);
        AssetTransferCompliances::<T>::insert(asset_id.clone(), asset_transfer_compliance);
    }: {
        let _ = AssetTransferCompliances::<T>::get(&asset_id);
    }
}
