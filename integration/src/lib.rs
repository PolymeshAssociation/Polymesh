use sp_core::Pair;
use sp_runtime::MultiSignature;
use sp_keyring::{sr25519, ed25519};

use polymesh_api::client::{
  AccountId,
  Signer,
  Result,
};

use polymesh_api::Api;

pub struct PolymeshAccount<P: Pair> {
  pub pair: P,
  pub account: AccountId,
}

impl PolymeshAccount<sr25519::sr25519::Pair> {
  pub fn alice() -> Self {
    Self::new(sr25519::Keyring::Alice.pair())
  }
}

impl<P> PolymeshAccount<P>
where
  P: Pair,
  MultiSignature: From<<P as Pair>::Signature>,
  AccountId: From<<P as Pair>::Public>,
{
  pub fn new(pair: P) -> Self {
    let account = pair.public().into();
    Self {
      pair,
      account,
    }
  }

  /// Generate signing key pair from string `s`.
  ///
  /// See [`from_string_with_seed`](Pair::from_string_with_seed) for more extensive documentation.
  pub fn from_string(s: &str, password_override: Option<&str>) -> Result<Self> {
    Ok(Self::new(P::from_string(s, password_override)?))
  }
}

impl From<sr25519::Keyring> for PolymeshAccount<sr25519::sr25519::Pair> {
  fn from(key: sr25519::Keyring) -> Self {
    Self::new(key.pair())
  }
}

impl From<ed25519::Keyring> for PolymeshAccount<ed25519::ed25519::Pair> {
  fn from(key: ed25519::Keyring) -> Self {
    Self::new(key.pair())
  }
}

#[async_trait::async_trait]
impl<P: Pair> Signer for PolymeshAccount<P>
where
  MultiSignature: From<<P as Pair>::Signature>,
{
  fn account(&self) -> AccountId {
    self.account.clone()
  }

  fn nonce(&self) -> Option<u32> {
    let account = self.account.to_hex();
    ureq::get(&format!("http://127.0.0.1:8080/account/{}/get_nonce", account))
      .call().expect("get_nonce request")
      .into_string().expect("response as string")
      .parse::<u32>().ok()
  }

  fn set_nonce(&mut self, _nonce: u32) {
  }

  async fn sign(&self, msg: &[u8]) -> polymesh_api::client::Result<MultiSignature> {
    Ok(self.pair.sign(msg).into())
  }
}

pub async fn client_api() -> Result<Api> {
  Ok(Api::new("ws://localhost:9944").await?)
}
