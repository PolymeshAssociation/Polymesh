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
use sp_std::prelude::Vec;

/// A wrapper for a document name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        DocumentName(v)
    }
}

/// A wrapper for a document URI.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentUri(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentUri {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        DocumentUri(v)
    }
}

/// A wrapper for a document hash.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentHash(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentHash {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        DocumentHash(v)
    }
}

/// Represents a document associated with an asset
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Document {
    /// Document name
    pub name: DocumentName,
    /// Document URI
    pub uri: DocumentUri,
    /// Document hash
    pub content_hash: DocumentHash,
}
