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
) -> (User<T>, TransferConditionExemptKey, BTreeSet<ScopeId>) {
    let (owner, ticker) = init_ticker::<T>();
    let scope_ids = (0..count as u128).map(IdentityId::from).collect();
    let exempt_key = TransferConditionExemptKey {
        asset: ticker.into(),
        op: StatOpType::Count,
        claim_type: Some(ClaimType::Accredited),
    };
    (owner, exempt_key, scope_ids)
}

/// Returns a set of `StatType`.
pub fn statistics_types(n_issuers: u32) -> BTreeSet<StatType> {
    (0..n_issuers)
        .map(|idx| StatType {
            op: StatOpType::Balance,
            claim_issuer: Some((ClaimType::Accredited, IdentityId::from(idx as u128))),
        })
        .collect()
}

/// Returns a set of `c` `TransferCondition` that will succeed and `t` conditions that will fail.
pub fn transfer_conditions(c: u32, t: u32) -> BTreeSet<TransferCondition> {
    let valid_claim_conditions: Vec<TransferCondition> = (0..c)
        .map(|idx| {
            TransferCondition::ClaimOwnership(
                StatClaim::Accredited(true),
                IdentityId::from(idx as u128),
                Permill::zero(),
                Permill::one(),
            )
        })
        .collect();

    let failed_claim_conditions: Vec<TransferCondition> = (0..t)
        .map(|_| {
            TransferCondition::ClaimOwnership(
                StatClaim::Accredited(true),
                IdentityId::from(0),
                Permill::one(),
                Permill::zero(),
            )
        })
        .collect();

    let all_conditions = [valid_claim_conditions, failed_claim_conditions].concat();
    all_conditions.into_iter().collect()
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

    verify_transfer_restrictions {
        // In the worst case, each condition (at most T::MaxTransferConditionsPerAsset) calls `Identity::<T>::fetch_claim' two times
        // and `AssetStats` one time. The number of calls to `TransferConditionExemptEntities` would be 12
        // (3 (Accredited, Affiliate, Jurisdiction) * 2 (balance, count) * 2 (from_did, to_did)).

        let c in 0..T::MaxTransferConditionsPerAsset::get() - 2;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let asset_scope = AssetScope::Ticker(ticker);

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let statistics_types = statistics_types(T::MaxStatsPerAsset::get() -1);
        Module::<T>::set_active_asset_stats(alice.origin().into(), asset_scope, statistics_types).unwrap();
        let transfer_conditions = transfer_conditions(c, 1);
        set_transfer_exception::<T>(alice.origin().into(), ticker, bob.did());
        Module::<T>::base_set_asset_transfer_compliance(alice.origin().into(), asset_scope, transfer_conditions).unwrap();

        for idx in 0..c {
            add_identity_claim::<T>(
                alice.did(),
                Claim::Accredited(Scope::Ticker(ticker)),
                IdentityId::from(idx as u128)
            );
        }
    }: {
        Module::<T>::verify_transfer_restrictions(
            &ticker,
            alice.did(),
            bob.did(),
            &alice.did(),
            &bob.did(),
            ONE_UNIT * POLY,
            0,
            ONE_UNIT,
            ONE_UNIT * POLY
        )
        .unwrap();
    }

    update_asset_stats {
        // In the worst case, `ActiveAssetStats` has the maximum number of statistics (T::MaxStatsPerAsset), all
        // are of type `StatOpType::Balance` and have identity claims. This would mean four reads and two writes everytime.

        // Number of active statistics
        let s in 0..T::MaxStatsPerAsset::get() -1;

        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let bob = UserBuilder::<T>::default().generate_did().build("Bob");
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let asset_scope = AssetScope::Ticker(ticker);

        make_asset::<T>(&alice, Some(ticker.as_ref()));
        let statistics_types = statistics_types(s);
        Module::<T>::set_active_asset_stats(alice.origin().into(), asset_scope, statistics_types).unwrap();
        for i in 0..s {
            add_identity_claim::<T>(
                alice.did(),
                Claim::Accredited(Scope::Ticker(ticker)),
                IdentityId::from(i as u128)
            );
        }
    }: {
        Module::<T>::update_asset_stats(
            &ticker,
            Some(&alice.did()),
            Some(&bob.did()),
            Some(ONE_UNIT * POLY),
            Some(ONE_UNIT),
            ONE_UNIT
        );
    }
}
