use polymesh_api::client::{
  IdentityId,
  AccountId,
  Signer,
  Result,
};
use polymesh_api::{
  Api,
  polymesh::types::{
    polymesh_primitives::{
      secondary_key::{
        KeyRecord,
        //SecondaryKey,
        //Permissions,
      },
    }
  }
};

pub async fn key_records(api: &Api, account: AccountId) -> Result<Option<KeyRecord<AccountId>>> {
  api.query().identity().key_records(account).await
}

pub async fn get_did(api: &Api, account: AccountId) -> Result<Option<IdentityId>> {
  let did = match key_records(api, account).await? {
    Some(KeyRecord::PrimaryKey(did)) => Some(did),
    Some(KeyRecord::SecondaryKey(did, _)) => Some(did),
    _ => None,
  };
  Ok(did)
}

#[derive(Debug, Default, Clone)]
pub struct PolymeshIdentity {
  pub did: IdentityId,
  pub account: AccountId,
}

impl PolymeshIdentity {
  pub async fn create_identity(api: &Api, cdd: &mut impl Signer, account: AccountId) -> Result<Self> {
    let did = match get_did(api, account).await? {
      Some(did) => did,
      None => {
        let mut _res = api
          .call()
          .identity()
          .cdd_register_did_with_cdd(account, vec![], None)?
          .sign_submit_and_watch(cdd)
          .await?;
        // TODO: Process Identity::DidCreated(did, _, _) event.
        get_did(api, account).await?.unwrap()
      }
    };
    Ok(Self {
      did,
      account
    })
  }
}
