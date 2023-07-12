use std::collections::HashMap;
use std::sync::Mutex;

use thiserror::Error;

use actix_web::{
    error,
    http::{header::ContentType, StatusCode},
    HttpResponse,
};
use actix_web::{get, web, App, HttpServer};

use polymesh_api::client::{AccountId, ChainApi};
use polymesh_api::Api;

use integration::*;

#[derive(Debug, Error)]
pub enum Error {
    #[error("Polymesh API error: {0}")]
    PolymeshApiError(polymesh_api::client::Error),
}

impl From<polymesh_api::client::Error> for Error {
    fn from(e: polymesh_api::client::Error) -> Self {
        Self::PolymeshApiError(e)
    }
}

impl error::ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code())
            .insert_header(ContentType::html())
            .body(self.to_string())
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            Error::PolymeshApiError(_) => StatusCode::BAD_GATEWAY,
        }
    }
}

type Result<T, E = Error> = std::result::Result<T, E>;

struct Backend {
    client: Api,
    accounts: HashMap<AccountId, u32>,
}

impl Backend {
    async fn new() -> Result<Self> {
        Ok(Self {
            client: client_api().await?,
            accounts: Default::default(),
        })
    }

    async fn get_nonce(&mut self, account: AccountId) -> Result<u32> {
        use std::collections::hash_map::Entry;
        match self.accounts.entry(account) {
            Entry::Occupied(mut entry) => {
                let nonce = entry.get_mut();
                let last = *nonce;
                *nonce += 1;
                return Ok(last);
            }
            Entry::Vacant(entry) => {
                let nonce = self.client.get_nonce(account).await?;
                entry.insert(nonce + 1);
                Ok(nonce)
            }
        }
    }
}

struct AppState {
    backend: Mutex<Backend>,
}

impl AppState {
    async fn new() -> Result<Self> {
        Ok(Self {
            backend: Mutex::new(Backend::new().await?),
        })
    }
}

#[get("/account/0x{id}/get_nonce")]
async fn account_get_nonce(
    data: web::Data<AppState>,
    path: web::Path<AccountId>,
) -> Result<String> {
    let account_id = path.into_inner();
    let mut backend = data.backend.lock().unwrap();

    let nonce = backend.get_nonce(account_id).await?;

    Ok(format!("{nonce}"))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let state = web::Data::new(AppState::new().await.expect("app backend"));

    HttpServer::new(move || {
        App::new()
            .app_data(state.clone())
            .service(account_get_nonce)
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await
}
