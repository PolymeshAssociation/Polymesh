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

use crate::{SecondaryKey, Signatory};
use codec::{Decode, Encode};
use sp_core::{crypto::Public as PublicType, sr25519::Public};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::From, prelude::Vec};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Identity<AccountId: Encode + Decode> {
    pub primary_key: AccountId,
    pub secondary_keys: Vec<SecondaryKey<AccountId>>,
}

impl<AccountId> Identity<AccountId>
where
    AccountId: Encode + Decode + Clone + Default + Eq + Ord,
{
    /// Creates an [`Identity`] from an `AccountId`.
    pub fn new(primary_key: AccountId) -> Self {
        Identity {
            primary_key,
            ..Default::default()
        }
    }

    /// Check if `signer` is already a secondary key.
    pub fn contains_secondary_key(&self, signer: &Signatory<AccountId>) -> bool {
        self.secondary_keys.iter().any(|sk| sk.signer == *signer)
    }

    /// It adds `new_secondary_keys` to `self`.
    pub fn add_secondary_keys(
        &mut self,
        new_secondary_keys: impl IntoIterator<Item = SecondaryKey<AccountId>>,
    ) -> &mut Self {
        self.secondary_keys.extend(new_secondary_keys);

        self
    }

    /// It removes `keys_to_remove` from secondary keys.
    pub fn remove_secondary_keys(
        &mut self,
        signers_to_remove: &[Signatory<AccountId>],
    ) -> &mut Self {
        self.secondary_keys.retain(|curr_si| {
            signers_to_remove
                .iter()
                .find(|&signer| curr_si.signer == *signer)
                .is_none()
        });
        self
    }

    /// Returns the complexity of the identity's permissions.
    pub fn complexity(&self) -> usize {
        self.secondary_keys
            .iter()
            .fold(0, |cost, key| cost.saturating_add(key.complexity()))
    }
}

impl<AccountId> From<Public> for Identity<AccountId>
where
    AccountId: Encode + Decode + PublicType,
{
    fn from(p: Public) -> Self {
        Identity {
            primary_key: AccountId::from_slice(&p.0[..]),
            ..Default::default()
        }
    }
}
