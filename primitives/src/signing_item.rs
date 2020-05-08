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

use crate::{AccountKey, IdentityId};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    cmp::{Ord, Ordering, PartialOrd},
    prelude::Vec,
    vec,
};

// use crate::entity::IgnoredCaseString;

/// Key permissions.
/// # TODO
/// 2. Review documents:
///     - [MESH-235](https://polymath.atlassian.net/browse/MESH-235)
///     - [Polymesh: Roles/Permissions](https://docs.google.com/document/d/12u-rMavow4fvidsFlLcLe7DAXuqWk8XUHOBV9kw05Z8/)
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SignatoryType {
    External,
    Identity,
    MultiSig,
    Relayer,
    Custom(u8),
}

impl Default for SignatoryType {
    fn default() -> Self {
        SignatoryType::External
    }
}

/// It supports different elements as a signer.
#[allow(missing_docs)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Signatory {
    Identity(IdentityId),
    AccountKey(AccountKey),
}

impl Default for Signatory {
    fn default() -> Self {
        Signatory::Identity(IdentityId::default())
    }
}

impl From<AccountKey> for Signatory {
    fn from(v: AccountKey) -> Self {
        Signatory::AccountKey(v)
    }
}

impl From<IdentityId> for Signatory {
    fn from(v: IdentityId) -> Self {
        Signatory::Identity(v)
    }
}

impl PartialEq<AccountKey> for Signatory {
    fn eq(&self, other: &AccountKey) -> bool {
        match self {
            Signatory::AccountKey(ref key) => key == other,
            _ => false,
        }
    }
}

impl PartialEq<IdentityId> for Signatory {
    fn eq(&self, other: &IdentityId) -> bool {
        match self {
            Signatory::Identity(ref id) => id == other,
            _ => false,
        }
    }
}

impl Signatory {
    /// Checks if Signatory is either a particular Identity or a particular key
    pub fn eq_either(&self, other_identity: &IdentityId, other_key: &AccountKey) -> bool {
        match self {
            Signatory::AccountKey(ref key) => key == other_key,
            Signatory::Identity(ref id) => id == other_identity,
        }
    }

    /// This signatory as `IdentityId` or None.
    pub fn as_identity(&self) -> Option<&IdentityId> {
        match self {
            Signatory::Identity(id) => Some(id),
            _ => None,
        }
    }

    /// This signatory as `AccountKey` or None.
    pub fn as_account_key(&self) -> Option<&AccountKey> {
        match self {
            Signatory::AccountKey(key) => Some(key),
            _ => None,
        }
    }
}

impl PartialOrd for Signatory {
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Signatory {
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Signatory::Identity(id) => match other {
                Signatory::Identity(other_id) => id.cmp(other_id),
                Signatory::AccountKey(..) => Ordering::Greater,
            },
            Signatory::AccountKey(key) => match other {
                Signatory::AccountKey(other_key) => key.cmp(other_key),
                Signatory::Identity(..) => Ordering::Less,
            },
        }
    }
}

/// A signing key contains a type and a group of permissions.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SigningItem {
    pub signer: Signatory,
    pub signer_type: SignatoryType,
    pub permissions: Vec<Permission>,
}

impl SigningItem {
    /// It creates an 'External' signing key.
    pub fn new(signer: Signatory, permissions: Vec<Permission>) -> Self {
        Self {
            signer,
            signer_type: SignatoryType::External,
            permissions,
        }
    }

    /// It checks if this key has specified `permission` permission.
    /// permission `Permission::Full` is special and denotates that this key can be used for any permission.
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions
            .iter()
            .any(|r| permission == *r || *r == Permission::Full)
    }
}

impl From<AccountKey> for SigningItem {
    fn from(s: AccountKey) -> Self {
        Self::new(Signatory::AccountKey(s), vec![])
    }
}

impl From<IdentityId> for SigningItem {
    fn from(id: IdentityId) -> Self {
        Self::new(Signatory::Identity(id), vec![])
    }
}

impl PartialEq for SigningItem {
    fn eq(&self, other: &Self) -> bool {
        self.signer == other.signer
            && self.signer_type == other.signer_type
            && self.permissions == other.permissions
    }
}

impl PartialEq<AccountKey> for SigningItem {
    fn eq(&self, other: &AccountKey) -> bool {
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
    use super::{AccountKey, Permission, Signatory, SigningItem};
    use crate::IdentityId;
    use std::convert::{From, TryFrom};

    #[test]
    fn build_test() {
        let key = AccountKey::try_from("ABCDABCD".as_bytes()).unwrap();
        let rk1 = SigningItem::new(Signatory::AccountKey(key.clone()), vec![]);
        let rk2 = SigningItem::from(key.clone());
        assert_eq!(rk1, rk2);

        let rk3 = SigningItem::new(
            Signatory::AccountKey(key.clone()),
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
        let key = AccountKey::try_from("ABCDABCD".as_bytes()).unwrap();
        let full_key = SigningItem::new(Signatory::AccountKey(key.clone()), vec![Permission::Full]);
        let not_full_key = SigningItem::new(Signatory::AccountKey(key), vec![Permission::Operator]);
        assert_eq!(full_key.has_permission(Permission::Operator), true);
        assert_eq!(full_key.has_permission(Permission::Admin), true);

        assert_eq!(not_full_key.has_permission(Permission::Operator), true);
        assert_eq!(not_full_key.has_permission(Permission::Admin), false);
    }

    #[test]
    fn signer_build_and_eq_tests() {
        let k = "ABCDABCD".as_bytes().to_vec();
        let key = AccountKey::try_from(k.as_slice()).unwrap();
        let iden = IdentityId::try_from(
            "did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976",
        )
        .unwrap();
        assert_eq!(Signatory::from(key), key);
        assert_ne!(Signatory::from(key), iden);
        assert_eq!(Signatory::from(iden), iden);
        assert_ne!(Signatory::from(iden), key);
    }
}
