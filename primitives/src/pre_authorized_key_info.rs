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
