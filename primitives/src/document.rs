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

//! Document type
use codec::{Decode, Encode};
use polymesh_primitives_derive::VecU8StrongTyped;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::prelude::Vec;

/// A wrapper for a document name.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DocumentName(pub Vec<u8>);

/// A wrapper for a document URI.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DocumentUri(pub Vec<u8>);

/// A wrapper for a document hash.
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord, VecU8StrongTyped,
)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DocumentHash(pub Vec<u8>);

/// Represents a document associated with an asset
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Document {
    /// Document URI
    pub uri: DocumentUri,
    /// Document hash
    pub content_hash: DocumentHash,
}
