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
    asset::AssetType, investor_zkproof_data::v1::InvestorZKProofData, jurisdiction::CountryCode,
    statistics::*, transfer_compliance::*, AccountId, Balance, CddId, Claim, ClaimType, IdentityId,
    InvestorUid, PortfolioId, Scope, ScopeId, Ticker,
};
use sp_arithmetic::Permill;
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

    pub fn scope_id(&self, ticker: Ticker) -> ScopeId {
        InvestorZKProofData::make_scope_id(&ticker.as_slice(), &self.uid())
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

    pub fn fetch_stat_claim(&self, claim_issuer: &(ClaimType, IdentityId)) -> Stat2ndKey {
        let claim = self.claims.get(claim_issuer);
        Stat2ndKey::new_from(&claim_issuer.0, claim.cloned())
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
        self.claims
            .iter()
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
            active_stats.clone().into_iter().collect(),
        ));
        self.active_stats = active_stats;
    }

    pub fn set_transfer_conditions(&mut self, conditions: Vec<TransferCondition>) {
        assert_ok!(Statistics::set_asset_transfer_compliance(
            self.owner_origin(),
            self.asset_scope,
            conditions.clone().into_iter().collect(),
        ));
        self.transfer_conditions = conditions;
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
            }
        }
    }

    pub fn make_stat_claim(
        &self,
        claim_type: ClaimType,
        has: bool,
        jur: Option<CountryCode>,
    ) -> StatClaim {
        let claim = self.make_claim(claim_type, jur);
        StatClaim::new_from(&claim, has).expect("Unsupported ClaimType")
    }

    pub fn set_investors_exempt(&mut self, ids: &[u64], is_exempt: bool) {
        println!("ids = {:?}", ids);
        let investors = ids
            .into_iter()
            .map(|id| self.investor(*id).scope_id(self.asset))
            .collect::<Vec<_>>();
        println!("investors = {:?}", investors);
        for condition in &self.transfer_conditions {
            let exempt_key = condition.get_exempt_key(self.asset_scope);
            println!(" -- exempt={:?}", exempt_key);
            assert_ok!(Statistics::set_entities_exempt(
                self.owner_origin(),
                is_exempt,
                exempt_key,
                investors.clone().into_iter().collect()
            ));
        }
    }

    pub fn add_claim_to_investors(
        &mut self,
        ids: &[u64],
        claim_type: ClaimType,
        jur: Option<CountryCode>,
    ) {
        let claim = self.make_claim(claim_type, jur);

        // build list of (issuer, claim) pairs.
        let claims = self
            .issuers
            .values_mut()
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
        self.investors.values().for_each(|i| {
            println!("investor[{}]: bal={}", i.id, i.balance);
        })
    }

    /// The number of investors with `balance > 0`.
    pub fn active_investor_count(&self) -> u64 {
        self.investors.values().filter(|i| i.balance > 0).count() as u64
    }

    /// Count the number of investors with `balance > 0` and the matching claim.
    pub fn calculate_stat_count(
        &self,
        claim_issuer: Option<(ClaimType, IdentityId)>,
        key2: &Stat2ndKey,
    ) -> u64 {
        if let Some(claim_issuer) = claim_issuer {
            self.investors
                .values()
                .filter(|i| i.balance > 0 && i.fetch_stat_claim(&claim_issuer) == *key2)
                .count() as u64
        } else {
            // Special case, count all investors with a balance.
            self.active_investor_count()
        }
    }

    /// Calculate the balance of all investors the matching claim.
    pub fn calculate_stat_balance(
        &self,
        claim_issuer: Option<(ClaimType, IdentityId)>,
        key2: &Stat2ndKey,
    ) -> Balance {
        let claim_issuer = claim_issuer.expect("Need claim issuer for Balance stats.");
        self.investors
            .values()
            .filter(|i| i.fetch_stat_claim(&claim_issuer) == *key2)
            .map(|i| i.balance)
            .sum()
    }

    pub fn make_investors(&mut self, count: u64) -> Batch {
        let ids = (0..count).map(|_| self.new_investor()).collect();
        Batch { ids }
    }

    pub fn make_batches(
        &mut self,
        batches: Vec<(u64, Balance, Vec<(ClaimType, Option<CountryCode>)>)>,
    ) -> Result<Vec<Batch>, DispatchError> {
        batches
            .into_iter()
            .map(|(size, sto_buy, claims)| self.new_batch(size, sto_buy, claims))
            .collect()
    }

    pub fn new_batch(
        &mut self,
        size: u64,
        sto_buy: Balance,
        claims: Vec<(ClaimType, Option<CountryCode>)>,
    ) -> Result<Batch, DispatchError> {
        println!("Create batch: {:?}", (size, sto_buy, &claims));
        // Create investors for this batch.
        let batch = self.make_investors(size);

        // Add claims.
        for (claim_type, jur) in claims {
            self.add_claim_to_investors(&batch.ids[..], claim_type, jur);
        }

        // Fake STO.
        let sto = batch
            .ids
            .iter()
            .map(|id| (*id, sto_buy))
            .collect::<Vec<_>>();
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

    pub fn fetch_stats_key2(&self, stat_type: &StatType) -> Vec<Stat2ndKey> {
        match stat_type.claim_issuer {
            None => vec![Stat2ndKey::NoClaimStat],
            Some((claim_type, did)) => {
                if let Some(issuer) = self.issuers.get(&did) {
                    issuer
                        .fetch_claims(claim_type)
                        .into_iter()
                        .map(|claim| Stat2ndKey::new_from(&claim_type, Some(claim)))
                        .chain(vec![Stat2ndKey::new_from(&claim_type, None)].into_iter())
                        .collect()
                } else {
                    vec![Stat2ndKey::new_from(&claim_type, None)]
                }
            }
        }
    }

    pub fn get_claim_stats(
        &self,
        op: StatOpType,
        claim_type: ClaimType,
        has: bool,
        jur: Option<CountryCode>,
    ) -> Vec<(u128, u128)> {
        let key2 = if has {
            Stat2ndKey::from(self.make_claim(claim_type, jur))
        } else {
            Stat2ndKey::new_from(&claim_type, None)
        };

        self.issuers
            .values()
            .filter(|issuer| issuer.is_trusted_for(&claim_type))
            .map(|i| Stat1stKey {
                asset: self.asset_scope,
                stat_type: StatType {
                    op,
                    claim_issuer: Some((claim_type, i.issuer.did)),
                },
            })
            .map(|key1| {
                let claim_issuer = key1.stat_type.claim_issuer;
                // Calculate the expected value.
                let cal_value = match op {
                    StatOpType::Count => self.calculate_stat_count(claim_issuer, &key2) as u128,
                    StatOpType::Balance => self.calculate_stat_balance(claim_issuer, &key2),
                };
                // Get stat from pallet.
                let value = Statistics::asset_stats(key1, key2.clone());
                (cal_value, value)
            })
            .collect()
    }

    #[track_caller]
    pub fn ensure_claim_stats(
        &self,
        op: StatOpType,
        claim_type: ClaimType,
        has: bool,
        jur: Option<CountryCode>,
    ) {
        let claim_name = match (has, claim_type, jur) {
            (false, ClaimType::Jurisdiction, _) => "No Jurisdiction".into(),
            (true, ClaimType::Jurisdiction, Some(cc)) => format!("Jurisdiction::{:?}", cc),
            (false, c_type, None) => format!("non-{:?}", c_type),
            (true, c_type, None) => format!("{:?}", c_type),
            _ => {
                unimplemented!(
                    "Unsupported claim stats: has={}, claim={:?}, jur={:?}",
                    has,
                    claim_type,
                    jur
                );
            }
        };
        self.get_claim_stats(op, claim_type, has, jur)
            .into_iter()
            .for_each(|(cal, value)| {
                println!("{:?}: {} = ({}, {})", op, claim_name, cal, value);
                assert_eq!(cal, value);
            });
    }

    #[track_caller]
    pub fn ensure_asset_stat(&self, stat_type: &StatType) {
        let key1 = Stat1stKey {
            asset: self.asset_scope,
            stat_type: *stat_type,
        };
        for key2 in self.fetch_stats_key2(stat_type).iter() {
            let value = Statistics::asset_stats(key1, key2);
            match (stat_type.op, stat_type.claim_issuer) {
                (StatOpType::Count, claim_issuer) => {
                    let cal_value = self.calculate_stat_count(claim_issuer, &key2);
                    println!("Count[{:?}]: cal={:?}, stat={:?}", key2, cal_value, value);
                    assert_eq!(value, cal_value as u128);
                }
                (StatOpType::Balance, None) => {
                    // Per-investor balances are not stored in the stats.
                }
                (StatOpType::Balance, claim_issuer) => {
                    let cal_value = self.calculate_stat_balance(claim_issuer, &key2);
                    println!("Balance[{:?}]: cal={:?}, stat={:?}", key2, cal_value, value);
                    assert_eq!(value, cal_value as u128);
                }
            }
        }
    }

    #[track_caller]
    pub fn ensure_asset_stats(&self) {
        println!("active_stats = {}", self.active_stats.len());
        // check all active stats.
        for stat_type in &self.active_stats {
            self.ensure_asset_stat(stat_type);
        }
    }
}

/// Create some batches of investors.
fn create_batches(tracker: &mut AssetTracker) -> Vec<Batch> {
    // batches
    tracker
        .make_batches(vec![
            // (batch_size, sto_buy, claims: Vec<(ClaimType, Option<CountryCode>>)
            (
                1,
                100_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Affiliate, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::US)),
                ],
            ),
            (
                40,
                10_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::US)),
                ],
            ),
            (
                10,
                2_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::GB)),
                ],
            ),
            (
                2,
                1_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::CA)),
                ],
            ),
        ])
        .expect("Failed to create batches")
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
        StatType {
            op: StatOpType::Count,
            claim_issuer: None,
        },
        StatType {
            op: StatOpType::Balance,
            claim_issuer: None,
        },
    ];
    let issuers = vec![User::new(AccountKeyring::Dave)];
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
                claim_issuer: Some((*claim_type, issuer.did)),
            });
            stats.push(StatType {
                op: StatOpType::Balance,
                claim_issuer: Some((*claim_type, issuer.did)),
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
    tracker.ensure_claim_stats(
        StatOpType::Count,
        ClaimType::Jurisdiction,
        true,
        Some(CountryCode::US),
    );
    tracker.ensure_claim_stats(
        StatOpType::Balance,
        ClaimType::Jurisdiction,
        true,
        Some(CountryCode::US),
    );
    // Check a Jurisdiction with no investors (KP).
    tracker.ensure_claim_stats(
        StatOpType::Count,
        ClaimType::Jurisdiction,
        true,
        Some(CountryCode::KP),
    );
    tracker.ensure_claim_stats(
        StatOpType::Balance,
        ClaimType::Jurisdiction,
        true,
        Some(CountryCode::KP),
    );

    // Check Accredited.
    tracker.ensure_claim_stats(StatOpType::Count, ClaimType::Accredited, true, None);
    tracker.ensure_claim_stats(StatOpType::Balance, ClaimType::Accredited, true, None);

    // Check Affiliate.
    tracker.ensure_claim_stats(StatOpType::Count, ClaimType::Affiliate, true, None);
    tracker.ensure_claim_stats(StatOpType::Balance, ClaimType::Affiliate, true, None);
    // Check non-Affiliate.
    tracker.ensure_claim_stats(StatOpType::Count, ClaimType::Affiliate, false, None);
    tracker.ensure_claim_stats(StatOpType::Balance, ClaimType::Affiliate, false, None);
}

#[test]
fn max_investor_rule() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(max_investor_rule_with_ext);
}

fn max_investor_rule_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let stats = vec![StatType {
        op: StatOpType::Count,
        claim_issuer: None,
    }];
    // Active stats.
    tracker.set_active_stats(stats);

    // Mint
    tracker.mint(100_000_000);

    // Create investor batches.
    let _batches = create_batches(&mut tracker);

    tracker.ensure_asset_stats();

    // Set max investor count rule to `max == active_investor_count`.
    let cur_count = tracker.active_investor_count();
    tracker.set_transfer_conditions(vec![TransferCondition::MaxInvestorCount(cur_count)]);

    // Try adding another investor.
    let id = tracker.new_investor(); // No balance yet.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();
}

#[test]
fn max_investor_ownership_rule() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(max_investor_ownership_rule_with_ext);
}

fn max_investor_ownership_rule_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let stats = vec![StatType {
        op: StatOpType::Balance,
        claim_issuer: None,
    }];
    // Active stats.
    tracker.set_active_stats(stats);

    // Set max ownership to 25%.
    let p25 = HashablePermill(Permill::from_rational(25u32, 100u32));
    tracker.set_transfer_conditions(vec![TransferCondition::MaxInvestorOwnership(p25)]);

    // Mint is not restricted by transfer rules.
    tracker.mint(100_000);

    tracker.ensure_asset_stats();

    // Add a new investor and transfer less then 25%.
    let id = tracker.new_investor(); // No balance yet.
    tracker.do_valid_transfer(tracker.owner_id, id, 10_000); // 10%

    tracker.ensure_asset_stats();

    // Try transfer more so they would have >25%.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 16_000);

    tracker.ensure_asset_stats();
}

#[test]
fn claim_count_rule() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(claim_count_rule_with_ext);
}

fn claim_count_rule_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let issuer = User::new(AccountKeyring::Dave);
    let claim_types = vec![ClaimType::Accredited];
    // Add issuer.
    tracker.add_issuer(&issuer, &claim_types[..]);

    // Make the owner Accredited.
    tracker.add_claim_to_investors(&[tracker.owner_id], ClaimType::Accredited, None);

    // Active stats.
    let stats = vec![StatType {
        op: StatOpType::Count,
        claim_issuer: Some((ClaimType::Accredited, issuer.did)),
    }];
    tracker.set_active_stats(stats);

    // Set transfer conditions.  max=40 Non-Accredited.
    let no_claim = tracker.make_stat_claim(ClaimType::Accredited, false, None);
    tracker.set_transfer_conditions(vec![TransferCondition::ClaimCount(
        no_claim,
        issuer.did,
        0,
        Some(40),
    )]);

    // Mint
    tracker.mint(100_000_000);

    // Create some investor batches.  40 - Non-Accredited and 10 Accredited.
    tracker
        .make_batches(vec![
            // (batch_size, sto_buy, claims: Vec<(ClaimType, Option<CountryCode>>)
            (
                40,
                10_000u128,
                vec![(ClaimType::Jurisdiction, Some(CountryCode::US))],
            ),
            (
                10,
                2_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::GB)),
                ],
            ),
        ])
        .expect("Failed to create batches");

    tracker.ensure_asset_stats();

    // Create a new Non-Accredited investor.
    let id = tracker.new_investor(); // No balance yet.

    // Try transfer some tokens to them.  Should fail.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();

    // Exempt the receiver from the transfer rules.
    tracker.set_investors_exempt(&[id], true);

    // Retry transfer.
    // Should still fail since the sender needs to be exempt for Count rules.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();

    // Exempt the sender from the transfer rules.
    tracker.set_investors_exempt(&[tracker.owner_id], true);

    // Retry transfer.
    // Should pass.
    tracker.do_valid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();
}

#[test]
fn jurisdiction_count_rule() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(jurisdiction_count_rule_with_ext);
}

fn jurisdiction_count_rule_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let issuer = User::new(AccountKeyring::Dave);
    let claim_type = ClaimType::Jurisdiction;
    // Add issuer.
    tracker.add_issuer(&issuer, &[claim_type]);

    // Active stats.
    let stats = vec![StatType {
        op: StatOpType::Count,
        claim_issuer: Some((claim_type, issuer.did)),
    }];
    tracker.set_active_stats(stats);

    // Set transfer conditions.  max=10 investors in Jurisdiction GB.
    let claim = tracker.make_stat_claim(claim_type, true, Some(CountryCode::GB));
    tracker.set_transfer_conditions(vec![TransferCondition::ClaimCount(
        claim,
        issuer.did,
        0,
        Some(10),
    )]);

    // Mint
    tracker.mint(100_000_000);

    // Create some investor batches.  40 - US and 10 GB.
    tracker
        .make_batches(vec![
            // (batch_size, sto_buy, claims: Vec<(ClaimType, Option<CountryCode>>)
            (
                40,
                10_000u128,
                vec![(ClaimType::Jurisdiction, Some(CountryCode::US))],
            ),
            (
                10,
                2_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::GB)),
                ],
            ),
        ])
        .expect("Failed to create batches");

    tracker.ensure_asset_stats();

    // Create a new GB investor.
    let id = tracker.new_investor(); // No balance yet.
    tracker.add_claim_to_investors(&[id], claim_type, Some(CountryCode::GB));

    // Try transfer some tokens to them.  Should fail.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();

    // Exempt the receiver from the transfer rules.
    tracker.set_investors_exempt(&[id], true);

    // Retry transfer.
    // Should still fail since the sender needs to be exempt for Count rules.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();

    // Exempt the sender from the transfer rules.
    tracker.set_investors_exempt(&[tracker.owner_id], true);

    // Retry transfer.
    // Should pass.
    tracker.do_valid_transfer(tracker.owner_id, id, 1_000);

    tracker.ensure_asset_stats();
}

#[test]
fn jurisdiction_ownership_rule() {
    ExtBuilder::default()
        .cdd_providers(vec![CDD_PROVIDER.to_account_id()])
        .build()
        .execute_with(jurisdiction_ownership_rule_with_ext);
}

fn jurisdiction_ownership_rule_with_ext() {
    // Create an asset.
    let mut tracker = AssetTracker::new();

    let issuer = User::new(AccountKeyring::Dave);
    let claim_type = ClaimType::Jurisdiction;
    // Add issuer.
    tracker.add_issuer(&issuer, &[claim_type]);

    // Active stats.
    let stats = vec![StatType {
        op: StatOpType::Balance,
        claim_issuer: Some((claim_type, issuer.did)),
    }];
    tracker.set_active_stats(stats);

    // Set transfer conditions.  max=10 investors in Jurisdiction GB.
    let claim = tracker.make_stat_claim(claim_type, true, Some(CountryCode::GB));
    let p0 = HashablePermill(Permill::from_rational(0u32, 100u32));
    let p25 = HashablePermill(Permill::from_rational(25u32, 100u32));
    tracker.set_transfer_conditions(vec![TransferCondition::ClaimOwnership(
        claim, issuer.did, p0, p25,
    )]);

    // Mint
    tracker.mint(1_000_000);

    // Create some investor batches.  40 - US and 10 GB.
    tracker
        .make_batches(vec![
            // (batch_size, sto_buy, claims: Vec<(ClaimType, Option<CountryCode>>)
            (
                40,
                10_000u128,
                vec![(ClaimType::Jurisdiction, Some(CountryCode::US))],
            ),
            (
                10,
                2_000u128,
                vec![
                    (ClaimType::Accredited, None),
                    (ClaimType::Jurisdiction, Some(CountryCode::GB)),
                ],
            ),
        ])
        .expect("Failed to create batches");

    tracker.ensure_asset_stats();

    // Create a new GB investor.
    let id = tracker.new_investor(); // No balance yet.
    tracker.add_claim_to_investors(&[id], claim_type, Some(CountryCode::GB));

    // Try transfer more then 25% of the tokens to them.  Should fail.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 260_000);

    tracker.ensure_asset_stats();

    // Exempt the sender from the transfer rules.
    tracker.set_investors_exempt(&[tracker.owner_id], true);

    // Retry transfer.
    // Should still fail since the receiver needs to be exempt for Count rules.
    tracker.ensure_invalid_transfer(tracker.owner_id, id, 260_000);

    tracker.ensure_asset_stats();

    // Exempt the receiver from the transfer rules.
    tracker.set_investors_exempt(&[id], true);

    // Retry transfer.
    // Should pass.
    tracker.do_valid_transfer(tracker.owner_id, id, 260_000);

    tracker.ensure_asset_stats();
}
