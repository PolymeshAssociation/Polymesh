use std::sync::Arc;

use sp_core::Pair;
use sp_keyring::{ed25519, sr25519};
use sp_runtime::MultiSignature;

use polymesh_api::client::{AccountId, PairSigner, Result, Signer};

use crate::User;

/// AccountSigner is wrapper for signing keys (sr25519, ed25519, etc...).
#[derive(Clone)]
pub struct AccountSigner {
    signer: Arc<dyn Signer + Send + Sync>,
    account: AccountId,
}

impl AccountSigner {
    pub fn new<P: Pair>(pair: P) -> Self
    where
        MultiSignature: From<<P as Pair>::Signature>,
        AccountId: From<<P as Pair>::Public>,
    {
        let signer = PairSigner::new(pair);
        let account = signer.account();
        Self {
            signer: Arc::new(signer),
            account,
        }
    }

    pub fn alice() -> Self {
        Self::new(sr25519::Keyring::Alice.pair())
    }

    pub fn bob() -> Self {
        Self::new(sr25519::Keyring::Bob.pair())
    }

    /// Generate signing key pair from string `s`.
    pub fn from_string(s: &str) -> Result<Self> {
        Ok(Self::new(sr25519::sr25519::Pair::from_string(s, None)?))
    }
}

impl From<AccountSigner> for User {
    fn from(signer: AccountSigner) -> User {
        User { signer, did: None }
    }
}

impl From<sr25519::Keyring> for AccountSigner {
    fn from(key: sr25519::Keyring) -> Self {
        Self::new(key.pair())
    }
}

impl From<ed25519::Keyring> for AccountSigner {
    fn from(key: ed25519::Keyring) -> Self {
        Self::new(key.pair())
    }
}

#[async_trait::async_trait]
impl Signer for AccountSigner {
    fn account(&self) -> AccountId {
        self.account.clone()
    }

    fn nonce(&self) -> Option<u32> {
        let account = self.account.to_hex();
        ureq::get(&format!(
            "http://127.0.0.1:8080/account/{}/get_nonce",
            account
        ))
        .call()
        .expect("get_nonce request")
        .into_string()
        .expect("response as string")
        .parse::<u32>()
        .ok()
    }

    fn set_nonce(&mut self, _nonce: u32) {}

    async fn sign(&self, msg: &[u8]) -> polymesh_api::client::Result<MultiSignature> {
        Ok(self.signer.sign(msg).await?)
    }
}
