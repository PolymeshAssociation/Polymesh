use crate::identity_id::IdentityId;
use crate::key::Key;
use codec::{Decode, Encode};

/// Enum that represents either a Key or an Identity
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug)]
pub enum IdentityOrKey {
    /// An Identity identitified by IdentityId
    Identity(IdentityId),
    /// A key identified by Key
    Key(Key),
    /// Default type that means "Anyone"
    Any,
}

impl Default for IdentityOrKey {
    fn default() -> Self {
        IdentityOrKey::Any
    }
}

impl From<Key> for IdentityOrKey {
    fn from(v: Key) -> Self {
        IdentityOrKey::Key(v)
    }
}

impl From<IdentityId> for IdentityOrKey {
    fn from(v: IdentityId) -> Self {
        IdentityOrKey::Identity(v)
    }
}

impl PartialEq<IdentityId> for IdentityOrKey {
    fn eq(&self, other: &IdentityId) -> bool {
        if let IdentityOrKey::Identity(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl PartialEq<Key> for IdentityOrKey {
    fn eq(&self, other: &Key) -> bool {
        if let IdentityOrKey::Key(v) = self {
            v == other
        } else {
            false
        }
    }
}

impl IdentityOrKey {
    /// Checks if IdentityOrKey to either an Identity or a key
    pub fn eq_either(&self, other_identity: &IdentityId, other_key: &Key) -> bool {
        if let IdentityOrKey::Identity(v) = self {
            v == other_identity
        } else if let IdentityOrKey::Key(v) = self {
            v == other_key
        } else {
            false
        }
    }
}
