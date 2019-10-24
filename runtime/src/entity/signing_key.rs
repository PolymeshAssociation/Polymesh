use crate::entity::Key;
use rstd::{prelude::Vec, vec};

// use crate::entity::IgnoredCaseString;

/// Identity roles.
/// # TODO
/// 2. Review documents:
///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)
///     - [Polymesh: Roles/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)
///
#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum KeyRole {
    Full,
    Admin,
    Operator,
}

/// Signing key type.
/// See [MESH-378](https://polymath.atlassian.net/browse/MESH-378)
#[derive(codec::Encode, codec::Decode, Clone, PartialEq, Eq, Debug)]
pub enum SigningKeyType {
    External,
    Identity,
    Multisig,
    Relayer,
}

impl Default for SigningKeyType {
    fn default() -> Self {
        SigningKeyType::External
    }
}

/// A signing key contains a type and a group of roles.
#[derive(codec::Encode, codec::Decode, Default, Clone, Eq, Debug)]
pub struct SigningKey {
    pub key: Key,
    pub key_type: SigningKeyType,
    pub roles: Vec<KeyRole>,
}

impl SigningKey {
    pub fn new(key: Key, roles: Vec<KeyRole>) -> Self {
        Self {
            key,
            key_type: SigningKeyType::External,
            roles,
        }
    }

    /// It checks if this key has specified `role` role.
    /// Role `KeyRole::Full` is special and denotates that this key can be used for any role.
    pub fn has_role(&self, role: KeyRole) -> bool {
        self.roles
            .iter()
            .find(|&r| role == *r || *r == KeyRole::Full)
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
        self.key == other.key && self.key_type == other.key_type && self.roles == other.roles
    }
}

impl PartialEq<Key> for SigningKey {
    fn eq(&self, other: &Key) -> bool {
        self.key == *other
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, KeyRole, SigningKey};
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let rk1 = SigningKey::new(key.clone(), vec![]);
        let rk2 = SigningKey::from(key.clone());
        assert_eq!(rk1, rk2);

        let rk3 = SigningKey::new(key.clone(), vec![KeyRole::Operator, KeyRole::Admin]);
        assert_ne!(rk1, rk3);

        let mut rk4 = SigningKey::from(key);
        rk4.roles = vec![KeyRole::Operator, KeyRole::Admin];
        assert_eq!(rk3, rk4);
    }

    #[test]
    fn full_role_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let full_key = SigningKey::new(key.clone(), vec![KeyRole::Full]);
        let not_full_key = SigningKey::new(key, vec![KeyRole::Operator]);
        assert_eq!(full_key.has_role(KeyRole::Operator), true);
        assert_eq!(full_key.has_role(KeyRole::Admin), true);

        assert_eq!(not_full_key.has_role(KeyRole::Operator), true);
        assert_eq!(not_full_key.has_role(KeyRole::Admin), false);
    }
}
