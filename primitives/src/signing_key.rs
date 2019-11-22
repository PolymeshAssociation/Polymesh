use parity_scale_codec::{Decode, Encode};
use rstd::{
    cmp::{Ord, Ordering, PartialOrd},
    prelude::Vec,
    vec,
};

use crate::{Key, KeyType};

// use crate::entity::IgnoredCaseString;

/// Key permissions.
/// # TODO
/// 2. Review documents:
///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)
///     - [Polymesh: Roles/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Permission {
    Full,
    Admin,
    Operator,
    SpendFunds,
    Custom(u8),
}

/// A signing key contains a type and a group of permissions.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, Eq, Debug)]
pub struct SigningKey {
    pub key: Key,
    pub key_type: KeyType,
    pub permissions: Vec<Permission>,
}

impl SigningKey {
    /// It creates an 'External' signing key.
    pub fn new(key: Key, permissions: Vec<Permission>) -> Self {
        Self {
            key,
            key_type: KeyType::External,
            permissions,
        }
    }

    /// It checks if this key has specified `permission` permission.
    /// permission `Permission::Full` is special and denotates that this key can be used for any permission.
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions
            .iter()
            .find(|&r| permission == *r || *r == Permission::Full)
            .is_some()
    }
}

impl From<Key> for SigningKey {
    fn from(s: Key) -> Self {
        Self::new(s, vec![])
    }
}

impl PartialEq for SigningKey {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key
            && self.key_type == other.key_type
            && self.permissions == other.permissions
    }
}

impl PartialEq<Key> for SigningKey {
    fn eq(&self, other: &Key) -> bool {
        self.key == *other
    }
}

impl PartialOrd for SigningKey {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.key.cmp(&other.key))
    }
}

impl Ord for SigningKey {
    fn cmp(&self, other: &Self) -> Ordering {
        self.key.cmp(&other.key)
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Permission, SigningKey};
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let rk1 = SigningKey::new(key.clone(), vec![]);
        let rk2 = SigningKey::from(key.clone());
        assert_eq!(rk1, rk2);

        let rk3 = SigningKey::new(key.clone(), vec![Permission::Operator, Permission::Admin]);
        assert_ne!(rk1, rk3);

        let mut rk4 = SigningKey::from(key);
        rk4.permissions = vec![Permission::Operator, Permission::Admin];
        assert_eq!(rk3, rk4);
    }

    #[test]
    fn full_permission_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let full_key = SigningKey::new(key.clone(), vec![Permission::Full]);
        let not_full_key = SigningKey::new(key, vec![Permission::Operator]);
        assert_eq!(full_key.has_permission(Permission::Operator), true);
        assert_eq!(full_key.has_permission(Permission::Admin), true);

        assert_eq!(not_full_key.has_permission(Permission::Operator), true);
        assert_eq!(not_full_key.has_permission(Permission::Admin), false);
    }
}
