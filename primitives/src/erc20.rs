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

#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
use codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;

use polymesh_primitives_derive::StringStrongTyped;

/// Maximum length for ERC-20 token name.
pub const MAX_NAME_LEN: usize = 100;

/// Maximum length for ERC-20 token symbol.
pub const MAX_SYMBOL_LEN: usize = 20;

/// Maximum decimals for ERC-20 token.
pub const MAX_DECIMALS: u8 = 8;

/// ERC-20 token name.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, StringStrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Name(pub String);

impl Name {
    /// Generate a test asset name.
    pub fn test_asset(asset_idx: u32) -> Self {
        Name(format!("Test Asset {}", asset_idx))
    }
}

/// ERC-20 token symbol.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, StringStrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Symbol(pub String);

impl Symbol {
    /// Generate a test asset symbol.
    pub fn test_asset(asset_idx: u32) -> Self {
        Symbol(format!("TST{}", asset_idx))
    }
}
