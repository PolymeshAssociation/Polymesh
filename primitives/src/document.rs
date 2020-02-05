//! Document type

use codec::{Decode, Encode};
use sp_std::{convert::TryFrom, default::Default, prelude::Vec};

/// Represents a document associated with an asset
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Document {
    /// Document name
    pub name: Vec<u8>,
    /// Document URI
    pub uri: Vec<u8>,
    /// Document hash
    pub hash: Vec<u8>,
}
