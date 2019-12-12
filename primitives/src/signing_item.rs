use codec::{Decode, Encode};
use rstd::{
    cmp::{Ord, Ordering, PartialOrd},
    prelude::Vec,
    vec,
};

use crate::{IdentityId, Key};

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

/// Signing key type.
#[allow(missing_docs)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
pub enum SignerType {
    External,
    Identity,
    Multisig,
    Relayer,
    Custom(u8),
}

impl Default for SignerType {
    fn default() -> Self {
        SignerType::External
    }
}

/// It supports different elements as a signer.
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum Signer {
    Identity(IdentityId),
    Key(Key),
}

impl Default for Signer {
    fn default() -> Self {
        Signer::Identity(IdentityId::default())
    }
}

impl PartialEq<Key> for Signer {
    fn eq(&self, other: &Key) -> bool {
        match self {
            Signer::Key(ref key) => key == other,
            _ => false,
        }
    }
}

impl PartialEq<IdentityId> for Signer {
    fn eq(&self, other: &IdentityId) -> bool {
        match self {
            Signer::Identity(ref id) => id == other,
            _ => false,
        }
    }
}

impl PartialOrd for Signer {
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Signer {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Signer::Identity(id) => match other {
                Signer::Identity(other_id) => id.cmp(other_id),
                Signer::Key(..) => Ordering::Greater,
            },
            Signer::Key(key) => match other {
                Signer::Key(other_key) => key.cmp(other_key),
                Signer::Identity(..) => Ordering::Less,
            },
        }
    }
}

/// A signing key contains a type and a group of permissions.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, Eq, Debug)]
pub struct SigningItem {
    pub signer: Signer,
    pub signer_type: SignerType,
    pub permissions: Vec<Permission>,
}

impl SigningItem {
    /// It creates an 'External' signing key.
    pub fn new(signer: Signer, permissions: Vec<Permission>) -> Self {
        Self {
            signer,
            signer_type: SignerType::External,
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

impl From<Key> for SigningItem {
    fn from(s: Key) -> Self {
        Self::new(Signer::Key(s), vec![])
    }
}

impl From<IdentityId> for SigningItem {
    fn from(id: IdentityId) -> Self {
        Self::new(Signer::Identity(id), vec![])
    }
}

impl PartialEq for SigningItem {
    fn eq(&self, other: &Self) -> bool {
        self.signer == other.signer
            && self.signer_type == other.signer_type
            && self.permissions == other.permissions
    }
}

impl PartialEq<Key> for SigningItem {
    fn eq(&self, other: &Key) -> bool {
        self.signer == *other
    }
}

impl PartialEq<IdentityId> for SigningItem {
    fn eq(&self, other: &IdentityId) -> bool {
        self.signer == *other
    }
}

impl PartialOrd for SigningItem {
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.signer.partial_cmp(&other.signer)
    }
}

impl Ord for SigningItem {
    fn cmp(&self, other: &Self) -> Ordering {
        self.signer.cmp(&other.signer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, Permission, Signer, SigningItem};
    use crate::IdentityId;
    use std::convert::{From, TryFrom};

    #[test]
    fn build_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let rk1 = SigningItem::new(Signer::Key(key.clone()), vec![]);
        let rk2 = SigningItem::from(key.clone());
        assert_eq!(rk1, rk2);

        let rk3 = SigningItem::new(
            Signer::Key(key.clone()),
            vec![Permission::Operator, Permission::Admin],
        );
        assert_ne!(rk1, rk3);

        let mut rk4 = SigningItem::from(key);
        rk4.permissions = vec![Permission::Operator, Permission::Admin];
        assert_eq!(rk3, rk4);

        let si1 = SigningItem::from(IdentityId::from(1u128));
        let si2 = SigningItem::from(IdentityId::from(1u128));
        assert_eq!(si1, si2);

        let si3 = SigningItem::from(IdentityId::from(2u128));
        assert_ne!(si1, si3);

        assert_ne!(si1, rk1);
    }

    #[test]
    fn full_permission_test() {
        let key = Key::try_from("ABCDABCD".as_bytes()).unwrap();
        let full_key = SigningItem::new(Signer::Key(key.clone()), vec![Permission::Full]);
        let not_full_key = SigningItem::new(Signer::Key(key), vec![Permission::Operator]);
        assert_eq!(full_key.has_permission(Permission::Operator), true);
        assert_eq!(full_key.has_permission(Permission::Admin), true);

        assert_eq!(not_full_key.has_permission(Permission::Operator), true);
        assert_eq!(not_full_key.has_permission(Permission::Admin), false);
    }
}
