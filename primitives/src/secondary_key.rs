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

use crate::IdentityId;
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

/// It supports different elements as a signer.
#[allow(missing_docs)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Signatory<AccountId> {
    Identity(IdentityId),
    Account(AccountId),
}

impl<AccountId> Default for Signatory<AccountId> {
    fn default() -> Self {
        Signatory::Identity(IdentityId::default())
    }
}

impl<AccountId> From<IdentityId> for Signatory<AccountId> {
    fn from(v: IdentityId) -> Self {
        Signatory::Identity(v)
    }
}

impl<AccountId> PartialEq<IdentityId> for Signatory<AccountId> {
    fn eq(&self, other: &IdentityId) -> bool {
        match self {
            Signatory::Identity(ref id) => id == other,
            _ => false,
        }
    }
}

impl<AccountId> Signatory<AccountId>
where
    AccountId: PartialEq,
{
    /// Checks if Signatory is either a particular Identity or a particular key
    pub fn eq_either(&self, other_identity: &IdentityId, other_key: &AccountId) -> bool {
        match self {
            Signatory::Account(ref key) => key == other_key,
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

    /// This signatory as `AccountId` or None.
    pub fn as_account(&self) -> Option<&AccountId> {
        match self {
            Signatory::Account(key) => Some(key),
            _ => None,
        }
    }
}

impl<AccountId> PartialOrd for Signatory<AccountId>
where
    AccountId: Ord,
{
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<AccountId> Ord for Signatory<AccountId>
where
    AccountId: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self {
            Signatory::Identity(id) => match other {
                Signatory::Identity(other_id) => id.cmp(other_id),
                Signatory::Account(..) => Ordering::Greater,
            },
            Signatory::Account(key) => match other {
                Signatory::Account(other_key) => key.cmp(other_key),
                Signatory::Identity(..) => Ordering::Less,
            },
        }
    }
}

/// A secondary key contains a type and a group of permissions.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SecondaryKey<AccountId> {
    pub signer: Signatory<AccountId>,
    pub permissions: Vec<Permission>,
}

impl<AccountId> SecondaryKey<AccountId> {
    /// It creates an 'External' secondary key.
    pub fn new(signer: Signatory<AccountId>, permissions: Vec<Permission>) -> Self {
        Self {
            signer,
            permissions,
        }
    }

    /// Creates a [`SecondaryKey`] from an `AccountId`.
    pub fn from_account_id(s: AccountId) -> Self {
        Self::new(Signatory::Account(s), vec![])
    }

    /// It checks if this key has specified `permission` permission.
    /// permission `Permission::Full` is special and denotates that this key can be used for any permission.
    pub fn has_permission(&self, permission: Permission) -> bool {
        self.permissions
            .iter()
            .any(|r| permission == *r || *r == Permission::Full)
    }
}

impl<AccountId> From<IdentityId> for SecondaryKey<AccountId> {
    fn from(id: IdentityId) -> Self {
        Self::new(Signatory::Identity(id), vec![])
    }
}

impl<AccountId> PartialEq for SecondaryKey<AccountId>
where
    AccountId: PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.signer == other.signer && self.permissions == other.permissions
    }
}

impl<AccountId> PartialEq<IdentityId> for SecondaryKey<AccountId> {
    fn eq(&self, other: &IdentityId) -> bool {
        if let Signatory::Identity(id) = self.signer {
            id == *other
        } else {
            false
        }
    }
}

impl<AccountId> PartialOrd for SecondaryKey<AccountId>
where
    AccountId: Ord,
{
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.signer.partial_cmp(&other.signer)
    }
}

impl<AccountId> Ord for SecondaryKey<AccountId>
where
    AccountId: Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.signer.cmp(&other.signer)
    }
}

#[cfg(test)]
mod tests {
    use super::{Permission, SecondaryKey, Signatory};
    use crate::IdentityId;
    use sp_core::sr25519::Public;
    use std::convert::{From, TryFrom};

    #[test]
    fn build_test() {
        let key = Public::from_raw([b'A'; 32]);
        let rk1 = SecondaryKey::new(Signatory::Account(key.clone()), vec![]);
        let rk2 = SecondaryKey::from_account_id(key.clone());
        assert_eq!(rk1, rk2);

        let rk3 = SecondaryKey::new(
            Signatory::Account(key.clone()),
            vec![Permission::Operator, Permission::Admin],
        );
        assert_ne!(rk1, rk3);

        let mut rk4 = SecondaryKey::from_account_id(key);
        rk4.permissions = vec![Permission::Operator, Permission::Admin];
        assert_eq!(rk3, rk4);

        let si1 = SecondaryKey::from(IdentityId::from(1u128));
        let si2 = SecondaryKey::from(IdentityId::from(1u128));
        assert_eq!(si1, si2);

        let si3 = SecondaryKey::from(IdentityId::from(2u128));
        assert_ne!(si1, si3);

        assert_ne!(si1, rk1);
    }

    #[test]
    fn full_permission_test() {
        let key = Public::from_raw([b'A'; 32]);
        let full_key = SecondaryKey::new(Signatory::Account(key.clone()), vec![Permission::Full]);
        let not_full_key = SecondaryKey::new(Signatory::Account(key), vec![Permission::Operator]);
        assert_eq!(full_key.has_permission(Permission::Operator), true);
        assert_eq!(full_key.has_permission(Permission::Admin), true);

        assert_eq!(not_full_key.has_permission(Permission::Operator), true);
        assert_eq!(not_full_key.has_permission(Permission::Admin), false);
    }

    #[test]
    fn signer_build_and_eq_tests() {
        let key = Public::from_raw([b'A'; 32]);
        let iden = IdentityId::try_from(
            "did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976",
        )
        .unwrap();
        let iden_sig: Signatory<sp_core::sr25519::Public> = Signatory::from(iden);
        assert_ne!(Signatory::Account(key), iden_sig);
        assert_eq!(iden_sig, iden);
        assert_ne!(iden_sig, Signatory::Account(key));
    }
}
