// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::{
    identity_id::IdentityId,
    signing_item::{Permission, Signatory},
    Ticker,
};
use codec::{Decode, Encode};
use frame_support::dispatch::DispatchError;
use sp_std::prelude::*;

/// Authorization data for two step processes.
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
    JoinIdentity(JoinIdentityData),
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

/// Authorization Identity data
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct JoinIdentityData {
    /// Target DID under which signing_item need to be added
    pub target_did: IdentityId,

    /// Signing Item
    pub permissions: Vec<Permission>,
}

impl JoinIdentityData {
    /// Use to create the new Self object by providing target_did and permission
    pub fn new(target_did: IdentityId, permissions: Vec<Permission>) -> Self {
        Self {
            target_did: target_did,
            permissions: permissions,
        }
    }
}

/// Data required to fetch and authorization
#[derive(Encode, Decode, Clone, Default, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub struct AuthIdentifier(pub Signatory, pub u64);
