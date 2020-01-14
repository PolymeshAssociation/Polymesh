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

#[cfg(test)]
mod tests {
    use super::*;
    use std::convert::TryFrom;

    #[test]
    fn build_and_eq_tests() {
        let k = "ABCDABCD".as_bytes().to_vec();
        let key = Key::try_from(k.as_slice()).unwrap();
        let iden = IdentityId::try_from(
            "did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976",
        )
        .unwrap();
        assert_eq!(IdentityOrKey::from(key), key);
        assert_ne!(IdentityOrKey::from(key), iden);
        assert_eq!(IdentityOrKey::from(iden), iden);
        assert_ne!(IdentityOrKey::from(iden), key);
    }
}
