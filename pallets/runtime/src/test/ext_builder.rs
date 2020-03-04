use crate::{
    asset::{self, TickerRegistrationConfig},
    test::TestStorage,
};

use polymesh_primitives::{AccountKey, Identity, IdentityId};
use polymesh_runtime_balances as balances;
use polymesh_runtime_common::traits::identity::LinkedKeyInfo;
use polymesh_runtime_group as group;
use polymesh_runtime_identity as identity;

use sp_core::sr25519::Public;
use sp_io::TestExternalities;
use test_client::AccountKeyring;

use std::{cell::RefCell, convert::From};

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
    fn make_cdd_identities(
        &self,
    ) -> (
        Vec<(IdentityId, Identity)>,
        Vec<(AccountKey, LinkedKeyInfo)>,
    ) {
        let keys = self
            .cdd_providers
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

        // Define CDD providers.
        let (cdd_identities, cdd_links) = self.make_cdd_identities();
        let cdd_ids: Vec<IdentityId> = cdd_identities.iter().map(|(id, _)| id.clone()).collect();

        // Identity genesis.
        identity::GenesisConfig::<TestStorage> {
            owner: root.clone().into(),
            did_creation_fee: 250,
            did_records: cdd_identities,
            key_to_identity_ids: cdd_links,
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
        group::GenesisConfig::<TestStorage, group::Instance2> {
            members: cdd_ids,
            ..Default::default()
        }
        .assimilate_storage(&mut storage)
        .unwrap();

        sp_io::TestExternalities::new(storage)
    }
}
