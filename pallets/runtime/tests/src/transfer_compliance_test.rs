use super::{
    storage::{
        account_from, create_investor_uid, make_account, provide_scope_claim, TestStorage, User,
    },
    ExtBuilder,
};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
};
use polymesh_primitives::{
    asset::AssetType, jurisdiction::CountryCode, statistics::*, transfer_compliance::*, AccountId,
    Balance, CddId, Claim, ClaimType, IdentityId, InvestorUid, PortfolioId,
    Scope, ScopeId, Ticker,
};
use sp_std::convert::TryFrom;
use std::collections::{HashMap, HashSet};
use test_client::AccountKeyring;

type Origin = <TestStorage as frame_system::Config>::Origin;
type Identity = pallet_identity::Module<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type Statistics = pallet_statistics::Module<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type Error = pallet_statistics::Error<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;

const CDD_PROVIDER: AccountKeyring = AccountKeyring::Eve;

#[derive(Clone)]
struct InvestorState {
    id: u64,
    did: IdentityId,
    acc: AccountId,
    balance: Balance,
    claims: HashMap<(ClaimType, IdentityId), Claim>,
}

impl InvestorState {
    pub fn new(id: u64) -> Self {
        let acc = account_from(id);
        let (_, did) = make_account(acc.clone()).expect("Failed to make account.");
        Self {
            id,
            did,
            acc,
            balance: Default::default(),
            claims: Default::default(),
        }
    }

    fn origin(&self) -> Origin {
        Origin::signed(self.acc.clone())
    }

    pub fn uid(&self) -> InvestorUid {
        create_investor_uid(self.acc.clone())
    }

    pub fn provide_scope_claim(&self, ticker: Ticker) -> (ScopeId, CddId) {
        provide_scope_claim(
            self.did,
            ticker,
            self.uid(),
            CDD_PROVIDER.to_account_id(),
            None,
        )
    }

    pub fn add_issuer_claim(&mut self, did: &IdentityId, acc: &AccountId, claim: &Claim) {
        assert_ok!(Identity::add_claim(
            Origin::signed(acc.clone()),
            self.did,
            claim.clone(),
            None,
        ));
        let claim_type = claim.claim_type();
        self.claims.insert((claim_type, *did), claim.clone());
    }

    pub fn fetch_claim(&self, claim_issuer: &(ClaimType, IdentityId)) -> Option<&Claim> {
        self.claims.get(claim_issuer)
    }
}

#[derive(Clone)]
struct IssuerState {
    issuer: User,
    trusted_for: HashSet<ClaimType>,
    claims: HashSet<Claim>,
}

impl IssuerState {
    pub fn new(issuer: User, trusted_for: &[ClaimType]) -> Self {
        Self {
          issuer,
          trusted_for: trusted_for.iter().cloned().collect(),
          claims: HashSet::new(),
        }
    }

    pub fn is_trusted_for(&self, claim_type: &ClaimType) -> bool {
        self.trusted_for.contains(claim_type)
    }

    pub fn add_claim(&mut self, claim: &Claim) {
        self.claims.insert(claim.clone());
    }

    pub fn fetch_claims(&self, claim_type: ClaimType) -> Vec<Claim> {
        self.claims.iter()
            .filter(|c| c.claim_type() == claim_type)
            .cloned()
            .collect()
    }

    pub fn did(&self) -> IdentityId {
        self.issuer.did
    }
}

#[derive(Debug, Clone)]
struct Batch {
    ids: Vec<u64>,
}

#[derive(Clone)]
struct AssetTracker {
    name: String,
    asset: Ticker,
    total_supply: Balance,
    disable_iu: bool,
    asset_scope: AssetScope,

    issuers: HashMap<IdentityId, IssuerState>,

    owner_id: u64,
    investor_start_id: u64,
    investor_next_id: u64,
    investors: HashMap<u64, InvestorState>,

    active_stats: Vec<StatType>,
    transfer_conditions: Vec<TransferCondition>,
}

impl AssetTracker {
    pub fn new() -> Self {
        Self::new_full(0, "ACME")
    }

    pub fn new_full(owner_id: u64, name: &str) -> Self {
        let asset = Ticker::try_from(name.as_bytes()).expect("Bad ticker");
        let investor_start_id = owner_id + 1000;
        let mut tracker = Self {
            name: name.into(),
            asset,
            total_supply: 0,
            disable_iu: false,
            asset_scope: AssetScope::from(asset),

            issuers: HashMap::new(),

            owner_id,
            investor_start_id,
            investor_next_id: investor_start_id,
            investors: HashMap::new(),

            active_stats: Vec::new(),
            transfer_conditions: Vec::new(),
        };
        tracker.init();

        tracker
    }

    fn init(&mut self) {
        // Create the owner investor state.
        self.create_investor(self.owner_id);

        assert_ok!(Asset::create_asset(
            self.owner_origin(),
            format!("Token {}", self.name).into(),
            self.asset,
            true,
            AssetType::default(),
            vec![],
            None,
            self.disable_iu,
        ));

        self.allow_all_transfers();
    }

    pub fn set_active_stats(&mut self, active_stats: Vec<StatType>) {
        assert_ok!(Statistics::set_active_asset_stats(
            self.owner_origin(),
            self.asset_scope,
            active_stats.clone(),
        ));
        self.active_stats = active_stats;
    }

    pub fn owner(&self) -> &InvestorState {
        self.investors.get(&self.owner_id).expect("Missing owner")
    }

    pub fn owner_mut(&mut self) -> &mut InvestorState {
        self.investors
            .get_mut(&self.owner_id)
            .expect("Missing owner")
    }

    pub fn owner_origin(&self) -> Origin {
        self.owner().origin()
    }

    pub fn mint(&mut self, amount: Balance) {
        assert_ok!(Asset::issue(self.owner_origin(), self.asset, amount));
        self.total_supply += amount;
        self.owner_mut().balance += amount;
    }

    pub fn fake_sto(&mut self, amounts: &[(u64, Balance)]) -> DispatchResult {
        for (id, amount) in amounts.into_iter() {
            self.do_transfer(self.owner_id, *id, *amount)?;
        }
        Ok(())
    }

    pub fn add_issuer(&mut self, issuer: &User, trusted_for: &[ClaimType]) {
        let issuer = IssuerState::new(*issuer, trusted_for);
        self.issuers.insert(issuer.did(), issuer);
    }

    pub fn make_claim(&self, claim_type: ClaimType, jur: Option<CountryCode>) -> Claim {
        let scope = Scope::from(self.asset);
        match claim_type {
            ClaimType::Accredited => Claim::Accredited(scope),
            ClaimType::Affiliate => Claim::Affiliate(scope),
            ClaimType::BuyLockup => Claim::BuyLockup(scope),
            ClaimType::SellLockup => Claim::SellLockup(scope),
            ClaimType::KnowYourCustomer => Claim::KnowYourCustomer(scope),
            ClaimType::Jurisdiction => {
                let jur = jur.expect("Need Jurisdiction");
                Claim::Jurisdiction(jur, scope)
            }
            ClaimType::Exempted => Claim::Exempted(scope),
            ClaimType::Blocked => Claim::Blocked(scope),
            _ => {
                panic!("Asset issuers can't create {:?} claims", claim_type);
            },
        }
    }

    pub fn add_claim_to_investors(&mut self, ids: &[u64], claim_type: ClaimType, jur: Option<CountryCode>) {
        let claim = self.make_claim(claim_type, jur);

        // build list of (issuer, claim) pairs.
        let claims = self.issuers.values_mut()
            .filter(|issuer| issuer.is_trusted_for(&claim_type))
            .map(|i| {
                i.add_claim(&claim);
                (i.issuer.did, i.issuer.acc(), &claim)
            })
            .collect::<Vec<_>>();

        // add claims to each investor.
        for id in ids.into_iter() {
            for (did, acc, claim) in &claims {
                self.add_claim_to_investor(*id, did, acc, claim);
            }
        }
    }

    fn add_claim_to_investor(&mut self, id: u64, did: &IdentityId, acc: &AccountId, claim: &Claim) {
        self.investor_mut(id).add_issuer_claim(did, acc, claim)
    }

    pub fn investor_start_id(&self) -> u64 {
        self.investor_start_id
    }

    pub fn investor_count(&self) -> u64 {
        self.investor_next_id - self.investor_start_id
    }

    pub fn dump_investors(&self) {
        self.investors
            .values()
            .for_each(|i| {
                eprintln!("investor[{}]: bal={}", i.id, i.balance);
            })
    }

    /// The number of investors with `balance > 0`.
    pub fn active_investor_count(&self) -> u64 {
        self.investors
            .values()
            .filter(|i| i.balance > 0)
            .count() as u64
    }

    /// Count the number of investors with `balance > 0` and the matching claim.
    pub fn calculate_stat_count(&self, claim_issuer: Option<(ClaimType, IdentityId)>, claim: &Option<Claim>) -> u64 {
        if let Some(claim_issuer) = claim_issuer {
            self.investors
                .values()
                .filter(|i| {
                    i.balance > 0 &&
                        i.fetch_claim(&claim_issuer) == claim.as_ref()
                })
                .count() as u64
        } else {
            // Special case, count all investors with a balance.
            self.active_investor_count()
        }
    }

    /// Calculate the balance of all investors the matching claim.
    pub fn calculate_stat_balance(&self, claim_issuer: Option<(ClaimType, IdentityId)>, claim: &Option<Claim>) -> Balance {
        let claim_issuer = claim_issuer.expect("Need claim issuer for Balance stats.");
        self.investors.values()
            .filter(|i| i.fetch_claim(&claim_issuer) == claim.as_ref())
            .map(|i| i.balance)
            .sum()
    }

    pub fn make_investors(&mut self, count: u64) -> Batch {
        let ids = (0..count).map(|_| self.new_investor()).collect();
        Batch { ids }
    }

    pub fn make_batches(&mut self, batches: Vec<(u64, Balance, Vec<(ClaimType, Option<CountryCode>)>)>) -> Result<Vec<Batch>, DispatchError> {
        batches.into_iter()
            .map(|(size, sto_buy, claims)| self.new_batch(size, sto_buy, claims))
            .collect()
    }

    pub fn new_batch(&mut self, size: u64, sto_buy: Balance, claims: Vec<(ClaimType, Option<CountryCode>)>) -> Result<Batch, DispatchError> {
        eprintln!("Create batch: {:?}", (size, sto_buy, &claims));
        // Create investors for this batch.
        let batch = self.make_investors(size);

        // Add claims.
        for (claim_type, jur) in claims {
            self.add_claim_to_investors(&batch.ids[..], claim_type, jur);
        }

        // Fake STO.
        let sto = batch.ids.iter().map(|id| (*id, sto_buy)).collect::<Vec<_>>();
        self.fake_sto(sto.as_slice())?;

        Ok(batch)
    }

    pub fn new_investor(&mut self) -> u64 {
        let id = self.investor_next_id;
        self.investor_next_id += 1;
        self.create_investor(id);
        id
    }

    fn create_investor(&mut self, id: u64) {
        let investor = InvestorState::new(id);
        investor.provide_scope_claim(self.asset);
        assert!(self.investors.insert(id, investor).is_none());
    }

    pub fn investor(&self, id: u64) -> &InvestorState {
        self.investors.get(&id).expect("Missing investor")
    }

    pub fn investor_mut(&mut self, id: u64) -> &mut InvestorState {
        self.investors.get_mut(&id).expect("Missing investor")
    }

    #[track_caller]
    pub fn allow_all_transfers(&self) {
        assert_ok!(ComplianceManager::add_compliance_requirement(
            self.owner().origin(),
            self.asset,
            vec![],
            vec![]
        ));
    }

    fn get_investor_portfolio(&self, id: u64) -> PortfolioId {
        let did = self.investor(id).did;
        PortfolioId::default_portfolio(did)
    }

    fn do_transfer(&mut self, from: u64, to: u64, amount: u128) -> DispatchResult {
        Asset::base_transfer(
            self.get_investor_portfolio(from),
            self.get_investor_portfolio(to),
            &self.asset,
            amount,
        )?;

        // Update investor balances.
        self.investor_mut(from).balance -= amount;
        self.investor_mut(to).balance += amount;

        Ok(())
    }

    #[track_caller]
    pub fn do_valid_transfer(&mut self, from: u64, to: u64, amount: u128) {
        assert_ok!(self.do_transfer(from, to, amount));
    }

    #[track_caller]
    pub fn ensure_invalid_transfer(&mut self, from: u64, to: u64, amount: u128) {
        assert_noop!(
            self.do_transfer(from, to, amount),
            AssetError::InvalidTransfer
        );
    }

    #[track_caller]
    pub fn ensure_legacy_investor_count(&self) {
        let cal_count = self.active_investor_count();
        let count = Statistics::investor_count(&self.asset);
        eprintln!("calculated={}, count={}", cal_count, count);
        assert_eq!(cal_count, count);
    }

    pub fn fetch_stats_key2(&self, stat_type: &StatType) -> Vec<Stat2ndKey> {
        match stat_type.claim_issuer {
            None => vec![Stat2ndKey { claim: None }],
            Some((claim_type, did)) => {
                if let Some(issuer) = self.issuers.get(&did) {
                    issuer.fetch_claims(claim_type).into_iter()
                        .map(|claim| Stat2ndKey { claim: Some(claim) })
                        .collect()
                } else {
                    vec![]
                }
            }
        }
    }

    pub fn get_claim_stats(&self, op: StatOpType, claim_type: ClaimType, jur: Option<CountryCode>) -> Vec<(u128, u128)> {
        let key2 = Stat2ndKey {
            claim: Some(self.make_claim(claim_type, jur))
        };

        self.issuers.values()
            .filter(|issuer| issuer.is_trusted_for(&claim_type))
            .map(|i| {
                Stat1stKey {
                    asset: self.asset_scope,
                    stat_type: StatType {
                        op,
                        claim_issuer: Some((claim_type, i.issuer.did)),
                    }
                }
            })
            .map(|key1| {
                let claim_issuer = key1.stat_type.claim_issuer;
                // Calculate the expected value.
                let cal_value = match op {
                    StatOpType::Count => {
                        self.calculate_stat_count(claim_issuer, &key2.claim) as u128
                    }
                    StatOpType::Balance => {
                        self.calculate_stat_balance(claim_issuer, &key2.claim)
                    }
                };
                // Get stat from pallet.
                let value = Statistics::asset_stats(key1, key2.clone());
                (cal_value, value)
            })
            .collect()
    }

    pub fn get_claim_count_stats(&self, claim_type: ClaimType, jur: Option<CountryCode>) -> Vec<(u128, u128)> {
        self.get_claim_stats(StatOpType::Count, claim_type, jur)
    }

    pub fn get_claim_balance_stats(&self, claim_type: ClaimType, jur: Option<CountryCode>) -> Vec<(u128, u128)> {
        self.get_claim_stats(StatOpType::Balance, claim_type, jur)
    }

    pub fn get_claim_percent_stats(&self, claim_type: ClaimType, jur: Option<CountryCode>) -> Vec<(u128, u128)> {
        self.get_claim_stats(StatOpType::Balance, claim_type, jur)
            .into_iter()
            .map(|(cal, value) {
                (cal, value)
            })
            .collect()
    }

    #[track_caller]
    pub fn ensure_asset_stat(&self, stat_type: &StatType) {
        let key1 = Stat1stKey { asset: self.asset_scope, stat_type: *stat_type };
        for key2 in self.fetch_stats_key2(stat_type).iter() {
            let value = Statistics::asset_stats(key1, key2);
            match (stat_type.op, stat_type.claim_issuer) {
                (StatOpType::Count, claim_issuer) => {
                    let cal_value = self.calculate_stat_count(claim_issuer, &key2.claim);
                    eprintln!("Count[{:?}]: cal={:?}, stat={:?}", key2.claim, cal_value, value);
                    assert_eq!(value, cal_value as u128);
                }
                (StatOpType::Balance, None) => {
                    // Per-investor balances are not stored in the stats.
                }
                (StatOpType::Balance, claim_issuer) => {
                    let cal_value = self.calculate_stat_balance(claim_issuer, &key2.claim);
                    eprintln!("Balance[{:?}]: cal={:?}, stat={:?}", key2.claim, cal_value, value);
                    assert_eq!(value, cal_value as u128);
                }
            }
        }
    }

    #[track_caller]
    pub fn ensure_asset_stats(&self) {
        self.ensure_legacy_investor_count();

        eprintln!("active_stats = {}", self.active_stats.len());
        // check all active stats.
        for stat_type in &self.active_stats {
            self.ensure_asset_stat(stat_type);
        }
    }
}

/// Create some batches of investors.
fn create_batches(tracker: &mut AssetTracker) -> Vec<Batch> {
    // batches
    tracker.make_batches(vec![
        // (batch_size, sto_buy, claims: Vec<(ClaimType, Option<CountryCode>>)
        (1, 100_000u128, vec![
            (ClaimType::Accredited, None),
            (ClaimType::Affiliate, None),
            (ClaimType::Jurisdiction, Some(CountryCode::US))
        ]),
        (40, 10_000u128, vec![
            (ClaimType::Accredited, None),
            (ClaimType::Jurisdiction, Some(CountryCode::US))
        ]),
        (10, 2_000u128, vec![
            (ClaimType::Accredited, None),
            (ClaimType::Jurisdiction, Some(CountryCode::GB))
        ]),
        (2, 1_000u128, vec![
            (ClaimType::Accredited, None),
            (ClaimType::Jurisdiction, Some(CountryCode::CA))
        ])
    ]).expect("Failed to create batches")
}

#[test]
fn legacy_investor_count() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(legacy_investor_count_with_ext);
}

fn legacy_investor_count_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    // Mint
    tracker.mint(100_000_000);

    // Create investor batches.
    create_batches(&mut tracker);

    tracker.ensure_asset_stats();
}

#[test]
fn multiple_stats() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(multiple_stats_with_ext);
}

fn multiple_stats_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let mut stats = vec![
        StatType { op: StatOpType::Count, claim_issuer: None },
        StatType { op: StatOpType::Balance, claim_issuer: None },
    ];
    let issuers = vec![
        User::new(AccountKeyring::Dave),
    ];
    let claim_types = vec![
        ClaimType::Accredited,
        ClaimType::Affiliate,
        ClaimType::Jurisdiction,
    ];
    // Add issuers.
    for issuer in &issuers {
        for claim_type in &claim_types {
            stats.push(StatType {
                op: StatOpType::Count,
                claim_issuer: Some((*claim_type, issuer.did))
            });
            stats.push(StatType {
                op: StatOpType::Balance,
                claim_issuer: Some((*claim_type, issuer.did))
            });
        }
        tracker.add_issuer(issuer, &claim_types[..]);
    }
    // Active stats.
    tracker.set_active_stats(stats);

    // Mint
    tracker.mint(100_000_000);

    // Create investor batches.
    let _batches = create_batches(&mut tracker);

    tracker.ensure_asset_stats();

    // check some stats.
    tracker.get_claim_count_stats(ClaimType::Jurisdiction, Some(CountryCode::US))
        .into_iter().for_each(|(cal, value)| {
            eprintln!("Jurisdiction::US = ({}, {})", cal, value);
            assert_eq!(cal, value);
        });
    tracker.get_claim_balance_stats(ClaimType::Jurisdiction, Some(CountryCode::US))
        .into_iter().for_each(|(cal, value)| {
            eprintln!("Jurisdiction::US = ({}, {})", cal, value);
            assert_eq!(cal, value);
        });
    // Check a Jurisdiction with no investors (KP).
    tracker.get_claim_count_stats(ClaimType::Jurisdiction, Some(CountryCode::KP))
        .into_iter().for_each(|(cal, value)| {
            eprintln!("Jurisdiction::KP = ({}, {})", cal, value);
            assert_eq!(cal, value);
        });
    tracker.get_claim_balance_stats(ClaimType::Jurisdiction, Some(CountryCode::KP))
        .into_iter().for_each(|(cal, value)| {
            eprintln!("Jurisdiction::KP = ({}, {})", cal, value);
            assert_eq!(cal, value);
        });
}

