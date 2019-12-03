use crate::{IdentityId, KeyType, Permission, SigningKey};

use codec::{Decode, Encode};
use rstd::prelude::Vec;

/// It stores information of pre-authorized keys.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Eq, Debug)]
pub struct PreAuthorizedKeyInfo {
    pub identity: IdentityId,
    pub key_type: KeyType,
    pub permissions: Vec<Permission>,
}

impl PreAuthorizedKeyInfo {
    /// Create from `sk` signing key to target `id` identity.
    pub fn new(sk: SigningKey, id: IdentityId) -> Self {
        Self {
            identity: id,
            key_type: sk.key_type,
            permissions: sk.permissions,
        }
    }
}

impl PartialEq for PreAuthorizedKeyInfo {
    fn eq(&self, other: &Self) -> bool {
        self.identity == other.identity
    }
}

impl PartialEq<IdentityId> for PreAuthorizedKeyInfo {
    fn eq(&self, other: &IdentityId) -> bool {
        self.identity == *other
    }
}
