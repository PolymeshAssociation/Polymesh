use crate::{
    asset::{self, TickerRegistrationConfig},
    test::TestStorage,
};

use pallet_committee as committee;
use polymesh_primitives::{AccountKey, Identity, IdentityId};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::identity::LinkedKeyInfo;
use polymesh_runtime_group as group;
use polymesh_runtime_identity as identity;

use sp_core::sr25519::Public;
use sp_io::TestExternalities;
use test_client::AccountKeyring;

use std::{cell::RefCell, convert::From};

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

#[derive(Default)]
pub struct ExtBuilder {
    transaction_base_fee: u128,
    transaction_byte_fee: u128,
    weight_to_fee: u128,
    existential_deposit: u128,
    transfer_fee: u128,
    creation_fee: u128,
    monied: bool,
    vesting: bool,
    cdd_providers: Vec<Public>,
    governance_committee_members: Vec<Public>,
    governance_committee_vote_threshold: BuilderVoteThreshold,
}

thread_local! {
    static EXISTENTIAL_DEPOSIT: RefCell<u128> = RefCell::new(0);
    static TRANSFER_FEE: RefCell<u128> = RefCell::new(0);
    static CREATION_FEE: RefCell<u128> = RefCell::new(0);
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

    #[allow(dead_code)]
    pub fn transfer_fee(mut self, transfer_fee: u128) -> Self {
        self.transfer_fee = transfer_fee;
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

    pub fn set_associated_consts(&self) {
        EXISTENTIAL_DEPOSIT.with(|v| *v.borrow_mut() = self.existential_deposit);
        TRANSFER_FEE.with(|v| *v.borrow_mut() = self.transfer_fee);
        CREATION_FEE.with(|v| *v.borrow_mut() = self.creation_fee);
        TRANSACTION_BASE_FEE.with(|v| *v.borrow_mut() = self.transaction_base_fee);
        TRANSACTION_BYTE_FEE.with(|v| *v.borrow_mut() = self.transaction_byte_fee);
        WEIGHT_TO_FEE.with(|v| *v.borrow_mut() = self.weight_to_fee);
    }

    fn make_balances(&self) -> Vec<(Public, u128)> {
        if self.monied {
            vec![
                (
                    AccountKeyring::Alice.public(),
                    10 * self.existential_deposit,
                ),
                (AccountKeyring::Bob.public(), 20 * self.existential_deposit),
                (
                    AccountKeyring::Charlie.public(),
                    30 * self.existential_deposit,
                ),
                (AccountKeyring::Dave.public(), 40 * self.existential_deposit),
                // CDD Accounts
                (AccountKeyring::Eve.public(), 1_000_000),
                (AccountKeyring::Ferdie.public(), 1_000_000),
            ]
        } else {
            vec![]
        }
    }

    fn make_vesting(&self) -> Vec<(Public, u64, u64, u128)> {
        if self.vesting && self.monied {
            vec![
                (
                    AccountKeyring::Alice.public(),
                    0,
                    10,
                    5 * self.existential_deposit,
                ),
                (AccountKeyring::Bob.public(), 10, 20, 0),
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

        let root = AccountKeyring::Alice.public();

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
            owner: root.clone().into(),
            did_creation_fee: 250,
            did_records: system_identities.clone(),
            key_to_identity_ids: system_links,
            identities: vec![],
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // Balances genesis.
        balances::GenesisConfig::<TestStorage> {
            balances: self.make_balances(),
            vesting: self.make_vesting(),
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        // Asset genesis.
        asset::GenesisConfig::<TestStorage> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 8,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
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
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        sp_io::TestExternalities::new(storage)
    }
}
