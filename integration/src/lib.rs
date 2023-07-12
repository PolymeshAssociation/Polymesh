use polymesh_api::client::{
  Result,
};

use polymesh_api::Api;

mod account;
pub use account::*;

mod identity;
pub use identity::*;

pub async fn client_api() -> Result<Api> {
  Ok(Api::new("ws://localhost:9944").await?)
}
