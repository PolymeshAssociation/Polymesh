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

use codec::{Decode, Encode};
use sp_core::{crypto::Public as PublicType, sr25519::Public};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::From, prelude::Vec};

use crate::{IdentityRole, Signatory, SigningKey};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Identity<AccountId> {
    pub roles: Vec<IdentityRole>,
    pub master_key: AccountId,
    pub signing_keys: Vec<SigningKey<AccountId>>,
}

impl<AccountId> Identity<AccountId>
where
    AccountId: Clone + Default + Eq + Ord,
{
    /// Creates an [`Identity`] from an `AccountId`.
    pub fn new(master_key: AccountId) -> Self {
        Identity {
            master_key,
            ..Default::default()
        }
    }

    /// It checks if this entity contains IdentityRole `role`.
    pub fn has_role(&self, role: IdentityRole) -> bool {
        self.roles.contains(&role)
    }

    /// It adds `new_signing_keys` to `self`.
    /// It also keeps its internal list sorted and removes duplicate elements.
    pub fn add_signing_keys(&mut self, new_signing_keys: &[SigningKey<AccountId>]) -> &mut Self {
        self.signing_keys.extend_from_slice(new_signing_keys);
        self.signing_keys.sort();
        self.signing_keys.dedup();

        self
    }

    /// It removes `keys_to_remove` from signing keys.
    pub fn remove_signing_keys(&mut self, signers_to_remove: &[Signatory<AccountId>]) -> &mut Self {
        self.signing_keys.retain(|curr_si| {
            signers_to_remove
                .iter()
                .find(|&signer| curr_si.signer == *signer)
                .is_none()
        });

        self
    }
}

impl<AccountId> From<Public> for Identity<AccountId>
where
    AccountId: PublicType,
{
    fn from(p: Public) -> Self {
        Identity {
            master_key: AccountId::from_slice(&p.0[..]),
            ..Default::default()
        }
    }
}
