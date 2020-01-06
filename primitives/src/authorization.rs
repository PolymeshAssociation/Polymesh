use crate::identity_id::IdentityId;
use codec::{Decode, Encode};
use rstd::prelude::Vec;

/// Authorization data for two step prcoesses.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AuthorizationData {
    /// Authorization to transfer a ticker
    TransferTicker(Vec<u8>),
    /// No authorization data
    None,
    /// Any other authorization
    Custom(Vec<u8>),
}

impl Default for AuthorizationData {
    fn default() -> Self {
        AuthorizationData::None
    }
}

/// Authorization struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Authorization<U> {
    /// Enum that contains authorization type and data
    pub authorization_data: AuthorizationData,

    /// Identity of the organization/individual that added this authorization
    pub authorized_by: IdentityId,

    /// time when this authorization expires. optional.
    pub expiry: Option<U>,

    // Extra data to allow iterating over the authorizations.
    /// Authorization ID of the next Authorization
    pub next_authorization: Option<u64>,
    /// Authorization ID of the previous Authorization
    pub previous_authorization: Option<u64>,
}
