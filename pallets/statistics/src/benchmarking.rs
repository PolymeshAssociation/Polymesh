use crate::*;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    benchs::{make_asset, AccountIdOf, User, UserBuilder},
    traits::{asset::Config as Asset, TestUtilsFn},
};
use polymesh_primitives::{jurisdiction::*, statistics::*, ClaimType};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

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

fn make_stats(issuer: IdentityId, count: u32) -> BTreeSet<StatType> {
    (0..count as usize)
        .into_iter()
        .map(|idx| {
            let (op, claim_type) = STAT_TYPES[idx % STAT_TYPES.len()];
            StatType {
                op,
                claim_issuer: claim_type.map(|ct| (ct, issuer)),
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

const TRANSFER_CONDITIONS: &[(StatOpType, Option<StatClaim>)] = &[
    (StatOpType::Count, None),
    (StatOpType::Balance, None),
    (StatOpType::Count, Some(StatClaim::Accredited(true))),
    (StatOpType::Balance, Some(StatClaim::Accredited(true))),
    (StatOpType::Count, Some(StatClaim::Accredited(false))),
    (StatOpType::Balance, Some(StatClaim::Accredited(false))),
    (StatOpType::Count, Some(StatClaim::Affiliate(true))),
    (StatOpType::Balance, Some(StatClaim::Affiliate(true))),
    (StatOpType::Count, Some(StatClaim::Affiliate(false))),
    (StatOpType::Balance, Some(StatClaim::Affiliate(false))),
    (StatOpType::Count, Some(StatClaim::Jurisdiction(None))),
    (StatOpType::Balance, Some(StatClaim::Jurisdiction(None))),
    (
        StatOpType::Count,
        Some(StatClaim::Jurisdiction(Some(CountryCode::CA))),
    ),
    (
        StatOpType::Balance,
        Some(StatClaim::Jurisdiction(Some(CountryCode::CA))),
    ),
    (
        StatOpType::Count,
        Some(StatClaim::Jurisdiction(Some(CountryCode::US))),
    ),
    (
        StatOpType::Balance,
        Some(StatClaim::Jurisdiction(Some(CountryCode::US))),
    ),
];

fn make_transfer_conditions(issuer: IdentityId, count: u32) -> BTreeSet<TransferCondition> {
    let p0 = HashablePermill(sp_arithmetic::Permill::from_rational(0u32, 100u32));
    let p40 = HashablePermill(sp_arithmetic::Permill::from_rational(40u32, 100u32));
    (0..count as usize)
        .into_iter()
        .map(
            |idx| match TRANSFER_CONDITIONS[idx % TRANSFER_CONDITIONS.len()] {
                (StatOpType::Count, None) => TransferCondition::MaxInvestorCount(10),
                (StatOpType::Balance, None) => TransferCondition::MaxInvestorOwnership(p40),
                (StatOpType::Count, Some(c)) => {
                    TransferCondition::ClaimCount(c, issuer, 0, Some(10))
                }
                (StatOpType::Balance, Some(c)) => {
                    TransferCondition::ClaimOwnership(c, issuer, p0, p40)
                }
            },
        )
        .collect()
}

fn init_ticker<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, Ticker) {
    let owner = UserBuilder::<T>::default().generate_did().build("OWNER");
    let ticker = make_asset::<T>(&owner, Some(b"1"));
    (owner, ticker)
}

fn init_ctm<T: Config + Asset + TestUtilsFn<AccountIdOf<T>>>(
    max_transfer_manager_per_asset: u32,
) -> (User<T>, Ticker, Vec<TransferManager>) {
    let (owner, ticker) = init_ticker::<T>();
    let tms = (0..max_transfer_manager_per_asset)
        .map(|x| TransferManager::CountTransferManager(x.into()))
        .collect::<Vec<_>>();
    ActiveTransferManagers::insert(ticker, tms.clone());
    (owner, ticker, tms)
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
    let issuer = owner.did.expect("Owner missing identity");
    let stats = make_stats(issuer, count_stats);
    let conditions = make_transfer_conditions(issuer, count_conditions);
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

    add_transfer_manager {
        let max_tm = T::MaxTransferManagersPerAsset::get().saturating_sub(1);
        let (owner, ticker, mut tms) = init_ctm::<T>(max_tm);

        let last_tm = TransferManager::CountTransferManager(max_tm as u64 + 42u64);
        tms.push(last_tm.clone());
    }: _(owner.origin, ticker, last_tm)
    verify {
        assert_eq!(Module::<T>::transfer_managers(ticker), tms);
    }

    remove_transfer_manager {
        let (owner, ticker, mut tms) = init_ctm::<T>(T::MaxTransferManagersPerAsset::get());
        let last_tm = tms.pop().expect("MaxTransferManagersPerAsset should be greater than zero");
    }: _(owner.origin, ticker, last_tm)
    verify {
        assert_eq!(Module::<T>::transfer_managers(ticker), tms);
    }

    add_exempted_entities {
        // Length of the vector of Exempted identities being added.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, ticker) = init_ticker::<T>();
        let scope_ids = (0..i as u128).map(IdentityId::from).collect::<Vec<_>>();
        let tm = TransferManager::CountTransferManager(420);
        let ephemeral_tm = tm.clone();
    }: _(owner.origin, ticker, ephemeral_tm, scope_ids)
    verify {
        assert!(Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)) == (i != 0));
    }

    remove_exempted_entities {
        // Length of the vector of Exempted identities being removed.
        let i in 0 .. limits::MAX_EXEMPTED_IDENTITIES;

        let (owner, ticker) = init_ticker::<T>();
        let tm = TransferManager::CountTransferManager(420);
        let scope_ids = (0..i).map(|x| {
            let scope_id = IdentityId::from(x as u128);
            ExemptEntities::insert((ticker, tm.clone()), scope_id.clone(), true);
            scope_id
        }).collect::<Vec<_>>();
        let ephemeral_tm = tm.clone();
    }: _(owner.origin, ticker, ephemeral_tm, scope_ids)
    verify {
        assert!(!Module::<T>::entity_exempt((ticker, tm), IdentityId::from(0u128)));
    }

    #[extra]
    verify_tm_restrictions {
        let t in 0 .. T::MaxTransferManagersPerAsset::get();

        let (owner, ticker) = init_ticker::<T>();
        let owner_did = owner.did.unwrap();
        let tms = (0..t).map(|x| {
            let tm = TransferManager::CountTransferManager(x.into());
            ExemptEntities::insert((ticker, tm.clone()), owner_did, true);
            tm
        }).collect::<Vec<_>>();
        ActiveTransferManagers::insert(ticker, tms.clone());
        InvestorCountPerAsset::insert(ticker, 1337);
    }: {
        // This will trigger the worse case (exemption)
        Module::<T>::verify_tm_restrictions(
            &ticker,
            owner_did,
            owner_did,
            100u32.into(),
            200u32.into(),
            0u32.into(),
            500u32.into(),
        ).unwrap();
    }

    set_active_asset_stats {
        let i in 1..T::MaxStatsPerAsset::get().saturating_sub(1);

        let (owner, ticker, stats, _) = init_transfer_conditions::<T>(i, 0);

    }: _(owner.origin, ticker.into(), stats)

    batch_update_asset_stats {
        let i in 1..COUNTRY_CODES.len() as u32;

        let max_stats = T::MaxStatsPerAsset::get().saturating_sub(1);
        let (owner, ticker, stats, _) = init_transfer_conditions::<T>(max_stats, 0);

        // Set active stats.
        Module::<T>::set_active_asset_stats(owner.origin.clone().into(), ticker.into(), stats)?;

        // Generate updates.
        let issuer = owner.did.expect("Owner missing identity");
        let stat_type = StatType {
            op: StatOpType::Count,
            claim_issuer: Some((ClaimType::Jurisdiction, issuer)),
        };
        let updates = make_jur_stat_updates(i, Some(1000u128));
    }: _(owner.origin, ticker.into(), stat_type, updates)

    set_asset_transfer_compliance {
        let i in 1..T::MaxTransferManagersPerAsset::get().saturating_sub(1);

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
}
