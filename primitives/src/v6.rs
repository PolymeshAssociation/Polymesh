#![allow(missing_docs)]

use crate::asset::AssetID;
use crate::ticker::Ticker;
use crate::{PortfolioPermissions, SubsetRestriction};
#[cfg(not(feature = "std"))]
use alloc::string::{String, ToString};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::{
    collections::{btree_map::BTreeMap, btree_set::BTreeSet},
    prelude::Vec,
};

#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PalletName(pub Vec<u8>);

impl From<PalletName> for crate::PalletName {
    fn from(old: PalletName) -> Self {
        Self(String::from_utf8_lossy(&old.0).to_string())
    }
}

#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct DispatchableName(pub Vec<u8>);

impl From<DispatchableName> for crate::ExtrinsicName {
    fn from(old: DispatchableName) -> Self {
        Self(String::from_utf8_lossy(&old.0).to_string())
    }
}

pub type DispatchableNames = SubsetRestriction<DispatchableName>;

impl From<SubsetRestriction<DispatchableName>> for SubsetRestriction<crate::ExtrinsicName> {
    fn from(old: SubsetRestriction<DispatchableName>) -> Self {
        match old {
            SubsetRestriction::Whole => SubsetRestriction::Whole,
            SubsetRestriction::These(names) => {
                SubsetRestriction::These(names.into_iter().map(|n| n.into()).collect())
            }
            SubsetRestriction::Except(names) => {
                SubsetRestriction::Except(names.into_iter().map(|n| n.into()).collect())
            }
        }
    }
}

#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct PalletPermissions {
    pub pallet_name: PalletName,
    pub dispatchable_names: DispatchableNames,
}

pub type AssetPermissions = SubsetRestriction<Ticker>;

impl From<SubsetRestriction<Ticker>> for SubsetRestriction<AssetID> {
    fn from(old: SubsetRestriction<Ticker>) -> Self {
        match old {
            SubsetRestriction::Whole => SubsetRestriction::Whole,
            SubsetRestriction::These(tickers) => {
                SubsetRestriction::These(tickers.into_iter().map(|t| t.into()).collect())
            }
            SubsetRestriction::Except(tickers) => {
                SubsetRestriction::Except(tickers.into_iter().map(|t| t.into()).collect())
            }
        }
    }
}

pub type ExtrinsicPermissions = SubsetRestriction<PalletPermissions>;

fn from_old_pallets(
    old: BTreeSet<PalletPermissions>,
) -> BTreeMap<crate::PalletName, crate::PalletPermissions> {
    old.into_iter()
        .map(|pallet| {
            (
                pallet.pallet_name.into(),
                crate::PalletPermissions {
                    extrinsics: pallet.dispatchable_names.into(),
                },
            )
        })
        .collect()
}

impl From<SubsetRestriction<PalletPermissions>> for crate::ExtrinsicPermissions {
    fn from(old: SubsetRestriction<PalletPermissions>) -> Self {
        match old {
            SubsetRestriction::Whole => Self::Whole,
            SubsetRestriction::These(pallets) => Self::These(from_old_pallets(pallets)),
            SubsetRestriction::Except(pallets) => Self::Except(from_old_pallets(pallets)),
        }
    }
}

#[derive(Decode, Encode, TypeInfo)]
pub struct Permissions {
    pub asset: AssetPermissions,
    pub extrinsic: ExtrinsicPermissions,
    pub portfolio: PortfolioPermissions,
}

impl From<Permissions> for crate::Permissions {
    fn from(old: Permissions) -> Self {
        Self {
            asset: old.asset.into(),
            extrinsic: old.extrinsic.into(),
            portfolio: old.portfolio,
        }
    }
}
