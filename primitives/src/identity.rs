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

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

#[cfg(feature = "running-ci")]
/// Defines the constants for the identity pallet.
pub mod limits {
    /// Maximum number of secondary keys allowed.
    pub const MAX_SECONDARY_KEYS: u32 = 2;
    /// Maximum number of assets allowed.
    pub const MAX_ASSETS: u32 = 4;
    /// Maximum number of portfolios allowed.
    pub const MAX_PORTFOLIOS: u32 = 4;
    /// Maximum number of pallets allowed.
    pub const MAX_PALLETS: u32 = 4;
    /// Maximum number of extrinsics allowed.
    pub const MAX_EXTRINSICS: u32 = 4;
}

#[cfg(not(feature = "running-ci"))]
/// Defines the constants for the identity pallet.
pub mod limits {
    /// Maximum number of secondary keys allowed.
    pub const MAX_SECONDARY_KEYS: u32 = 200;
    /// Maximum number of assets allowed.
    pub const MAX_ASSETS: usize = 2000;
    /// Maximum number of portfolios allowed.
    pub const MAX_PORTFOLIOS: usize = 2000;
    /// Maximum number of pallets allowed.
    pub const MAX_PALLETS: usize = 80;
    /// Maximum number of extrinsics allowed.
    pub const MAX_EXTRINSICS: usize = 80;
}

/// Identity record.
///
/// Used to check if an identity exists and lookup its primary key.
///
/// Asset Identities don't have a primary key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DidRecord<AccountId> {
    /// The identity's primary key, if it has one.
    pub primary_key: Option<AccountId>,
}

impl<AccountId> Default for DidRecord<AccountId> {
    fn default() -> Self {
        Self { primary_key: None }
    }
}

impl<AccountId> DidRecord<AccountId> {
    /// Creates an `DidRecord` from an `AccountId`.
    pub fn new(primary_key: AccountId) -> Self {
        Self {
            primary_key: Some(primary_key),
        }
    }
}
