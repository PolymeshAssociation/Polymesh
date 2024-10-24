// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::asset::AssetId;
use crate::{ExtrinsicName, IdentityId, PalletName, PortfolioId, SubsetRestriction};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::btree_map::BTreeMap,
    convert::TryInto,
    mem::size_of,
};

// We need to set a minimum complexity for pallet/extrinsic names
// to limit the total number of memory allocations.  Since each name
// requires an allocation.
//
// The average length of pallet/extrinsic names is 16.  So this
// minimum complexity only penalizes short names.
const MIN_NAME_COMPLEXITY: usize = 10;
fn name_complexity(name: &str) -> usize {
    // If the name length is lower then the minimum, then return the minimum.
    usize::max(name.len(), MIN_NAME_COMPLEXITY)
}

/// Asset permissions.
pub type AssetPermissions = SubsetRestriction<AssetId>;

/// A permission to call:
///
/// - specific functions, using `SubsetRestriction::These(_)`, or
///
/// - all but a specific set, using `SubsetRestriction::Except(_)`, or
///
/// - all functions, using `SubsetRestriction::Whole`
///
/// within some pallet.
pub type ExtrinsicNames = SubsetRestriction<ExtrinsicName>;

/// A permission to call a set of functions, as described by `extrinsics`,
/// within a given pallet `pallet_name`.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PalletPermissions {
    /// A subset of function names within the pallet.
    pub extrinsics: ExtrinsicNames,
}

impl PalletPermissions {
    /// Constructs new pallet permissions from given arguments.
    pub fn new(
        name: PalletName,
        extrinsics: SubsetRestriction<ExtrinsicName>,
    ) -> (PalletName, Self) {
        (name, PalletPermissions { extrinsics })
    }

    /// Helper method used in tests.
    pub fn entire_pallet(name: PalletName) -> (PalletName, Self) {
        (
            name,
            PalletPermissions {
                extrinsics: SubsetRestriction::Whole,
            },
        )
    }

    /// Constructs new pallet permissions for full access to a pallet.
    pub fn whole() -> Self {
        PalletPermissions {
            extrinsics: SubsetRestriction::Whole,
        }
    }

    /// Returns the complexity of the pallet permissions.
    pub fn complexity(&self) -> usize {
        self.extrinsics.fold(0usize, |cost, extrinsic_name| {
            cost.saturating_add(name_complexity(extrinsic_name))
        })
    }

    /// Return the number of extrinsic names.
    pub fn extrinsics_len(&self) -> usize {
        self.extrinsics.complexity()
    }
}

/// Extrinsic permissions.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ExtrinsicPermissions {
    /// Allow the whole pallet.
    #[default]
    Whole,
    /// Allow only these pallets.
    These(BTreeMap<PalletName, PalletPermissions>),
    /// Allow all pallets except these.
    Except(BTreeMap<PalletName, PalletPermissions>),
}

impl ExtrinsicPermissions {
    /// The empty permissions.
    pub fn empty() -> Self {
        Self::These(Default::default())
    }

    /// Constructs a permissions to allow everything except some pallets.
    pub fn except(it: impl IntoIterator<Item = (PalletName, PalletPermissions)>) -> Self {
        Self::Except(BTreeMap::from_iter(it))
    }

    /// Constructs a permissions to only allow these pallets.
    pub fn these(it: impl IntoIterator<Item = (PalletName, PalletPermissions)>) -> Self {
        Self::These(BTreeMap::from_iter(it))
    }

    /// Returns the complexity of the extrinsics permissions.
    pub fn complexity(&self) -> usize {
        self.inner()
            .map(|perms| {
                perms.iter().fold(0usize, |cost, (name, pallet)| {
                    cost.saturating_add(name_complexity(name))
                        .saturating_add(pallet.complexity())
                })
            })
            .unwrap_or(0)
    }

    /// Returns the length.
    pub fn len(&self) -> usize {
        self.inner().map(|perms| perms.len()).unwrap_or(0)
    }

    /// Returns the extrinsic count.
    pub fn extrinsic_count(&self) -> usize {
        self.inner()
            .map(|perms| {
                perms.values().fold(0usize, |cost, pallet| {
                    cost.saturating_add(pallet.extrinsics_len())
                })
            })
            .unwrap_or(0)
    }

    /// Returns `true` iff this permission set permits calling `pallet::extrinsic`.
    pub fn sufficient_for(&self, pallet: &PalletName, extrinsic: &ExtrinsicName) -> bool {
        let matches_any = |perms: &BTreeMap<PalletName, PalletPermissions>| {
            perms
                .get(pallet)
                .map(|perm| match &perm.extrinsics {
                    SubsetRestriction::Whole => true,
                    SubsetRestriction::These(funcs) => funcs.contains(extrinsic),
                    SubsetRestriction::Except(funcs) => !funcs.contains(extrinsic),
                })
                .unwrap_or(false)
        };
        match self {
            Self::Whole => true,
            Self::These(perms) => matches_any(perms),
            Self::Except(perms) => !matches_any(perms),
        }
    }

    /// Check that extrinsic permissions do not use the Except variant.
    pub fn check_no_except_perms(&self) -> bool {
        match self {
            Self::These(pallet_permissions) => {
                for (_, elem) in pallet_permissions {
                    if let SubsetRestriction::Except(_) = elem.extrinsics {
                        return false;
                    }
                }
                true
            }
            Self::Except(_) => false,
            Self::Whole => true,
        }
    }

    /// Returns the inner describing finite sets if any.
    pub fn inner(&self) -> Option<&BTreeMap<PalletName, PalletPermissions>> {
        match self {
            Self::Whole => None,
            Self::These(pallets) => Some(pallets),
            Self::Except(pallets) => Some(pallets),
        }
    }
}

/// Portfolio permissions.
pub type PortfolioPermissions = SubsetRestriction<PortfolioId>;

/// Signing key permissions.
///
/// Common cases of permissions:
/// - `Permissions::empty()`: no permissions,
/// - `Permissions::default()`: full permissions.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Permissions {
    /// The subset of assets under management.
    pub asset: AssetPermissions,
    /// The subset of callable extrinsics.
    pub extrinsic: ExtrinsicPermissions,
    /// The subset of portfolios management.
    pub portfolio: PortfolioPermissions,
}

impl Permissions {
    /// The empty permissions.
    pub fn empty() -> Self {
        Self {
            asset: SubsetRestriction::empty(),
            extrinsic: ExtrinsicPermissions::empty(),
            portfolio: SubsetRestriction::empty(),
        }
    }

    /// Empty permissions apart from given extrinsic permissions.
    pub fn from_pallet_permissions(
        pallet_permissions: impl IntoIterator<Item = (PalletName, PalletPermissions)>,
    ) -> Self {
        Self {
            asset: SubsetRestriction::empty(),
            extrinsic: ExtrinsicPermissions::These(pallet_permissions.into_iter().collect()),
            portfolio: SubsetRestriction::empty(),
        }
    }

    /// Returns the complexity of the permissions.
    pub fn complexity(&self) -> usize {
        // Calculate the pallet/extrinsic permissions complexity cost.
        let cost = self.extrinsic.complexity();

        // Asset permissions complexity cost.
        cost.saturating_add(self.asset.complexity().saturating_mul(size_of::<AssetId>()))
            // Portfolio permissions complexity cost.
            .saturating_add(
                self.portfolio
                    .complexity()
                    .saturating_mul(size_of::<PortfolioId>()),
            )
    }

    /// Return number of assets, portfolios, pallets, and extrinsics.
    ///
    /// This is used for weight calculation.
    pub fn counts(&self) -> (u32, u32, u32, u32) {
        // Count the number of assets.
        let assets = self.asset.complexity().try_into().unwrap_or(u32::MAX);
        // Count the number of portfolios.
        let portfolios = self.portfolio.complexity().try_into().unwrap_or(u32::MAX);
        // Count the number of pallets.
        let pallets = self.extrinsic.len().try_into().unwrap_or(u32::MAX);
        // Count the total number of extrinsics.
        let extrinsics = self
            .extrinsic
            .extrinsic_count()
            .try_into()
            .unwrap_or(u32::MAX);

        (assets, portfolios, pallets, extrinsics)
    }
}

/// Account key record.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum KeyRecord<AccountId> {
    /// Key is the primary key and has full permissions.
    ///
    /// (Key's identity)
    PrimaryKey(IdentityId),
    /// Key is a secondary key and can have restricted permissions.
    ///
    /// (Key's identity)
    SecondaryKey(IdentityId),
    /// Key is a MuliSig signer key.
    ///
    /// (MultiSig account id)
    MultiSigSignerKey(AccountId),
}

impl<AccountId> KeyRecord<AccountId> {
    /// Check if the key is the primary key and return the identity.
    pub fn is_primary_key(&self) -> Option<IdentityId> {
        if let Self::PrimaryKey(did) = self {
            Some(*did)
        } else {
            None
        }
    }

    /// Check if the key is the secondary key and return the identity.
    pub fn is_secondary_key(&self) -> Option<IdentityId> {
        if let Self::SecondaryKey(did) = self {
            Some(*did)
        } else {
            None
        }
    }

    /// Get the identity and the key type (primary/secondary).
    pub fn get_did_key_type(&self) -> Option<(IdentityId, bool)> {
        match self {
            Self::PrimaryKey(did) => Some((*did, true)),
            Self::SecondaryKey(did) => Some((*did, false)),
            _ => None,
        }
    }

    /// Extract the identity if it is a primary/secondary key.
    pub fn as_did(&self) -> Option<IdentityId> {
        match self {
            Self::PrimaryKey(did) | Self::SecondaryKey(did) => Some(*did),
            _ => None,
        }
    }
}

/// It supports different elements as a signer.
#[allow(missing_docs)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Signatory<AccountId> {
    #[cfg_attr(feature = "std", serde(alias = "identity"))]
    Identity(IdentityId),
    #[cfg_attr(feature = "std", serde(alias = "account"))]
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
    pub fn eq_either(&self, other_identity: Option<IdentityId>, other_key: &AccountId) -> bool {
        match self {
            Signatory::Account(ref key) => key == other_key,
            Signatory::Identity(ref id) => Some(*id) == other_identity,
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

/// A secondary key and its permissions.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SecondaryKey<AccountId> {
    /// The account key.
    pub key: AccountId,
    /// The access permissions of the `key`.
    pub permissions: Permissions,
}

impl<AccountId> SecondaryKey<AccountId> {
    /// Creates a [`SecondaryKey`].
    pub fn new(key: AccountId, permissions: Permissions) -> Self {
        Self { key, permissions }
    }

    /// Creates a [`SecondaryKey`] with no permissions from an `AccountId`.
    pub fn from_account_id(key: AccountId) -> Self {
        Self {
            key,
            // No permissions.
            permissions: Permissions::empty(),
        }
    }

    /// Creates a [`SecondaryKey`] with full permissions from an `AccountId`.
    pub fn from_account_id_with_full_perms(key: AccountId) -> Self {
        Self {
            key,
            // Full permissions.
            permissions: Permissions::default(),
        }
    }

    /// Checks if the given key has permission to access the given asset.
    pub fn has_asset_permission(&self, asset: AssetId) -> bool {
        self.permissions.asset.ge(&SubsetRestriction::elem(asset))
    }

    /// Checks if the given key has permission to call the given extrinsic.
    pub fn has_extrinsic_permission(&self, pallet: &PalletName, extrinsic: &ExtrinsicName) -> bool {
        self.permissions.extrinsic.sufficient_for(pallet, extrinsic)
    }

    /// Checks if the given key has permission to access all given portfolios.
    pub fn has_portfolio_permission(&self, it: impl IntoIterator<Item = PortfolioId>) -> bool {
        self.permissions.portfolio.ge(&SubsetRestriction::elems(it))
    }

    /// Returns the complexity of the secondary key's permissions.
    pub fn complexity(&self) -> usize {
        self.permissions.complexity()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ExtrinsicPermissions, Permissions, PortfolioId, SecondaryKey, Signatory, SubsetRestriction,
    };
    use crate::{asset::AssetId, IdentityId};
    use sp_core::sr25519::Public;
    use std::convert::{From, TryFrom};

    #[test]
    fn build_test() {
        let key = Public::from_raw([b'A'; 32]);
        let rk1 = SecondaryKey::new(key.clone(), Permissions::empty());
        let rk2 = SecondaryKey::from_account_id(key.clone());
        assert_eq!(rk1, rk2);

        let rk3_permissions = Permissions {
            asset: SubsetRestriction::elem(AssetId::new([0; 16])),
            extrinsic: ExtrinsicPermissions::Whole,
            portfolio: SubsetRestriction::elem(PortfolioId::default_portfolio(IdentityId::from(
                1u128,
            ))),
        };
        let rk3 = SecondaryKey::new(key.clone(), rk3_permissions.clone());
        assert_ne!(rk1, rk3);

        let mut rk4 = SecondaryKey::from_account_id(key);
        rk4.permissions = rk3_permissions;
        assert_eq!(rk3, rk4);
    }

    #[test]
    fn has_permission_test() {
        let key = Public::from_raw([b'A'; 32]);
        let asset_id = AssetId::new([0; 16]);
        let asset_id2 = AssetId::new([1; 16]);
        let portfolio1 = PortfolioId::user_portfolio(IdentityId::default(), 1.into());
        let portfolio2 = PortfolioId::user_portfolio(IdentityId::default(), 2.into());
        let permissions = Permissions {
            asset: SubsetRestriction::elem(AssetId::new([0; 16])),
            extrinsic: ExtrinsicPermissions::Whole,
            portfolio: SubsetRestriction::elem(portfolio1),
        };
        let free_key = SecondaryKey::new(key.clone(), Permissions::default());
        let restricted_key = SecondaryKey::new(key, permissions.clone());
        assert!(free_key.has_asset_permission(asset_id2));
        assert!(free_key.has_extrinsic_permission(&"pallet".into(), &"function".into()));
        assert!(free_key.has_portfolio_permission(vec![portfolio1]));
        assert!(restricted_key.has_asset_permission(asset_id));
        assert!(!restricted_key.has_asset_permission(asset_id2));
        assert!(restricted_key.has_extrinsic_permission(&"pallet".into(), &"function".into()));
        assert!(restricted_key.has_portfolio_permission(vec![portfolio1]));
        assert!(!restricted_key.has_portfolio_permission(vec![portfolio2]));
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
