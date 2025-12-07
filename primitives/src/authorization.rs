// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::*;

use crate::agent::AgentGroup;
use crate::asset::AssetId;
use crate::identity_id::IdentityId;
use crate::secondary_key::Permissions;
use crate::{Balance, PortfolioId, Ticker};

/// Authorization data for two step processes.
#[derive(Decode, DecodeWithMemTracking, Encode, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Clone, Debug, Deserialize, TypeInfo, Serialize)]
pub enum AuthorizationData<AccountId> {
    /// CDD provider's attestation to change primary key
    AttestPrimaryKeyRotation(IdentityId),
    /// Authorization to change primary key
    RotatePrimaryKey,
    /// Authorization to transfer a ticker
    /// Must be issued by the current owner of the ticker
    TransferTicker(Ticker),
    /// Add a signer to multisig
    /// Must be issued to the identity that created the ms (and therefore owns it permanently)
    AddMultiSigSigner(AccountId),
    /// Authorization to transfer a token's ownership
    /// Must be issued by the current owner of the asset
    TransferAssetOwnership(AssetId),
    /// Authorization to join an Identity
    /// Must be issued by the identity which is being joined
    JoinIdentity(Permissions),
    /// Authorization to take custody of a portfolio
    PortfolioCustody(PortfolioId),
    /// Authorization to become an agent of the `AssetId` with the `AgentGroup`.
    BecomeAgent(AssetId, AgentGroup),
    /// Add Relayer paying key to user key
    /// Must be issued by the paying key.
    /// `AddRelayerPayingKey(user_key, paying_key, polyx_limit)`
    AddRelayerPayingKey(AccountId, AccountId, Balance),
    /// Authorization to change primary key and leave it as a secondary key
    /// with the given permissions.
    RotatePrimaryKeyToSecondary(Permissions),
}

impl<AccountId> AuthorizationData<AccountId> {
    /// Returns the `AuthorizationType` of this auth data.
    pub fn auth_type(&self) -> AuthorizationType {
        match self {
            Self::AttestPrimaryKeyRotation(..) => AuthorizationType::AttestPrimaryKeyRotation,
            Self::RotatePrimaryKey => AuthorizationType::RotatePrimaryKey,
            Self::TransferTicker(..) => AuthorizationType::TransferTicker,
            Self::BecomeAgent(..) => AuthorizationType::BecomeAgent,
            Self::AddMultiSigSigner(..) => AuthorizationType::AddMultiSigSigner,
            Self::TransferAssetOwnership(..) => AuthorizationType::TransferAssetOwnership,
            Self::JoinIdentity(..) => AuthorizationType::JoinIdentity,
            Self::PortfolioCustody(..) => AuthorizationType::PortfolioCustody,
            Self::AddRelayerPayingKey(..) => AuthorizationType::AddRelayerPayingKey,
            Self::RotatePrimaryKeyToSecondary(..) => AuthorizationType::RotatePrimaryKeyToSecondary,
        }
    }
}

/// Type of authorization.
#[derive(Eq, PartialEq, Encode, Decode, TypeInfo, Clone)]
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Serialize, Deserialize)]
pub enum AuthorizationType {
    /// CDD Authorization to rotate primary key.
    AttestPrimaryKeyRotation,
    /// Authorization to rotate primary key.
    RotatePrimaryKey,
    /// Authorization to transfer a ticker.
    TransferTicker,
    /// Authorization to add some key int a multi signer.
    AddMultiSigSigner,
    /// Authorization to transfer the asset ownership to other identity.
    TransferAssetOwnership,
    /// Join Identity authorization.
    JoinIdentity,
    /// Accept custody of a portfolio
    PortfolioCustody,
    /// Authorization to become an agent of a ticker.
    BecomeAgent,
    /// Authorization to add a Relayer paying key.
    AddRelayerPayingKey,
    /// Authorization to change primary key with an existing secondary key
    RotatePrimaryKeyToSecondary,
}

/// Authorization struct
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Debug)]
#[derive(Serialize, Deserialize)]
pub struct Authorization<AccountId, Moment> {
    /// Enum that contains authorization type and data
    pub authorization_data: AuthorizationData<AccountId>,

    /// Identity of the organization/individual that added this authorization
    pub authorized_by: IdentityId,

    /// time when this authorization expires. optional.
    pub expiry: Option<Moment>,

    /// Authorization id of this authorization
    pub auth_id: u64,

    /// Current count for how many times an authorization can fail. optional.
    pub count: u32,
}

/// Extract the authoriation variant's data, or bail.
#[macro_export]
macro_rules! extract_auth {
    ($data:expr, $variant:ident ( $($f:ident),*) ) => {
        match $data {
            $crate::authorization::AuthorizationData::$variant($($f),*) => ($($f),*),
            _ => frame_support::fail!(Error::<T>::BadAuthorizationType),
        }
    };
    ($data:expr, $variant:ident ) => {
        match $data {
            $crate::authorization::AuthorizationData::$variant => (),
            _ => frame_support::fail!(Error::<T>::BadAuthorizationType),
        }
    };
}
