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

//! Document type

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::prelude::Vec;

use polymesh_primitives_derive::VecU8StrongTyped;

use crate::{DocumentHash, Moment};

/// The local, per-ticker, ID of an asset documentation.
#[derive(Copy, Clone, Debug, Default, Eq, Hash, PartialEq, PartialOrd, Ord)]
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Serialize, Deserialize)]
pub struct DocumentId(pub u32);

/// A wrapper for a document name.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
pub struct DocumentName(pub Vec<u8>);

/// A wrapper for a document URI.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
pub struct DocumentUri(pub Vec<u8>);

/// A wrapper for a document's type.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[derive(Serialize, Deserialize)]
pub struct DocumentType(pub Vec<u8>);

/// Represents a document associated with an asset
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[derive(Serialize, Deserialize)]
pub struct Document {
    /// An URI where more details can be discovered.
    /// For example, this might link to an external `.pdf`.
    pub uri: DocumentUri,
    /// Hash of the document.
    pub content_hash: DocumentHash,
    /// The document's name.
    /// Need not be unique among a ticker's documents.
    pub name: DocumentName,
    /// The document's type.
    /// This is free form text with no uniqueness requirements.
    pub doc_type: Option<DocumentType>,
    /// When, legally speaking, the document was filed.
    /// Need not be when added to chain.
    pub filing_date: Option<Moment>,
}
