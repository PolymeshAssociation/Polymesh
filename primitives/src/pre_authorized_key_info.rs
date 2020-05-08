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

use crate::{IdentityId, SigningItem};

use codec::{Decode, Encode};

/// It stores information of pre-authorized keys.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Eq, Debug)]
pub struct PreAuthorizedKeyInfo {
    pub target_id: IdentityId,
    pub signing_item: SigningItem,
}

impl PreAuthorizedKeyInfo {
    /// Create from `sk` signing key to target `id` identity.
    pub fn new(si: SigningItem, id: IdentityId) -> Self {
        Self {
            target_id: id,
            signing_item: si,
        }
    }
}

impl PartialEq for PreAuthorizedKeyInfo {
    fn eq(&self, other: &Self) -> bool {
        self.target_id == other.target_id
    }
}

impl PartialEq<IdentityId> for PreAuthorizedKeyInfo {
    fn eq(&self, other: &IdentityId) -> bool {
        self.target_id == *other
    }
}
