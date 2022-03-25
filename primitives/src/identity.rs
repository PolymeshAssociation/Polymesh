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

use crate::SecondaryKey;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::Vec;

/// Identity record.
///
/// Used to check if an `Identity` exists and lookup it's primary key.
///
/// Asset Identities don't have a primary key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DidRecord<AccountId> {
    /// Identity's primary key, if it has one.
    pub primary_key: Option<AccountId>,
}

impl<AccountId> DidRecord<AccountId> {
    /// Creates an `DidRecord` from an `AccountId`.
    pub fn new(primary_key: AccountId) -> Self {
        Self {
            primary_key: Some(primary_key),
        }
    }
}

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IdentityRecord<AccountId> {
    pub primary_key: AccountId,
    pub secondary_keys: Vec<SecondaryKey<AccountId>>,
}
