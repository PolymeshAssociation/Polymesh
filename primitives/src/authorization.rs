use crate::identity_id::IdentityId;
use crate::signing_item::Signatory;
use crate::Ticker;
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchError;

/// Authorization data for two step prcoesses.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum AuthorizationData {
    /// CDD provider's attestation to change master key
    AttestMasterKeyRotation(IdentityId),
    /// Authorization to change master key
    RotateMasterKey(IdentityId),
    /// Authorization to transfer a ticker
    TransferTicker(Ticker),
    /// Add a signer to multisig
    AddMultiSigSigner,
    /// Authorization to transfer a token's ownership
    TransferAssetOwnership(Ticker),
    /// Authorization to join an Identity
    JoinIdentity(IdentityId),
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

impl From<AuthorizationError> for DispatchError {
    fn from(error: AuthorizationError) -> DispatchError {
        match error {
            AuthorizationError::Invalid => DispatchError::Other("Authorization does not exist"),
            AuthorizationError::Unauthorized => {
                DispatchError::Other("Illegal use of Authorization")
            }
            AuthorizationError::Expired => DispatchError::Other("Authorization expired"),
        }
    }
}

/// Authorization struct
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Authorization<U> {
    /// Enum that contains authorization type and data
    pub authorization_data: AuthorizationData,

    /// Identity of the organization/individual that added this authorization
    pub authorized_by: Signatory,

    /// time when this authorization expires. optional.
    pub expiry: Option<U>,

    /// Authorization id of this authorization
    pub auth_id: u64,
}

/// Data required to fetch and authorization
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct AuthIdentifier(pub Signatory, pub u64);
