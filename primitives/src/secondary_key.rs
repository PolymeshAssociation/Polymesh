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

use crate::{
    self as polymesh_primitives, DispatchableName, IdentityId, PalletName, PortfolioId,
    SubsetRestriction, Ticker,
};
use codec::{Decode, Encode};
use polymesh_primitives_derive::Migrate;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    cmp::{Ord, Ordering, PartialOrd},
    iter,
    prelude::Vec,
};

/// Asset permissions.
pub type AssetPermissions = SubsetRestriction<Ticker>;

/// A permission to call
///
/// - specific functions, using `SubsetRestriction(Some(_))`, or
///
/// - all functions, using `SubsetRestriction(None)`
///
/// within a given pallet `pallet_name`.
#[derive(Decode, Encode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PalletPermissions {
    /// The name of a pallet.
    pub pallet_name: PalletName,
    /// A subset of function names within the pallet.
    pub dispatchable_names: SubsetRestriction<DispatchableName>,
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
            dispatchable_names: SubsetRestriction(None),
        }
    }
}

/// Extrinsic permissions.
pub type ExtrinsicPermissions = SubsetRestriction<PalletPermissions>;

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
            extrinsic: SubsetRestriction(Some(pallet_permissions.into_iter().collect())),
            portfolio: SubsetRestriction::empty(),
        }
    }

    /// Adds extra extrinsic permissions to `self` for just one pallet. The result is stored in
    /// `self`.
    pub fn add_pallet_permissions(&mut self, pallet_permissions: PalletPermissions) {
        self.extrinsic = self.extrinsic.union(&SubsetRestriction(Some(
            iter::once(pallet_permissions).collect(),
        )));
    }
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

/// A secondary key is a signatory with defined permissions.
#[derive(Encode, Decode, Default, Clone, Eq, Debug, Migrate)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct SecondaryKey<AccountId: Encode + Decode> {
    /// The account or identity that is the signatory of this key.
    pub signer: Signatory<AccountId>,
    /// The access permissions of the signing key.
    #[migrate_from(Vec<runtime_upgrade::Permission>)]
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
            // Full permissions.
            permissions: Permissions::empty(),
        }
    }

    /// Checks if the given key has permission to access the given asset.
    pub fn has_asset_permission(&self, asset: Ticker) -> bool {
        self.permissions.asset.ge(&SubsetRestriction::elem(asset))
    }

    /// Checks if the given key has permission to call the given extrinsic.
    pub fn has_extrinsic_permission(
        &self,
        pallet_name: &PalletName,
        dispatchable_name: &DispatchableName,
    ) -> bool {
        match &self.permissions.extrinsic.0 {
            None => true,
            Some(pallet_perms) => pallet_perms
                .iter()
                .find(|perm| {
                    if &perm.pallet_name != pallet_name {
                        return false;
                    }
                    match &perm.dispatchable_names.0 {
                        None => true,
                        Some(funcs) => funcs.contains(dispatchable_name),
                    }
                })
                .is_some(),
        }
    }

    /// Checks if the given key has permission to access all given portfolios.
    pub fn has_portfolio_permission(&self, it: impl IntoIterator<Item = PortfolioId>) -> bool {
        self.permissions.portfolio.ge(&SubsetRestriction::elems(it))
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

impl<AccountId> PartialEq for SecondaryKey<AccountId>
where
    AccountId: Encode + Decode + PartialEq,
{
    fn eq(&self, other: &Self) -> bool {
        self.signer == other.signer && self.permissions == other.permissions
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

impl<AccountId> PartialOrd for SecondaryKey<AccountId>
where
    AccountId: Encode + Decode + Ord,
{
    /// Any key is less than any Identity.
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.signer.partial_cmp(&other.signer)
    }
}

impl<AccountId> Ord for SecondaryKey<AccountId>
where
    AccountId: Encode + Decode + Ord,
{
    fn cmp(&self, other: &Self) -> Ordering {
        self.signer.cmp(&other.signer)
    }
}

/// Runtime upgrade definitions.
#[allow(missing_docs)]
pub mod runtime_upgrade {
    use crate::migrate::{Empty, Migrate};
    use codec::Decode;
    use sp_std::vec::Vec;

    /// Old permission type for runtime upgrade purposes.
    #[derive(Decode, PartialEq)]
    pub enum Permission {
        Full,
        Admin,
        Operator,
        SpendFunds,
        Custom(u8),
    }

    impl Migrate for Vec<Permission> {
        type Into = super::Permissions;
        type Context = Empty;

        fn migrate(self, _: Self::Context) -> Option<Self::Into> {
            Some(if self.contains(&Permission::Full) {
                super::Permissions::default()
            } else {
                super::Permissions::empty()
            })
        }
    }
}

/// Vectorized redefinitions of runtime types for the sake of Polkadot.JS.
pub mod api {
    use crate::{DispatchableName, PalletName, PortfolioId, Signatory, SubsetRestriction, Ticker};
    use codec::{Decode, Encode};
    #[cfg(feature = "std")]
    use sp_runtime::{Deserialize, Serialize};
    use sp_std::vec::Vec;

    /// Asset permissions.
    #[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct AssetPermissions(pub Option<Vec<Ticker>>);

    /// A permission to call functions within a given pallet.
    #[derive(Decode, Encode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct PalletPermissions {
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
    pub struct ExtrinsicPermissions(pub Option<Vec<PalletPermissions>>);

    /// Portfolio permissions.
    #[derive(Encode, Decode, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct PortfolioPermissions(pub Option<Vec<PortfolioId>>);

    /// Signing key permissions.
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
                asset: AssetPermissions(Some(Vec::new())),
                extrinsic: ExtrinsicPermissions(Some(Vec::new())),
                portfolio: PortfolioPermissions(Some(Vec::new())),
            }
        }
    }

    /// A secondary key is a signatory with defined permissions.
    #[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
    #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
    pub struct SecondaryKey<AccountId> {
        /// The account or identity that is the signatory of this key.
        pub signer: Signatory<AccountId>,
        /// The access permissions of the signing key.
        pub permissions: Permissions,
    }

    impl From<super::AssetPermissions> for AssetPermissions {
        fn from(p: super::AssetPermissions) -> AssetPermissions {
            AssetPermissions(p.0.map(|elems| elems.into_iter().collect()))
        }
    }

    impl From<super::PalletPermissions> for PalletPermissions {
        fn from(p: super::PalletPermissions) -> PalletPermissions {
            PalletPermissions {
                pallet_name: p.pallet_name,
                total: p.dispatchable_names.0.is_none(),
                dispatchable_names: if let Some(elems) = p.dispatchable_names.0 {
                    elems.into_iter().collect()
                } else {
                    Default::default()
                },
            }
        }
    }

    impl From<super::ExtrinsicPermissions> for ExtrinsicPermissions {
        fn from(p: super::ExtrinsicPermissions) -> ExtrinsicPermissions {
            ExtrinsicPermissions(p.0.map(|elems| elems.into_iter().map(|e| e.into()).collect()))
        }
    }

    impl From<super::PortfolioPermissions> for PortfolioPermissions {
        fn from(p: super::PortfolioPermissions) -> PortfolioPermissions {
            PortfolioPermissions(p.0.map(|elems| elems.into_iter().collect()))
        }
    }

    impl From<super::Permissions> for Permissions {
        fn from(p: super::Permissions) -> Permissions {
            Permissions {
                asset: p.asset.into(),
                extrinsic: p.extrinsic.into(),
                portfolio: p.portfolio.into(),
            }
        }
    }

    impl<AccountId> From<super::SecondaryKey<AccountId>> for SecondaryKey<AccountId>
    where
        AccountId: Encode + Decode,
    {
        fn from(k: super::SecondaryKey<AccountId>) -> SecondaryKey<AccountId> {
            SecondaryKey {
                signer: k.signer,
                permissions: k.permissions.into(),
            }
        }
    }

    impl From<AssetPermissions> for super::AssetPermissions {
        fn from(p: AssetPermissions) -> super::AssetPermissions {
            SubsetRestriction(p.0.map(|elems| elems.into_iter().collect()))
        }
    }

    impl From<PalletPermissions> for super::PalletPermissions {
        fn from(p: PalletPermissions) -> super::PalletPermissions {
            super::PalletPermissions {
                pallet_name: p.pallet_name,
                dispatchable_names: SubsetRestriction(if !p.total {
                    Some(p.dispatchable_names.into_iter().collect())
                } else {
                    Default::default()
                }),
            }
        }
    }

    impl From<ExtrinsicPermissions> for super::ExtrinsicPermissions {
        fn from(p: ExtrinsicPermissions) -> super::ExtrinsicPermissions {
            SubsetRestriction(p.0.map(|elems| elems.into_iter().map(|e| e.into()).collect()))
        }
    }

    impl From<PortfolioPermissions> for super::PortfolioPermissions {
        fn from(p: PortfolioPermissions) -> super::PortfolioPermissions {
            SubsetRestriction(p.0.map(|elems| elems.into_iter().collect()))
        }
    }

    impl From<Permissions> for super::Permissions {
        fn from(p: Permissions) -> super::Permissions {
            super::Permissions {
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
                permissions: k.permissions.into(),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Permissions, SecondaryKey, Signatory, SubsetRestriction};
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
            extrinsic: SubsetRestriction(None),
            portfolio: SubsetRestriction::elem(1.into()),
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
        let portfolio1 = PortfolioId::user_portfolio(IdentityId::default(), 1);
        let portfolio2 = PortfolioId::user_portfolio(IdentityId::default(), 2);
        let permissions = Permissions {
            asset: SubsetRestriction::elem(ticker1),
            extrinsic: SubsetRestriction(None),
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
