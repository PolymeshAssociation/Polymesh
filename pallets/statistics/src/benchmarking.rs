use frame_benchmarking::benchmarks;
use sp_runtime::Permill;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

use polymesh_common_utilities::benchs::{make_asset, AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::constants::currency::{ONE_UNIT, POLY};
use polymesh_common_utilities::traits::{asset::Config as Asset, TestUtilsFn};
use polymesh_primitives::{jurisdiction::*, statistics::*, Claim, ClaimType, Scope};

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
                op,
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
        .map(|(_idx, stat)| match (stat.op, stat.claim_issuer) {
            (StatOpType::Count, None) => TransferCondition::MaxInvestorCount(10),
            (StatOpType::Balance, None) => TransferCondition::MaxInvestorOwnership(p40),
            (StatOpType::Count, Some((claim_type, issuer))) => {
                let claim = claim_type_to_stat_claim(claim_type).expect("Unsupported ClaimType");
                TransferCondition::ClaimCount(claim, issuer, 0, Some(10))
            }
            (StatOpType::Balance, Some((claim_type, issuer))) => {
                let claim = claim_type_to_stat_claim(claim_type).expect("Unsupported ClaimType");
                TransferCondition::ClaimOwnership(claim, issuer, p0, p40)
            }
        })
        .collect()
}

fn init_ticker<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, Ticker) {
    let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
    let ticker = make_asset::<T>(&owner, Some(b"1"));
    (owner, ticker)
}

fn init_transfer_conditions<T: Config + Asset + TestUtilsFn<AccountIdOf<T>>>(
    count_stats: u32,
    count_conditions: u32,
) -> (
    User<T>,
    Ticker,
    BTreeSet<StatType>,
    BTreeSet<TransferCondition>,
) {
    let (owner, ticker) = init_ticker::<T>();
    let stats = make_stats(count_stats);
    let conditions = make_transfer_conditions(&stats, count_conditions);
    (owner, ticker, stats, conditions)
}

fn init_exempts<T: Config + Asset + TestUtilsFn<AccountIdOf<T>>>(
    count: u32,
) -> (User<T>, TransferConditionExemptKey, BTreeSet<IdentityId>) {
    let (owner, ticker) = init_ticker::<T>();
    let scope_ids = (0..count as u128).map(IdentityId::from).collect();
    let exempt_key = TransferConditionExemptKey {
        asset: ticker.into(),
        op: StatOpType::Count,
        claim_type: Some(ClaimType::Accredited),
    };
    (owner, exempt_key, scope_ids)
}

/// Exempts `exempt_user_id` to follow a transfer condition of claim type `Accredited` for `ticker`.
pub fn set_transfer_exception<T: Config>(
    origin: T::RuntimeOrigin,
    ticker: Ticker,
    exempt_user_id: IdentityId,
) {
    let transfer_exception = TransferConditionExemptKey {
        asset: AssetScope::Ticker(ticker),
        op: StatOpType::Balance,
        claim_type: Some(ClaimType::Accredited),
    };
    Module::<T>::set_entities_exempt(
        origin.clone(),
        true,
        transfer_exception,
        [exempt_user_id].into(),
    )
    .unwrap();
}

/// Adds `claim` issued by `issuer_id` to `id`.
pub fn add_identity_claim<T: Config>(id: IdentityId, claim: Claim, issuer_id: IdentityId) {
    pallet_identity::Module::<T>::unverified_add_claim_with_scope(
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
    ticker: Ticker,
    n: u32,
    pause_restrictions: bool,
) {
    let asset_scope = AssetScope::from(ticker);

    // Adds the maximum number of active statistics
    let active_stats = (0..10)
        .map(|i| StatType {
            op: StatOpType::Count,
            claim_issuer: Some((ClaimType::Accredited, IdentityId::from(i as u128))),
        })
        .collect();
    Module::<T>::set_active_asset_stats(origin.clone(), asset_scope, active_stats).unwrap();

    let transfer_conditions: BTreeSet<TransferCondition> = (0..n)
        .map(|i| {
            let issuer_id = IdentityId::from(i as u128);
            add_identity_claim::<T>(
                sender_id,
                Claim::Accredited(Scope::Ticker(ticker)),
                issuer_id,
            );
            TransferCondition::ClaimCount(StatClaim::Accredited(true), issuer_id, 0, Some(1))
        })
        .collect();
    Module::<T>::set_asset_transfer_compliance(origin.clone(), asset_scope, transfer_conditions)
        .unwrap();
    if pause_restrictions {
        ActiveAssetStats::<T>::remove(&asset_scope);
        AssetTransferCompliances::<T>::mutate(asset_scope, |atc| {
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
    where_clause { where T: Asset, T: TestUtilsFn<AccountIdOf<T>> }

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
        Module::<T>::set_active_asset_stats(owner.origin.clone().into(), ticker.into(), stats)?;

        // Generate updates.
        let updates = make_jur_stat_updates(i, Some(1000u128));
    }: _(owner.origin, ticker.into(), stat_type, updates)

    set_asset_transfer_compliance {
        let i in 1..T::MaxTransferConditionsPerAsset::get().saturating_sub(1);

        let max_stats = T::MaxStatsPerAsset::get().saturating_sub(1);
        let (owner, ticker, stats, conditions) = init_transfer_conditions::<T>(max_stats, i);

        // Set active stats.
        Module::<T>::set_active_asset_stats(owner.origin.clone().into(), ticker.into(), stats)?;

    }: _(owner.origin, ticker.into(), conditions)

    set_entities_exempt {
        // Number of exempt entities being added.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, exempt_key, scope_ids) = init_exempts::<T>(i);
    }: set_entities_exempt(owner.origin, true, exempt_key, scope_ids)

    max_investor_count_restriction {
        // If `AssetStats` should be read
        let a in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let transfer_condition = TransferCondition::MaxInvestorCount(1);
        let changes = if a == 1 { Some((false, true)) } else { Some((true, true)) };
    }: {
        assert!(Module::<T>::check_transfer_condition(
            &transfer_condition,
            AssetScope::Ticker(ticker),
            &alice.did(),
            &bob.did(),
            0,
            ONE_UNIT,
            ONE_UNIT * POLY,
            changes,
            &mut weight_meter
        ).unwrap());
    }

    max_investor_ownership_restriction {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let transfer_condition = TransferCondition::MaxInvestorOwnership(Permill::one());
    }: {
        assert!(Module::<T>::check_transfer_condition(
            &transfer_condition,
            AssetScope::Ticker(ticker),
            &alice.did(),
            &bob.did(),
            0,
            ONE_UNIT,
            ONE_UNIT * POLY,
            None,
            &mut weight_meter
        ).unwrap());
    }

    claim_count_restriction_no_stats {
        // If `Claims` should be read
        let c in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let changes = if c == 0 { None } else { Some((false, false)) };
        let transfer_condition =
            TransferCondition::ClaimCount(StatClaim::Accredited(true), alice.did(), 0, Some(1));
    }: {
        assert!(Module::<T>::check_transfer_condition(
            &transfer_condition,
            AssetScope::Ticker(ticker),
            &alice.did(),
            &bob.did(),
            0,
            ONE_UNIT,
            ONE_UNIT * POLY,
            changes,
            &mut weight_meter
        ).unwrap());
    }

    claim_count_restriction_with_stats {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let transfer_condition =
            TransferCondition::ClaimCount(StatClaim::Accredited(true), alice.did(), 0, Some(1));
    }: {
        assert!(Module::<T>::check_transfer_condition(
            &transfer_condition,
            AssetScope::Ticker(ticker),
            &alice.did(),
            &bob.did(),
            0,
            ONE_UNIT,
            ONE_UNIT * POLY,
            Some((false, true)),
            &mut weight_meter
        ).unwrap());
    }

    claim_ownership_restriction {
        // If `AssetStats` should be read
        let a in 0..1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let transfer_condition =
            TransferCondition::ClaimOwnership(StatClaim::Accredited(true), IdentityId::from(0), Permill::zero(), Permill::one());
        if a == 1 {
            add_identity_claim::<T>(
                alice.did(),
                Claim::Accredited(Scope::Ticker(ticker)),
                IdentityId::from(0)
            );
        }
    }: {
        assert!(Module::<T>::check_transfer_condition(
            &transfer_condition,
            AssetScope::Ticker(ticker),
            &alice.did(),
            &bob.did(),
            0,
            ONE_UNIT,
            ONE_UNIT * POLY,
            None,
            &mut weight_meter
        ).unwrap());
    }

    update_asset_count_stats {
        // Number of times `AssetStats` is read/written
        let a in 0..2;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let issuer_id = IdentityId::from(0);
        let asset = AssetScope::Ticker(ticker);
        let stat_type = StatType{op: StatOpType::Count, claim_issuer: Some((ClaimType::Accredited, issuer_id))};
        let key1 = Stat1stKey { asset, stat_type };
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let changes = {
            if a == 0 {
                (true, true)
            } else if a == 1 {
                (false, true)
            } else {
                add_identity_claim::<T>(
                    alice.did(),
                    Claim::Accredited(Scope::Ticker(ticker)),
                    issuer_id,
                );
                (true, true)
            }
        };
    }: {
        let from_key2 = Module::<T>::fetch_claim_as_key(Some(&alice.did()), &key1);
        let to_key2 = Module::<T>::fetch_claim_as_key(Some(&bob.did()), &key1);
        Module::<T>::update_asset_count_stats(
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
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let issuer_id = IdentityId::from(0);
        let asset = AssetScope::Ticker(ticker);
        let stat_type = StatType{op: StatOpType::Balance, claim_issuer: Some((ClaimType::Accredited, issuer_id))};
        let key1 = Stat1stKey { asset, stat_type };
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let (from_balance, to_balance) = {
            if a == 0 {
                (Some(ONE_UNIT), Some(ONE_UNIT))
            } else {
                add_identity_claim::<T>(
                    alice.did(),
                    Claim::Accredited(Scope::Ticker(ticker)),
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
        let from_key2 = Module::<T>::fetch_claim_as_key(Some(&alice.did()), &key1);
        let to_key2 = Module::<T>::fetch_claim_as_key(Some(&bob.did()), &key1);
        Module::<T>::update_asset_balance_stats(
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

    verify_requirements {
        let i in 0..T::MaxTransferConditionsPerAsset::get();

        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let asset_scope = AssetScope::Ticker(ticker);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        let transfer_conditions: BTreeSet<TransferCondition> = (0..i)
            .map(|i| TransferCondition::MaxInvestorCount(i as u64))
            .collect();
    }: {
        assert!(
            Module::<T>::verify_requirements::<T::MaxTransferConditionsPerAsset>(
                &transfer_conditions.try_into().unwrap(),
                asset_scope,
                &alice.did(),
                &bob.did(),
                ONE_UNIT,
                1,
                1,
                ONE_UNIT,
                &mut weight_meter
            )
            .is_ok()
        );
    }

    active_asset_statistics_load {
        let a in 1..T::MaxStatsPerAsset::get();

        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_scope = AssetScope::Ticker(ticker);

        let statistics: BTreeSet<StatType> = (0..a)
            .map(|a| StatType {
                op: StatOpType::Count,
                claim_issuer: Some((ClaimType::Accredited, alice.did())),
            })
            .collect();
        let statistics: BoundedBTreeSet<StatType, T::MaxStatsPerAsset> = statistics.try_into().unwrap();
        ActiveAssetStats::<T>::insert(&asset_scope, statistics);
    }: {
        Module::<T>::active_asset_stats(asset_scope).into_iter();
    }

    is_exempt {
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let asset_scope = AssetScope::Ticker(ticker);
        let statistic_claim = StatClaim::Jurisdiction(Some(CountryCode::BR));
        let transfer_condition = TransferCondition::ClaimOwnership(statistic_claim, alice.did(), Permill::zero(), Permill::zero());
        TransferConditionExemptEntities::insert(transfer_condition.get_exempt_key(asset_scope.clone()), bob.did(), true);
    }: {
        assert!(
            Module::<T>::is_exempt(
                asset_scope,
                &transfer_condition,
                &alice.did(),
                &bob.did()
            )
        );
    }
}
