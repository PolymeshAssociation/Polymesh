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

use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
use sp_std::prelude::Vec;

/// Smart Extension types
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum SmartExtensionType {
    TransferManager,
    Offerings,
    Custom(Vec<u8>),
}

impl Default for SmartExtensionType {
    fn default() -> Self {
        SmartExtensionType::Custom(b"undefined".to_vec())
    }
}

/// A wrapper for a smart extension name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
pub struct SmartExtensionName(pub Vec<u8>);

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
/// U type refers to the AccountId which will act
/// as the address of the smart extension
pub struct SmartExtension<U> {
    /// Type of the extension
    pub extension_type: SmartExtensionType,
    /// Name of extension
    pub extension_name: SmartExtensionName,
    /// AccountId of the smart extension
    pub extension_id: U,
    /// Status of the smart extension
    pub is_archive: bool,
}
