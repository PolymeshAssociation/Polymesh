use std::collections::HashMap;

use polymesh_api::client::{Result, Signer};
use polymesh_api_client_extras::*;
pub use polymesh_api::{
    client::{AccountId, IdentityId},
    polymesh::types::{
        polymesh_primitives::secondary_key::{KeyRecord, Permissions, SecondaryKey},
        runtime::{events::*, RuntimeEvent},
    },
    Api,
};

mod account;
pub use account::*;

pub async fn client_api() -> Result<Api> {
    // TODO: use env to get URL.
    Ok(Api::new("ws://localhost:9944").await?)
}

#[derive(Clone)]
pub struct User {
    pub signer: AccountSigner,
    pub did: Option<IdentityId>,
}

#[async_trait::async_trait]
impl Signer for User {
    fn account(&self) -> AccountId {
        self.signer.account()
    }

    fn nonce(&self) -> Option<u32> {
        self.signer.nonce()
    }

    fn set_nonce(&mut self, nonce: u32) {
        self.signer.set_nonce(nonce)
    }

    async fn sign(&self, msg: &[u8]) -> polymesh_api::client::Result<sp_runtime::MultiSignature> {
        Ok(self.signer.sign(msg).await?)
    }
}

impl User {
    pub fn new(signer: AccountSigner) -> Self {
        Self { signer, did: None }
    }

    pub fn account(&self) -> AccountId {
        self.signer.account()
    }
}

pub const ONE_POLYX: u128 = 1_000_000;

pub struct PolymeshTester {
    pub api: Api,
    seed: String,
    init_polyx: u128,
    cdd: AccountSigner,
    users: HashMap<String, User>,
}

impl PolymeshTester {
    pub async fn new() -> Result<Self> {
        let api = client_api().await?;
        // Generate a seed based on current timestamp.
        // We use a 'seed' to allow running tests in parallel.
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("now later then epoch")
            .as_nanos();
        Ok(Self {
            api,
            init_polyx: 10_000 * ONE_POLYX,
            seed: format!("{}", ts),
            cdd: AccountSigner::alice(),
            users: Default::default(),
        })
    }

    /// Set how much POLYX to fund each user.
    pub fn set_init_polyx(&mut self, val: u128) {
        self.init_polyx = val * ONE_POLYX;
    }

    fn set_user_did(&mut self, name: &str, did: IdentityId) {
        if let Some(user) = self.users.get_mut(name) {
            user.did = Some(did);
        }
    }

    fn get_user(&mut self, name: &str) -> Result<User> {
        use std::collections::hash_map::Entry;
        match self.users.entry(name.to_string()) {
            Entry::Occupied(entry) => Ok(entry.get().clone()),
            Entry::Vacant(entry) => {
                let signer =
                    AccountSigner::from_string(&format!("//{}_{}", self.seed, entry.key()))?;
                let user = User::new(signer);
                Ok(entry.insert(user).clone())
            }
        }
    }

    /// Get the user if they exist, or create a new one.  Make sure the user
    /// has an identity.
    pub async fn user(&mut self, name: &str) -> Result<User> {
        let mut user = self.get_user(name)?;
        if user.did.is_none() {
            let did = self.register_and_fund(user.account()).await?;
            user.did = Some(did);
            self.set_user_did(name, did);
        }
        Ok(user)
    }

    async fn load_dids(&mut self, users: &mut [User]) -> Result<()> {
        for user in users {
            // Skip users that have an identity.
            if user.did.is_some() {
                continue;
            }
            // Try getting the user's identity from the chain.
            user.did = self.get_did(user.account()).await?;
        }
        Ok(())
    }

    /// Get the users if they exist, or create them.  Make sure the users
    /// have identities.
    pub async fn users(&mut self, names: &[&str]) -> Result<Vec<User>> {
        let mut users = Vec::with_capacity(names.len());
        for name in names {
            // Get or create user.
            users.push(self.get_user(name)?);
        }
        self.load_dids(users.as_mut_slice()).await?;
        // Calls for registering users and funding them.
        let mut calls = Vec::new();
        // Add calls to register users missing identities.
        let mut need_dids = Vec::new();
        for (idx, user) in users.iter().enumerate() {
            if user.did.is_some() {
                continue;
            }
            need_dids.push(idx);
            // User needs an identity.
            calls.push(
                self.api
                    .call()
                    .identity()
                    .cdd_register_did_with_cdd(user.account(), vec![], None)?
                    .into(),
            );
        }
        // Add calls to fund the users.
        for user in &users {
            // Transfer some funds to the user.
            calls.push(
                self.api
                    .call()
                    .balances()
                    .transfer(user.account().into(), self.init_polyx)?
                    .into(),
            );
        }
        // Execute batch.
        let mut res = self
            .api
            .call()
            .utility()
            .batch(calls)?
            .execute(&mut self.cdd)
            .await?;
        // Get new identities from batch events.
        let ids = get_created_ids(&mut res).await?;
        for idx in need_dids {
            let name = names[idx];
            match &ids[idx] {
                CreatedIds::IdentityCreated(did) => {
                    users[idx].did = Some(*did);
                    self.set_user_did(name, *did);
                }
                id => {
                    panic!("Unexpected id: {id:?}");
                }
            }
        }
        Ok(users)
    }

    pub async fn key_records(&self, account: AccountId) -> Result<Option<KeyRecord<AccountId>>> {
        self.api.query().identity().key_records(account).await
    }

    pub async fn get_did(&self, account: AccountId) -> Result<Option<IdentityId>> {
        let did = match self.key_records(account).await? {
            Some(KeyRecord::PrimaryKey(did)) => Some(did),
            Some(KeyRecord::SecondaryKey(did, _)) => Some(did),
            _ => None,
        };
        Ok(did)
    }

    pub async fn register_and_fund(&mut self, account: AccountId) -> Result<IdentityId> {
        let did = match self.get_did(account).await? {
            Some(did) => did,
            None => {
                // `account` is not linked to an identity.
                // Create a new identity with `account` as the primary key.
                let mut res = self
                    .api
                    .call()
                    .utility()
                    .batch(vec![
                        self.api
                            .call()
                            .identity()
                            .cdd_register_did_with_cdd(account, vec![], None)?
                            .into(),
                        self.api
                            .call()
                            .balances()
                            .transfer(account.into(), self.init_polyx)?
                            .into(),
                    ])?
                    .execute(&mut self.cdd)
                    .await?;
                get_identity_id(&mut res).await?.unwrap()
            }
        };
        Ok(did)
    }
}
