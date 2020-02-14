//! Document type

use codec::{Decode, Encode};
use sp_std::prelude::Vec;

/// A wrapper for a document name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentName {
    fn from(s: T) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(s.as_ref());
        DocumentName(v)
    }
}

/// A wrapper for a document URI.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentUri(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentUri {
    fn from(s: T) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(s.as_ref());
        DocumentUri(v)
    }
}

/// A wrapper for a document hash.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DocumentHash(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DocumentHash {
    fn from(s: T) -> Self {
        let mut v = Vec::new();
        v.extend_from_slice(s.as_ref());
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
    pub hash: DocumentHash,
}
