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

use crate::{DispatchableName, IdentityId, PalletName, PortfolioId, SubsetRestriction, Ticker};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    cmp::{Ord, Ordering, PartialOrd},
    collections::btree_set::BTreeSet,
    iter,
    mem::size_of,
};

// We need to set a minimum complexity for pallet/dispatchable names
// to limit the total number of memory allocations.  Since each name
// requires an allocation.
//
// The average length of pallet/dispatchable names is 16.  So this
// minimum complexity only penalizes short names.
const MIN_NAME_COMPLEXITY: usize = 10;
fn name_complexity(name: &[u8]) -> usize {
    // If the name length is lower then the minimum, then return the minimum.
    usize::max(name.len(), MIN_NAME_COMPLEXITY)
}

/// Asset permissions.
pub type AssetPermissions = SubsetRestriction<Ticker>;

/// A permission to call:
///
/// - specific functions, using `SubsetRestriction::These(_)`, or
///
/// - all but a specific set, using `SubsetRestriction::Except(_)`, or
///
/// - all functions, using `SubsetRestriction::Whole`
///
/// within some pallet.
pub type DispatchableNames = SubsetRestriction<DispatchableName>;

/// A permission to call a set of functions, as described by `dispatchable_names`,
/// within a given pallet `pallet_name`.
#[derive(Decode, Encode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PalletPermissions {
    /// The name of a pallet.
    pub pallet_name: PalletName,
    /// A subset of function names within the pallet.
    pub dispatchable_names: DispatchableNames,
}

impl PalletPermissions {
    /// Constructs new pallet permissions from given arguments.
    pub fn new(
        pallet_name: PalletName,
        dispatchable_names: SubsetRestriction<DispatchableName>,
    ) -> Self {
        PalletPermissions {
            pallet_name,
            dispatchable_names,
        }
    }

    /// Constructs new pallet permissions for full access to pallet `pallet_name`.
    pub fn entire_pallet(pallet_name: PalletName) -> Self {
        PalletPermissions {
            pallet_name,
            dispatchable_names: SubsetRestriction::Whole,
        }
    }

    /// Returns the complexity of the pallet permissions.
    pub fn complexity(&self) -> usize {
        self.dispatchable_names
            .fold(name_complexity(&self.pallet_name), |cost, dispatch_name| {
                cost.saturating_add(name_complexity(&dispatch_name))
            })
    }
}

/// Extrinsic permissions.
pub type ExtrinsicPermissions = SubsetRestriction<PalletPermissions>;

impl ExtrinsicPermissions {
    /// Returns `true` iff this permission set permits calling `pallet::dispatchable`.
    pub fn sufficient_for(&self, pallet: &PalletName, dispatchable: &DispatchableName) -> bool {
        let matches_any = |perms: &BTreeSet<PalletPermissions>| {
            perms.iter().any(|perm| {
                if &perm.pallet_name != pallet {
                    return false;
                }
                match &perm.dispatchable_names {
                    SubsetRestriction::Whole => true,
                    SubsetRestriction::These(funcs) => funcs.contains(dispatchable),
                    SubsetRestriction::Except(funcs) => !funcs.contains(dispatchable),
                }
            })
        };
        match self {
            SubsetRestriction::Whole => true,
            SubsetRestriction::These(perms) => matches_any(perms),
            SubsetRestriction::Except(perms) => !matches_any(perms),
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
#[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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
            extrinsic: SubsetRestriction::empty(),
            portfolio: SubsetRestriction::empty(),
        }
    }

    /// Empty permissions apart from given extrinsic permissions.
    pub fn from_pallet_permissions(
        pallet_permissions: impl IntoIterator<Item = PalletPermissions>,
    ) -> Self {
        Self {
            asset: SubsetRestriction::empty(),
            extrinsic: SubsetRestriction::These(pallet_permissions.into_iter().collect()),
            portfolio: SubsetRestriction::empty(),
        }
    }

    /// Adds extra extrinsic permissions to `self` for just one pallet. The result is stored in
    /// `self`.
    pub fn add_pallet_permissions(&mut self, pallet_permissions: PalletPermissions) {
        self.extrinsic = self.extrinsic.union(&SubsetRestriction::These(
            iter::once(pallet_permissions).collect(),
        ));
    }

    /// Returns the complexity of the permissions.
    pub fn complexity(&self) -> usize {
        // Calculate the pallet/extrinsic permissions complexity cost.
        let cost = self.extrinsic.fold(0usize, |cost, pallet| {
            cost.saturating_add(pallet.complexity())
        });

        // Asset permissions complexity cost.
        cost.saturating_add(self.asset.complexity().saturating_mul(size_of::<Ticker>()))
            // Portfolio permissions complexity cost.
            .saturating_add(
                self.portfolio
                    .complexity()
                    .saturating_mul(size_of::<PortfolioId>()),
            )
    }
}

/// It supports different elements as a signer.
#[allow(missing_docs)]
#[derive(Encode, Decode, Copy, Clone, PartialEq, Eq, Debug)]
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

/// A secondary key is a signatory with defined permissions.
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SecondaryKey<AccountId: Encode + Decode> {
    /// The account or identity that is the signatory of this key.
    pub signer: Signatory<AccountId>,
    /// The access permissions of the signing key.
    pub permissions: Permissions,
}

impl<AccountId> SecondaryKey<AccountId>
where
    AccountId: Encode + Decode,
{
    /// Creates a [`SecondaryKey`].
    pub fn new(signer: Signatory<AccountId>, permissions: Permissions) -> Self {
        Self {
            signer,
            permissions,
        }
    }

    /// Creates a [`SecondaryKey`] from an `AccountId`.
    pub fn from_account_id(s: AccountId) -> Self {
        Self {
            signer: Signatory::Account(s),
            // No permissions.
            permissions: Permissions::empty(),
        }
    }

    /// Creates a [`SecondaryKey`] with full permissions from an `AccountId`.
    pub fn from_account_id_with_full_perms(s: AccountId) -> Self {
        Self {
            signer: Signatory::Account(s),
            // Full permissions.
            permissions: Permissions::default(),
        }
    }

    /// Checks if the given key has permission to access the given asset.
    pub fn has_asset_permission(&self, asset: Ticker) -> bool {
        self.permissions.asset.ge(&SubsetRestriction::elem(asset))
    }

    /// Checks if the given key has permission to call the given extrinsic.
    pub fn has_extrinsic_permission(
        &self,
        pallet: &PalletName,
        dispatchable: &DispatchableName,
    ) -> bool {
        self.permissions
            .extrinsic
            .sufficient_for(pallet, dispatchable)
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

impl<AccountId> From<IdentityId> for SecondaryKey<AccountId>
where
    AccountId: Encode + Decode,
{
    fn from(id: IdentityId) -> Self {
        Self {
            signer: Signatory::Identity(id),
            permissions: Permissions::empty(),
        }
    }
}

impl<AccountId> PartialEq<IdentityId> for SecondaryKey<AccountId>
where
    AccountId: Encode + Decode,
{
    fn eq(&self, other: &IdentityId) -> bool {
        if let Signatory::Identity(id) = self.signer {
            id == *other
        } else {
            false
        }
    }
}

/// Hacks to workaround substrate and Polkadot.js restrictions/bugs.
pub mod api {
    use super::{
        AssetPermissions, ExtrinsicPermissions, PalletPermissions, Permissions,
        PortfolioPermissions,
    };
    use crate::{DispatchableName, PalletName, Signatory, SubsetRestriction};
    use codec::{Decode, Encode};
    #[cfg(feature = "std")]
    use sp_runtime::{Deserialize, Serialize};
    use sp_std::vec::Vec;

    /// A permission to call functions within a given pallet.
    #[derive(Decode, Encode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct LegacyPalletPermissions {
        /// The name of a pallet.
        pub pallet_name: PalletName,
        /// A workaround for https://github.com/polkadot-js/apps/issues/3632.
        ///
        /// - `total == false` - only the functions listed in `dispatchable_names` are allowed to be
        /// called.
        ///
        /// - `total == true` - `dispatchable_names` is ignored. Such permissions allow any function in
        /// `pallet_name` to be called.
        pub total: bool,
        /// A subset of function names within the pallet taken into account when `total == false`.
        pub dispatchable_names: Vec<DispatchableName>,
    }

    /// Extrinsic permissions.
    #[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct LegacyExtrinsicPermissions(pub Option<Vec<LegacyPalletPermissions>>);

    /// Signing key permissions.
    #[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct LegacyPermissions {
        /// The subset of assets under management.
        pub asset: AssetPermissions,
        /// The subset of callable extrinsics.
        pub extrinsic: LegacyExtrinsicPermissions,
        /// The subset of portfolios management.
        pub portfolio: PortfolioPermissions,
    }

    impl LegacyPermissions {
        /// The empty permissions.
        pub fn empty() -> Self {
            Self {
                asset: SubsetRestriction::empty(),
                extrinsic: LegacyExtrinsicPermissions(Some(Vec::new())),
                portfolio: SubsetRestriction::empty(),
            }
        }
    }

    /// The same secondary key object without the extra trait constraints.
    /// It is needed because it's not possible to define `decl_event!`
    /// with the required restrictions on `AccountId`
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct SecondaryKey<AccountId> {
        /// The account or identity that is the signatory of this key.
        pub signer: Signatory<AccountId>,
        /// The access permissions of the signing key.
        pub permissions: Permissions,
    }

    impl<AccountId> From<super::SecondaryKey<AccountId>> for SecondaryKey<AccountId>
    where
        AccountId: Encode + Decode,
    {
        fn from(k: super::SecondaryKey<AccountId>) -> SecondaryKey<AccountId> {
            SecondaryKey {
                signer: k.signer,
                permissions: k.permissions,
            }
        }
    }

    impl From<LegacyPalletPermissions> for PalletPermissions {
        fn from(p: LegacyPalletPermissions) -> PalletPermissions {
            PalletPermissions {
                pallet_name: p.pallet_name,
                dispatchable_names: if p.total {
                    SubsetRestriction::Whole
                } else {
                    SubsetRestriction::These(p.dispatchable_names.into_iter().collect())
                },
            }
        }
    }

    impl From<LegacyExtrinsicPermissions> for ExtrinsicPermissions {
        fn from(p: LegacyExtrinsicPermissions) -> ExtrinsicPermissions {
            match p.0 {
                Some(elems) => {
                    SubsetRestriction::These(elems.into_iter().map(|e| e.into()).collect())
                }
                None => SubsetRestriction::Whole,
            }
        }
    }

    impl From<LegacyPermissions> for Permissions {
        fn from(p: LegacyPermissions) -> Permissions {
            Permissions {
                asset: p.asset.into(),
                extrinsic: p.extrinsic.into(),
                portfolio: p.portfolio.into(),
            }
        }
    }

    impl<AccountId> From<SecondaryKey<AccountId>> for super::SecondaryKey<AccountId>
    where
        AccountId: Encode + Decode,
    {
        fn from(k: SecondaryKey<AccountId>) -> super::SecondaryKey<AccountId> {
            super::SecondaryKey {
                signer: k.signer,
                permissions: k.permissions,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Permissions, PortfolioId, SecondaryKey, Signatory, SubsetRestriction};
    use crate::{IdentityId, Ticker};
    use sp_core::sr25519::Public;
    use std::convert::{From, TryFrom};

    #[test]
    fn build_test() {
        let key = Public::from_raw([b'A'; 32]);
        let rk1 = SecondaryKey::new(Signatory::Account(key.clone()), Permissions::empty());
        let rk2 = SecondaryKey::from_account_id(key.clone());
        assert_eq!(rk1, rk2);

        let rk3_permissions = Permissions {
            asset: SubsetRestriction::elem(Ticker::try_from(&[1][..]).unwrap()),
            extrinsic: SubsetRestriction::Whole,
            portfolio: SubsetRestriction::elem(PortfolioId::default_portfolio(IdentityId::from(
                1u128,
            ))),
        };
        let rk3 = SecondaryKey::new(Signatory::Account(key.clone()), rk3_permissions.clone());
        assert_ne!(rk1, rk3);

        let mut rk4 = SecondaryKey::from_account_id(key);
        rk4.permissions = rk3_permissions;
        assert_eq!(rk3, rk4);

        let si1 = SecondaryKey::from(IdentityId::from(1u128));
        let si2 = SecondaryKey::from(IdentityId::from(1u128));
        assert_eq!(si1, si2);

        let si3 = SecondaryKey::from(IdentityId::from(2u128));
        assert_ne!(si1, si3);

        assert_ne!(si1, rk1);
    }

    #[test]
    fn has_permission_test() {
        let key = Public::from_raw([b'A'; 32]);
        let ticker1 = Ticker::try_from(&[1][..]).unwrap();
        let ticker2 = Ticker::try_from(&[2][..]).unwrap();
        let portfolio1 = PortfolioId::user_portfolio(IdentityId::default(), 1.into());
        let portfolio2 = PortfolioId::user_portfolio(IdentityId::default(), 2.into());
        let permissions = Permissions {
            asset: SubsetRestriction::elem(ticker1),
            extrinsic: SubsetRestriction::Whole,
            portfolio: SubsetRestriction::elem(portfolio1),
        };
        let free_key = SecondaryKey::new(Signatory::Account(key.clone()), Permissions::default());
        let restricted_key = SecondaryKey::new(Signatory::Account(key), permissions.clone());
        assert!(free_key.has_asset_permission(ticker2));
        assert!(free_key
            .has_extrinsic_permission(&b"pallet".as_ref().into(), &b"function".as_ref().into()));
        assert!(free_key.has_portfolio_permission(vec![portfolio1]));
        assert!(restricted_key.has_asset_permission(ticker1));
        assert!(!restricted_key.has_asset_permission(ticker2));
        assert!(restricted_key
            .has_extrinsic_permission(&b"pallet".as_ref().into(), &b"function".as_ref().into()));
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
