use crate::TestStorage;

use polymesh_common_utilities::{protocol_fee::ProtocolOp, traits::identity::LinkedKeyInfo};
use polymesh_primitives::{AccountKey, Identity, IdentityId, PosRatio};

use pallet_asset::{self as asset, TickerRegistrationConfig};
use pallet_balances as balances;
use pallet_committee as committee;
use pallet_group as group;
use pallet_identity as identity;
use pallet_pips as pips;

use sp_core::sr25519::Public;
use sp_io::TestExternalities;
use test_client::AccountKeyring;

use std::{cell::RefCell, convert::From, iter};

/// A prime number fee to test the split between multiple recipients.
pub const PROTOCOL_OP_BASE_FEE: u128 = 41;

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

pub struct MockProtocolBaseFees(pub Vec<(ProtocolOp, u128)>);

impl Default for MockProtocolBaseFees {
    fn default() -> Self {
        let ops = vec![
            ProtocolOp::AssetRegisterTicker,
            ProtocolOp::AssetIssue,
            ProtocolOp::AssetAddDocument,
            ProtocolOp::AssetCreateAsset,
            ProtocolOp::DividendNew,
            ProtocolOp::ComplianceManagerAddActiveRule,
            ProtocolOp::IdentityRegisterDid,
            ProtocolOp::IdentityCddRegisterDid,
            ProtocolOp::IdentityAddClaim,
            ProtocolOp::IdentitySetMasterKey,
            ProtocolOp::IdentityAddSigningItemsWithAuthorization,
            ProtocolOp::PipsPropose,
            ProtocolOp::VotingAddBallot,
        ];
        let fees = ops
            .into_iter()
            .zip(iter::repeat(PROTOCOL_OP_BASE_FEE))
            .collect();
        MockProtocolBaseFees(fees)
    }
}

#[derive(Default)]
pub struct ExtBuilder {
    transaction_base_fee: u128,
    transaction_byte_fee: u128,
    weight_to_fee: u128,
    existential_deposit: u128,
    monied: bool,
    vesting: bool,
    cdd_providers: Vec<Public>,
    governance_committee_members: Vec<Public>,
    governance_committee_vote_threshold: BuilderVoteThreshold,
    protocol_base_fees: MockProtocolBaseFees,
    protocol_coefficient: PosRatio,
}

thread_local! {
    static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(0);
    static TRANSACTION_BASE_FEE: RefCell<u128> = RefCell::new(0);
    static TRANSACTION_BYTE_FEE: RefCell<u128> = RefCell::new(1);
    static WEIGHT_TO_FEE: RefCell<u128> = RefCell::new(1);
}

impl ExtBuilder {
    pub fn transaction_fees(mut self, base_fee: u128, byte_fee: u128, weight_fee: u128) -> Self {
        self.transaction_base_fee = base_fee;
        self.transaction_byte_fee = byte_fee;
        self.weight_to_fee = weight_fee;
        self
    }

    pub fn existential_deposit(mut self, existential_deposit: u128) -> Self {
        self.existential_deposit = existential_deposit;
        self
    }

    pub fn monied(mut self, monied: bool) -> Self {
        self.monied = monied;
        if self.existential_deposit == 0 {
            self.existential_deposit = 1;
        }
        self
    }

    pub fn governance_committee(mut self, members: Vec<Public>) -> Self {
        self.governance_committee_members = members;
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
    pub fn cdd_providers(mut self, providers: Vec<Public>) -> Self {
        self.cdd_providers = providers;
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

    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        TRANSACTION_BASE_FEE.with(|v| *v.borrow_mut() = self.transaction_base_fee);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
        WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
    }

    fn make_balances(&self) -> Vec<(Public, u128)> {
        if self.monied {
            vec![
                (
                    AccountKeyring::Alice.public(),
                    1_000 * self.existential_deposit,
                ),
                (
                    AccountKeyring::Bob.public(),
                    2_000 * self.existential_deposit,
                ),
                (
                    AccountKeyring::Charlie.public(),
                    3_000 * self.existential_deposit,
                ),
                (
                    AccountKeyring::Dave.public(),
                    4_000 * self.existential_deposit,
                ),
                // CDD Accounts
                (AccountKeyring::Eve.public(), 1_000_000),
                (AccountKeyring::Ferdie.public(), 1_000_000),
            ]
        } else {
            vec![]
        }
    }

    /// It generates, based on CDD providers, a pair of vectors whose contain:
    ///  - mapping between DID and Identity info.
    ///  - mapping between an account key and its DID.
    /// Please note that generated DIDs start from 1.
    fn make_identities(
        accounts: &[Public],
    ) -> (
        Vec<(IdentityId, Identity)>,
        Vec<(AccountKey, LinkedKeyInfo)>,
    ) {
        let keys = accounts
            .iter()
            .map(|p| AccountKey::from(p.clone().0))
            .collect::<Vec<_>>();

        let identities = keys
            .iter()
            .enumerate()
            .map(|(idx, key)| {
                (
                    IdentityId::from((idx + 1) as u128),
                    Identity::from(key.clone()),
                )
            })
            .collect::<Vec<_>>();
        let key_links = keys
            .into_iter()
            .enumerate()
            .map(|(idx, key)| {
                (
                    key,
                    LinkedKeyInfo::Unique(IdentityId::from((idx + 1) as u128)),
                )
            })
            .collect::<Vec<_>>();

        (identities, key_links)
    }

    /// Create externalities.
    ///
    /// For each `cdd_providers`:
    ///     1. A new `IdentityId` is generated (from 1 to n),
    ///     2. CDD provider's account key is linked to its new Identity ID.
    ///     3. That Identity ID is added as member of CDD provider group.
    pub fn build(self) -> TestExternalities {
        let mut storage = frame_system::GenesisConfig::default()
            .build_storage::<TestStorage>()
            .unwrap();

        let _root = AccountKeyring::Alice.public();

        // Create Identitys.
        let mut system_accounts = self
            .cdd_providers
            .iter()
            .chain(self.governance_committee_members.iter())
            .cloned()
            .collect::<Vec<_>>();
        system_accounts.sort();
        system_accounts.dedup();

        let (system_identities, system_links) = Self::make_identities(system_accounts.as_slice());

        // Identity genesis.
        identity::GenesisConfig::<TestStorage> {
            did_records: system_identities.clone(),
            key_to_identity_ids: system_links,
            identities: vec![],
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // Balances genesis.
        balances::GenesisConfig::<TestStorage> {
            balances: self.make_balances(),
            identity_balances: vec![],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // Asset genesis.
        asset::GenesisConfig::<TestStorage> {
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 8,
                registration_length: Some(10000),
            },
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // CDD Service providers.
        let cdd_ids = self
            .cdd_providers
            .iter()
            .map(|key| {
                let (id, _) = system_identities
                    .iter()
                    .find(|(_id, info)| info.master_key == *key)
                    .unwrap();
                id
            })
            .cloned()
            .collect::<Vec<_>>();

        group::GenesisConfig::<TestStorage, group::Instance2> {
            active_members: cdd_ids,
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // Committee
        let gc_ids = self
            .governance_committee_members
            .iter()
            .map(|key| {
                let (id, _) = system_identities
                    .iter()
                    .find(|(_id, info)| info.master_key == *key)
                    .unwrap();
                id
            })
            .cloned()
            .collect::<Vec<_>>();

        group::GenesisConfig::<TestStorage, group::Instance1> {
            active_members: gc_ids.clone(),
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
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
        .assimilate_storage(&mut storage)
        .unwrap();

        pallet_protocol_fee::GenesisConfig::<TestStorage> {
            base_fees: self.protocol_base_fees.0,
            coefficient: self.protocol_coefficient,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        pips::GenesisConfig::<TestStorage> {
            prune_historical_pips: false,
            min_proposal_deposit: 50,
            quorum_threshold: 70,
            proposal_duration: 10,
            proposal_cool_off_period: 100,
            default_enactment_period: 100,
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        sp_io::TestExternalities::new(storage)
    }
}
