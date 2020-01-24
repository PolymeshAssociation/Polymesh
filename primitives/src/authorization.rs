use crate::signing_item::Signer;
use crate::Ticker;
use codec::{Decode, Encode};
use rstd::prelude::Vec;

/// Authorization data for two step prcoesses.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AuthorizationData {
    /// Authorization to transfer a ticker
    TransferTicker(Ticker),
    /// Authorization to transfer a token's ownership
    TransferTokenOwnership(Ticker),
    /// Any other authorization
    Custom(Ticker),
    /// No authorization data
    NoData,
}

impl Default for AuthorizationData {
    fn default() -> Self {
        AuthorizationData::NoData
    }
}

/// Status of an Authorization after consume is called on it.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AuthorizationError {
    /// Auth does not exist
    Invalid,
    /// Caller not authorized or the identity who created
    /// this authorization is not authorized to create this authorization
    Unauthorized,
    /// Auth expired already
    Expired,
}

impl From<AuthorizationError> for &'static str {
    fn from(error: AuthorizationError) -> &'static str {
        match error {
            AuthorizationError::Invalid => "Authorization does not exist",
            AuthorizationError::Unauthorized => "Illegal use of Authorization",
            AuthorizationError::Expired => "Authorization expired",
        }
    }
}

/// Authorization struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Authorization<U> {
    /// Enum that contains authorization type and data
    pub authorization_data: AuthorizationData,

    /// Identity of the organization/individual that added this authorization
    pub authorized_by: Signer,

    /// time when this authorization expires. optional.
    pub expiry: Option<U>,

    // Extra data to allow iterating over the authorizations.
    /// Authorization number of the next Authorization.
    /// Authorization number starts with 1.
    pub next_authorization: u64,
    /// Authorization number of the previous Authorization.
    /// Authorization number starts with 1.
    pub previous_authorization: u64,
}
