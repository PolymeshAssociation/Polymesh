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

use crate::impl_checked_inc;
use crate::Url;
use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::{PortableRegistry, TypeInfo};
use sp_std::prelude::Vec;

/// Asset Metadata Name.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct AssetMetadataName(pub Vec<u8>);

/// Asset Metadata Global Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct AssetMetadataGlobalKey(pub u64);
impl_checked_inc!(AssetMetadataGlobalKey);

/// Asset Metadata Local Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq)]
pub struct AssetMetadataLocalKey(pub u64);
impl_checked_inc!(AssetMetadataLocalKey);

/// Asset Metadata Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq)]
pub enum AssetMetadataKey {
    /// Global Metadata Key.
    Global(u64),
    /// Local Metadata Key.
    Local(u64),
}

/// Asset Metadata Value.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataValue(pub Vec<u8>);

/// Asset Metadata Value details.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataValueDetail<Moment> {
    /// Optional expiry date for the value.
    pub expiry: Option<Moment>,
    /// Locked status of the metadata value.
    pub locked_status: AssetMetadataLockStatus<Moment>,
}

impl<Moment: PartialOrd> AssetMetadataValueDetail<Moment> {
    /// Check if the metadata value has expired.
    pub fn is_expired(&self, now: Moment) -> bool {
        match &self.expiry {
            Some(e) => now > *e,
            None => false,
        }
    }

    /// Check if the metadata value is locked.
    pub fn is_locked(&self, now: Moment) -> bool {
        self.locked_status.is_locked(now)
    }
}

/// Asset Metadata Lock Status
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum AssetMetadataLockStatus<Moment> {
    /// Can be changed by asset issuer.
    Unlocked,
    /// Cannot be changed.
    Locked,
    /// Cannot be changed until `Moment`.
    LockedUntil(Moment),
}

impl<Moment: PartialOrd> AssetMetadataLockStatus<Moment> {
    /// Check if the lock status is locked.
    pub fn is_locked(&self, now: Moment) -> bool {
        match self {
            Self::Unlocked => false,
            Self::Locked => true,
            Self::LockedUntil(until) => now > *until,
        }
    }
}

impl<Moment> Default for AssetMetadataLockStatus<Moment> {
    fn default() -> Self {
        Self::Unlocked
    }
}

/// Asset Metadata description.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataDescription(pub Vec<u8>);

/// Asset Metadata Specs.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataSpec {
    /// Off-chain specs or documentation.
    pub url: Option<Url>,
    /// Description of metadata type.
    pub description: Option<AssetMetadataDescription>,
    /// Optional SCALE encoded `AssetMetadataScaleType`.
    pub scale_type: Option<Vec<u8>>,
}

impl AssetMetadataSpec {
    /// Returns the complexity of the metadata specs.
    pub fn complexity(&self) -> usize {
        let mut complexity = 0usize;
        if self.url.is_some() {
            complexity += 1;
        }
        if self.description.is_some() {
            complexity += 1;
        }
        if self.scale_type.is_some() {
            complexity += 1;
        }
        complexity
    }
}

/// Asset Metadata versioned SCALE type definition.
///
/// This allows for future changes to how SCALE type definitions are encoded.
#[derive(Encode, Decode)]
#[derive(Clone, PartialEq, Eq)]
pub enum AssetMetadataScaleType {
    /// Version 1 SCALE encoded type definition.
    V1(AssetMetadataScaleTypeV1),
}

/// Asset Metadata SCALE V1 type definition.
#[derive(Encode, Decode)]
#[derive(Clone, PartialEq, Eq)]
pub struct AssetMetadataScaleTypeV1 {
    /// A registry of type definitions.
    pub types: PortableRegistry,
    /// The top-level type id.
    pub ty: u32,
}
