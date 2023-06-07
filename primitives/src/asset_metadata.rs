// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
use codec::{Decode, DecodeAll, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::{PortableRegistry, TypeInfo};
use sp_std::prelude::Vec;

/// Asset Metadata Name.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq)]
pub struct AssetMetadataName(pub Vec<u8>);

/// Asset Metadata Global Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssetMetadataGlobalKey(pub u64);
impl_checked_inc!(AssetMetadataGlobalKey);

/// Asset Metadata Local Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Default, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub struct AssetMetadataLocalKey(pub u64);
impl_checked_inc!(AssetMetadataLocalKey);

/// Asset Metadata Key.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, Ord, PartialOrd)]
pub enum AssetMetadataKey {
    /// Global Metadata Key.
    Global(AssetMetadataGlobalKey),
    /// Local Metadata Key.
    Local(AssetMetadataLocalKey),
}

impl From<AssetMetadataLocalKey> for AssetMetadataKey {
    fn from(key: AssetMetadataLocalKey) -> Self {
        Self::Local(key)
    }
}

impl From<AssetMetadataGlobalKey> for AssetMetadataKey {
    fn from(key: AssetMetadataGlobalKey) -> Self {
        Self::Global(key)
    }
}

/// Asset Metadata Value.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataValue(pub Vec<u8>);

/// Asset Metadata Value details.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetMetadataValueDetail<Moment> {
    /// Optional expire date for the value.
    pub expire: Option<Moment>,
    /// Lock status of the metadata value.
    pub lock_status: AssetMetadataLockStatus<Moment>,
}

impl<Moment: PartialOrd> AssetMetadataValueDetail<Moment> {
    /// Check if the metadata value has expired.
    pub fn is_expired(&self, now: Moment) -> bool {
        match &self.expire {
            Some(e) => now > *e,
            None => false,
        }
    }

    /// Check if the metadata value is locked.
    pub fn is_locked(&self, now: Moment) -> bool {
        self.lock_status.is_locked(now)
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
            Self::LockedUntil(until) => now < *until,
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
    /// Optional SCALE encoded `AssetMetadataTypeDef`.
    pub type_def: Option<Vec<u8>>,
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
        if self.type_def.is_some() {
            complexity += 1;
        }
        complexity
    }

    /// Set the metadata type definition from an `AssetMetadataTypeDef`.
    ///
    /// This will encode the `AssetMetadataTypeDef` to a byte array.
    pub fn set_type_def(&mut self, type_def: AssetMetadataTypeDef) {
        self.type_def = Some(type_def.encode());
    }

    /// Decode the metadata type definition from `self.type_def` and return it as an `AssetMetadataTypeDef`.
    pub fn decode_type_def(&self) -> Result<Option<AssetMetadataTypeDef>, codec::Error> {
        self.type_def
            .as_ref()
            .map(|d| AssetMetadataTypeDef::decode_all(&mut &d[..]))
            .transpose()
    }

    /// Return the type definition length.
    pub fn type_def_len(&self) -> usize {
        self.type_def.as_ref().map(|d| d.len()).unwrap_or_default()
    }
}

/// Asset Metadata type definition.
#[derive(Encode, Decode)]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct AssetMetadataTypeDef {
    /// A registry of type definitions.
    pub types: PortableRegistry,
    /// The top-level type id.
    #[codec(compact)]
    pub ty: u32,
}

impl AssetMetadataTypeDef {
    /// Create metadata type definition from a type implmenting `scale_info::TypeInfo`.
    pub fn new_from_type<T: scale_info::StaticTypeInfo>() -> Self {
        let mut reg = scale_info::Registry::new();
        let ty = reg.register_type(&scale_info::meta_type::<T>()).id;
        Self {
            types: reg.into(),
            ty,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::Moment;

    /// Asset Metadata Test Type.
    ///
    /// Wrap a few `TypeInfo` types for testing.
    #[derive(Encode, Decode, TypeInfo)]
    #[derive(Clone, Debug, Default, PartialEq, Eq)]
    struct MetadataTestType1 {
        spec: AssetMetadataSpec,
        details: AssetMetadataValueDetail<Moment>,
    }

    #[test]
    fn encode_decode_metadata_specs_test() {
        let type_def = AssetMetadataTypeDef::new_from_type::<MetadataTestType1>();
        let mut spec = AssetMetadataSpec::default();
        // Encode type definition.
        spec.set_type_def(type_def.clone());
        println!("Type definition length: {}", spec.type_def_len());
        // Decode type definition.
        let type_def2 = spec
            .decode_type_def()
            .expect("Failed to decode type def.")
            .expect("Missing type def.");
        assert_eq!(type_def, type_def2);
    }
}
