use crate::TestStorage;
use confidential_identity::mocked::make_investor_uid as make_investor_uid_v2;
use pallet_asset::{self as asset, TickerRegistrationConfig};
use pallet_balances as balances;
use pallet_bridge::BridgeTx;
use pallet_committee as committee;
use pallet_group as group;
use pallet_identity as identity;
use pallet_pips as pips;
use polymesh_common_utilities::{
    constants::currency::POLY, protocol_fee::ProtocolOp, SystematicIssuers, GC_DID,
};
use polymesh_primitives::{
    cdd_id::InvestorUid, identity_id::GenesisIdentityRecord, AccountId, Balance, Identity,
    IdentityId, PosRatio, Signatory,
};
use sp_io::TestExternalities;
use sp_runtime::{Perbill, Storage};
use sp_std::{cell::RefCell, convert::From, iter};
use test_client::AccountKeyring;

/// A prime number fee to test the split between multiple recipients.
pub const PROTOCOL_OP_BASE_FEE: u128 = 41;

pub const COOL_OFF_PERIOD: u32 = 100;
const DEFAULT_BRIGDE_LIMIT: u128 = 1_000_000_000_000_000;
const DEFAULT_BRIDGE_TIMELOCK: u32 = 3;

struct BuilderVoteThreshold {
    pub numerator: u32,
    pub denominator: u32,
}

impl Default for BuilderVoteThreshold {
    fn default() -> Self {
        BuilderVoteThreshold {
            numerator: 2,
            denominator: 3,
        }
    }
}

#[derive(Clone)]
pub struct MockProtocolBaseFees(pub Vec<(ProtocolOp, u128)>);

impl Default for MockProtocolBaseFees {
    fn default() -> Self {
        let ops = vec![
            ProtocolOp::AssetRegisterTicker,
            ProtocolOp::AssetIssue,
            ProtocolOp::AssetAddDocument,
            ProtocolOp::AssetCreateAsset,
            ProtocolOp::DividendNew,
            ProtocolOp::ComplianceManagerAddComplianceRequirement,
            ProtocolOp::IdentityRegisterDid,
            ProtocolOp::IdentityCddRegisterDid,
            ProtocolOp::IdentityAddClaim,
            ProtocolOp::IdentitySetPrimaryKey,
            ProtocolOp::IdentityAddSecondaryKeysWithAuthorization,
            ProtocolOp::PipsPropose,
            ProtocolOp::VotingAddBallot,
            ProtocolOp::ContractsPutCode,
        ];
        let fees = ops
            .into_iter()
            .zip(iter::repeat(PROTOCOL_OP_BASE_FEE))
            .collect();
        MockProtocolBaseFees(fees)
    }
}

#[derive(Default)]
struct BridgeConfig {
    /// Complete TXs
    pub complete_txs: Vec<BridgeTx<AccountId>>,
    /// Bridge admin key. See `Bridge` documentation for details.
    pub admin: Option<AccountId>,
    /// signers of the controller multisig account.
    pub signers: Vec<Signatory<AccountId>>,
    /// # of signers required for controller multisig account.
    pub signatures_required: u64,
    /// Bridge limit.
    pub limit: Option<u128>,
    /// Bridge timelock.
    pub timelock: Option<u32>,
}

#[derive(Default)]
pub struct ExtBuilder {
    /// Minimum weight for the extrinsic (see `weight_to_fee` below).
    extrinsic_base_weight: u64,
    /// The transaction fee per byte.
    /// Transactions with bigger payloads will have a bigger `len_fee`.
    /// This is calculated as `transaction_byte_fee * tx.len()`.
    transaction_byte_fee: u128,
    /// Contributes to the `weight_fee`, indicating the compute requirements of a transaction.
    /// A more resource-intensive transaction will have a higher `weight_fee`.
    weight_to_fee: u128,
    /// Scaling factor for initial balances on genesis.
    balance_factor: u128,
    /// When `false`, no balances will be initialized on genesis.
    monied: bool,
    vesting: bool,
    /// CDD Service provides. Their DID will be generated.
    cdd_providers: Vec<AccountId>,
    /// Governance committee members. Their DID will be generated.
    governance_committee_members: Vec<AccountId>,
    governance_committee_vote_threshold: BuilderVoteThreshold,
    /// Regular users. Their DID will be generated.
    regular_users: Vec<Identity<AccountId>>,

    protocol_base_fees: MockProtocolBaseFees,
    protocol_coefficient: PosRatio,
    /// Percentage fee share of a network (treasury + validators) in instantiation fee
    /// of a smart extension.
    network_fee_share: Perbill,
    /// Maximum number of transfer manager an asset can have.
    max_no_of_tm_allowed: u32,
    /// The minimum duration for a checkpoint period, in seconds.
    min_checkpoint_duration: u64,
    adjust: Option<Box<dyn FnOnce(&mut Storage)>>,
    /*
    /// Enable `put_code` in contracts pallet
    enable_contracts_put_code: bool,
    */
    /// Bridge configuration
    bridge: BridgeConfig,
    itn_rewards: Vec<(AccountId, Balance)>,
}

thread_local! {
    pub static EXTRINSIC_BASE_WEIGHT: RefCell<u64> = RefCell::new(0);
    pub static TRANSACTION_BYTE_FEE: RefCell<u128> = RefCell::new(0);
    pub static WEIGHT_TO_FEE: RefCell<u128> = RefCell::new(0);
    pub static NETWORK_FEE_SHARE: RefCell<Perbill> = RefCell::new(Perbill::from_percent(0));
    pub static MAX_NO_OF_TM_ALLOWED: RefCell<u32> = RefCell::new(0);
}

impl ExtBuilder {
    /// Sets the minimum weight for the extrinsic (see also `weight_fee`).
    pub fn base_weight(mut self, extrinsic_base_weight: u64) -> Self {
        self.extrinsic_base_weight = extrinsic_base_weight;
        self
    }

    /// Sets the fee per each byte in a transaction.
    /// The full byte fee is defined as: `transaction_byte_fee * tx.len()`.
    pub fn byte_fee(mut self, transaction_byte_fee: u128) -> Self {
        self.transaction_byte_fee = transaction_byte_fee;
        self
    }

    /// Sets the fee to charge per weight.
    /// A more demanding computation will have a higher fee for its weight.
    pub fn weight_fee(mut self, weight_to_fee: u128) -> Self {
        self.weight_to_fee = weight_to_fee;
        self
    }

    /// Sets parameters for transaction fees
    /// (`extrinsic_base_weight`, `transaction_byte_fee`, and `weight_to_fee`).
    /// See the corresponding methods for more details.
    pub fn transaction_fees(
        self,
        extrinsic_base_weight: u64,
        transaction_byte_fee: u128,
        weight_to_fee: u128,
    ) -> Self {
        self.base_weight(extrinsic_base_weight)
            .byte_fee(transaction_byte_fee)
            .weight_fee(weight_to_fee)
    }

    /// Set the scaling factor used for initial balances on genesis to `factor`.
    /// The default is `0`.
    pub fn balance_factor(mut self, factor: u128) -> Self {
        self.balance_factor = factor;
        self
    }

    /// Set whether balances should be initialized on genesis.
    /// This also does `.balance_factor(1)` when it is `0`.
    /// The default is `false`.
    pub fn monied(mut self, monied: bool) -> Self {
        self.monied = monied;
        if self.balance_factor == 0 {
            self.balance_factor = 1;
        }
        self
    }

    pub fn governance_committee(mut self, members: Vec<AccountId>) -> Self {
        self.governance_committee_members = members;
        self.governance_committee_members.sort();
        self
    }

    pub fn governance_committee_vote_threshold(mut self, threshold: (u32, u32)) -> Self {
        self.governance_committee_vote_threshold = BuilderVoteThreshold {
            numerator: threshold.0,
            denominator: threshold.1,
        };
        self
    }

    /// It sets `providers` as CDD providers.
    pub fn cdd_providers(mut self, providers: Vec<AccountId>) -> Self {
        self.cdd_providers = providers;
        self.cdd_providers.sort();
        self
    }

    /// Adds DID to `users` accounts.
    pub fn add_regular_users(mut self, users: &[Identity<AccountId>]) -> Self {
        self.regular_users.extend_from_slice(users);
        self
    }

    pub fn add_regular_users_from_accounts(mut self, accounts: &[AccountId]) -> Self {
        let identities = accounts
            .iter()
            .map(|acc: &AccountId| Identity::new(acc.clone()))
            .collect::<Vec<_>>();

        self.regular_users.extend(identities);
        self
    }

    /// Set maximum of tms allowed for an asset
    pub fn set_max_tms_allowed(mut self, tm_count: u32) -> Self {
        self.max_no_of_tm_allowed = tm_count;
        self
    }

    pub fn set_protocol_base_fees(mut self, fees: MockProtocolBaseFees) -> Self {
        self.protocol_base_fees = fees;
        self
    }

    pub fn set_protocol_coefficient(mut self, coefficient: (u32, u32)) -> Self {
        self.protocol_coefficient = PosRatio::from(coefficient);
        self
    }

    /// Assigning the fee share in the instantiation fee
    pub fn network_fee_share(mut self, share: Perbill) -> Self {
        self.network_fee_share = share;
        self
    }

    /// Provide a closure `with` to run on the storage for final adjustments.
    pub fn adjust(mut self, with: Box<dyn FnOnce(&mut Storage)>) -> Self {
        self.adjust = Some(with);
        self
    }

    /*
    /// Enables `contracts::put_code` at genesis if `enable` is `true`.
    /// By default, it is disabled.
    pub fn set_contracts_put_code(mut self, enable: bool) -> Self {
        self.enable_contracts_put_code = enable;
        self
    }
    */

    pub fn set_bridge_complete_tx(mut self, txs: Vec<BridgeTx<AccountId>>) -> Self {
        self.bridge.complete_txs = txs;
        self
    }

    /// Sets the bridge controller.
    pub fn set_bridge_controller(
        mut self,
        admin: AccountId,
        signers: Vec<AccountId>,
        signatures_required: u64,
    ) -> Self {
        self.bridge.admin = Some(admin);
        self.bridge.signers = signers
            .into_iter()
            .map(Signatory::Account)
            .collect::<Vec<_>>();
        self.bridge.signatures_required = signatures_required;
        self
    }

    pub fn set_bridge_limit(mut self, limit: u128) -> Self {
        self.bridge.limit = Some(limit);
        self
    }

    pub fn set_bridge_timelock(mut self, timelock: u32) -> Self {
        self.bridge.timelock = Some(timelock);
        self
    }

    pub fn set_itn_rewards(mut self, itn_rewards: Vec<(AccountId, Balance)>) -> Self {
        self.itn_rewards = itn_rewards;
        self
    }

    fn set_associated_consts(&self) {
        EXTRINSIC_BASE_WEIGHT.with(|v| *v.borrow_mut() = self.extrinsic_base_weight);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
        WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
        NETWORK_FEE_SHARE.with(|v| *v.borrow_mut() = self.network_fee_share);
        MAX_NO_OF_TM_ALLOWED.with(|v| *v.borrow_mut() = self.max_no_of_tm_allowed);
    }

    fn make_balances(&self) -> Vec<(AccountId, u128)> {
        if self.monied {
            vec![
                (
                    AccountKeyring::Alice.to_account_id(),
                    1_000 * POLY * self.balance_factor,
                ),
                (
                    AccountKeyring::Bob.to_account_id(),
                    2_000 * POLY * self.balance_factor,
                ),
                (
                    AccountKeyring::Charlie.to_account_id(),
                    3_000 * POLY * self.balance_factor,
                ),
                (
                    AccountKeyring::Dave.to_account_id(),
                    4_000 * POLY * self.balance_factor,
                ),
                // CDD Accounts
                (AccountKeyring::Eve.to_account_id(), 1_000_000),
                (AccountKeyring::Ferdie.to_account_id(), 1_000_000),
            ]
        } else {
            vec![]
        }
    }

    /// Generates a mapping between DID and Identity info.
    ///
    /// DIDs are generated sequentially from `offset`.
    fn make_identities(
        identities: impl Iterator<Item = AccountId>,
        offset: usize,
        issuers: Vec<IdentityId>,
    ) -> Vec<GenesisIdentityRecord<AccountId>> {
        identities
            .enumerate()
            .map(|(idx, primary_key)| {
                let did_index = (idx + offset + 1) as u128;
                let did = IdentityId::from(did_index);
                let investor: InvestorUid = make_investor_uid_v2(did.as_bytes()).into();

                GenesisIdentityRecord {
                    primary_key,
                    issuers: issuers.clone(),
                    did,
                    investor,
                    ..Default::default()
                }
            })
            .collect()
    }

    fn make_account_did_map(
        accounts: Vec<AccountId>,
        did_maker: impl Fn(usize) -> IdentityId,
    ) -> Vec<(AccountId, IdentityId)> {
        accounts
            .into_iter()
            .enumerate()
            .map(|(idx, acc)| (acc, did_maker(idx)))
            .collect()
    }

    fn build_bridge(&self, storage: &mut Storage) {
        if let Some(creator) = &self.bridge.admin {
            pallet_bridge::GenesisConfig::<TestStorage> {
                creator: creator.clone(),
                signers: self.bridge.signers.clone(),
                signatures_required: self.bridge.signatures_required,
                bridge_limit: (self.bridge.limit.unwrap_or(DEFAULT_BRIGDE_LIMIT), 1),
                timelock: self
                    .bridge
                    .timelock
                    .unwrap_or(DEFAULT_BRIDGE_TIMELOCK)
                    .into(),
                ..Default::default()
            }
            .assimilate_storage(storage)
            .unwrap();
        }
    }

    fn build_identity_genesis(
        &self,
        storage: &mut Storage,
        identities: Vec<GenesisIdentityRecord<AccountId>>,
    ) {
        // New identities are just `system users` + `regular users`.
        identity::GenesisConfig::<TestStorage> {
            identities,
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    fn build_balances_genesis(&self, storage: &mut Storage) {
        balances::GenesisConfig::<TestStorage> {
            balances: self.make_balances(),
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    fn build_asset_genesis(&self, storage: &mut Storage) {
        let ticker_registration_config = TickerRegistrationConfig {
            max_ticker_length: 8,
            registration_length: Some(10000),
        };
        asset::GenesisConfig::<TestStorage> {
            /*
            versions: vec![
                (SmartExtensionType::TransferManager, 5000),
                (SmartExtensionType::Offerings, 5000),
                (SmartExtensionType::SmartWallet, 5000),
            ],
            */
            classic_migration_tickers: vec![],
            classic_migration_contract_did: IdentityId::from(1),
            classic_migration_tconfig: ticker_registration_config.clone(),
            ticker_registration_config,
            reserved_country_currency_codes: vec![],
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    /// For each `cdd_providers`:
    ///     1. A new `IdentityId` is generated (from 1 to n),
    ///     2. CDD provider's account key is linked to its new Identity ID.
    ///     3. That Identity ID is added as member of CDD provider group.
    fn build_cdd_providers_genesis(
        &self,
        storage: &mut Storage,
        identities: &[GenesisIdentityRecord<AccountId>],
    ) {
        let mut cdd_ids = identities
            .iter()
            .map(|gen_id| gen_id.did)
            .collect::<Vec<_>>();
        cdd_ids.push(GC_DID);
        cdd_ids.sort();

        group::GenesisConfig::<TestStorage, group::Instance2> {
            active_members_limit: u32::MAX,
            active_members: cdd_ids,
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    fn build_committee_genesis(
        &self,
        storage: &mut Storage,
        identities: &[GenesisIdentityRecord<AccountId>],
    ) {
        let mut gc_ids = identities
            .iter()
            .map(|gen_id| gen_id.did)
            .collect::<Vec<_>>();
        gc_ids.sort();

        group::GenesisConfig::<TestStorage, group::Instance1> {
            active_members_limit: u32::MAX,
            active_members: gc_ids.clone(),
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap();

        committee::GenesisConfig::<TestStorage, committee::Instance1> {
            members: gc_ids,
            vote_threshold: (
                self.governance_committee_vote_threshold.numerator,
                self.governance_committee_vote_threshold.denominator,
            ),
            release_coordinator: IdentityId::from(999),
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    fn build_protocol_fee_genesis(&self, storage: &mut Storage) {
        pallet_protocol_fee::GenesisConfig {
            base_fees: self.protocol_base_fees.0.clone(),
            coefficient: self.protocol_coefficient,
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    fn build_pips_genesis(&self, storage: &mut Storage) {
        pips::GenesisConfig::<TestStorage> {
            prune_historical_pips: false,
            min_proposal_deposit: 50,
            default_enactment_period: 100,
            max_pip_skip_count: 1,
            active_pip_limit: 5,
            pending_pip_expiry: <_>::default(),
        }
        .assimilate_storage(storage)
        .unwrap();
    }

    /*
    fn build_contracts_genesis(&self, storage: &mut Storage) {
        polymesh_contracts::GenesisConfig {
            enable_put_code: self.enable_contracts_put_code,
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap();
    }
    */

    fn build_bridge_genesis(&self, storage: &mut Storage) {
        pallet_bridge::GenesisConfig::<TestStorage> {
            complete_txs: self.bridge.complete_txs.clone(),
            ..Default::default()
        }
        .assimilate_storage(storage)
        .unwrap()
    }

    fn build_rewards_genesis(&self, storage: &mut Storage) {
        pallet_rewards::GenesisConfig::<TestStorage> {
            itn_rewards: self.itn_rewards.clone(),
        }
        .assimilate_storage(storage)
        .unwrap()
    }

    /// Create externalities.
    pub fn build(self) -> TestExternalities {
        self.set_associated_consts();

        // Regular users should intersect neither with CDD providers nor with GC members.
        assert!(!self
            .regular_users
            .iter()
            .any(|id| self.cdd_providers.contains(&id.primary_key)
                || self.governance_committee_members.contains(&id.primary_key)));

        // System identities.
        let cdd_identities = Self::make_identities(self.cdd_providers.iter().cloned(), 0, vec![]);
        let gc_only_accs = self
            .governance_committee_members
            .iter()
            .filter(|acc| !self.cdd_providers.contains(acc))
            .cloned()
            .collect::<Vec<_>>();
        let gc_only_identities =
            Self::make_identities(gc_only_accs.iter().cloned(), cdd_identities.len(), vec![]);
        let gc_and_cdd_identities = cdd_identities.iter().filter(|gen_id| {
            self.governance_committee_members
                .contains(&gen_id.primary_key)
        });
        let gc_full_identities = gc_only_identities
            .iter()
            .chain(gc_and_cdd_identities)
            .cloned()
            .collect::<Vec<_>>();

        //  User identities.
        let issuer_did = cdd_identities
            .iter()
            .map(|gen_id| gen_id.did)
            .next()
            .unwrap_or(SystematicIssuers::CDDProvider.as_id());
        let regular_accounts = self.regular_users.iter().map(|id| id.primary_key.clone());

        // Create regular user identities + .
        let mut user_identities = Self::make_identities(
            regular_accounts,
            cdd_identities.len() + gc_only_identities.len(),
            vec![issuer_did],
        );
        // Add secondary keys (and permissions) to new identites.
        for user_id in user_identities.iter_mut() {
            if let Some(user) = self
                .regular_users
                .iter()
                .find(|ru| ru.primary_key == user_id.primary_key)
            {
                user_id.secondary_keys = user.secondary_keys.clone();
            }
        }

        let identities = cdd_identities
            .iter()
            .chain(gc_only_identities.iter())
            .chain(user_identities.iter())
            .cloned()
            .collect();

        // Create storage and assimilate each genesis.
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestStorage>()
            .expect("TestStorage cannot build its own storage");

        self.build_identity_genesis(&mut storage, identities);
        self.build_balances_genesis(&mut storage);
        self.build_asset_genesis(&mut storage);
        self.build_cdd_providers_genesis(&mut storage, cdd_identities.as_slice());
        self.build_committee_genesis(&mut storage, gc_full_identities.as_slice());
        self.build_protocol_fee_genesis(&mut storage);
        self.build_pips_genesis(&mut storage);
        //self.build_contracts_genesis(&mut storage);
        self.build_bridge_genesis(&mut storage);
        self.build_rewards_genesis(&mut storage);

        self.build_bridge(&mut storage);

        if let Some(adjust) = self.adjust {
            adjust(&mut storage);
        }

        sp_io::TestExternalities::new(storage)
    }
}
