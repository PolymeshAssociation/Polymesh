use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::Vec;

use crate::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use crate::{impl_checked_inc, Ticker};

/// Controls the next available id for an NFT collection.
#[derive(Clone, Debug, Decode, Default, Eq, Encode, PartialEq, TypeInfo)]
pub struct NFTCollectionId(u64);
impl_checked_inc!(NFTCollectionId);

/// Controls the next available id for an NFT within a collection.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct NFTId(u64);
impl_checked_inc!(NFTId);

/// Defines an NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollection {
    id: NFTCollectionId,
    ticker: Ticker,
}

impl NFTCollection {
    /// Creates a new `NFTCollection`.
    pub fn new(id: NFTCollectionId, ticker: Ticker) -> Self {
        Self { id, ticker }
    }
}

/// The metadata keys for the NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollectionKeys(Vec<AssetMetadataKey>);

impl NFTCollectionKeys {
    /// Returns the number of metadata keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of all metadata keys.
    pub fn metadata_keys(&self) -> &[AssetMetadataKey] {
        &self.0
    }

    /// Returns true if the given key exists.
    pub fn contains(&self, key: &AssetMetadataKey) -> bool {
        self.0.contains(key)
    }
}

/// Defines a metadata attribute which is a composed of a key and a value
#[derive(Clone, Debug, Decode, Encode, PartialEq, TypeInfo)]
pub struct NFTMetadataAttribute {
    /// The mmetadata key
    pub key: AssetMetadataKey,
    /// The mmetadata value
    pub value: AssetMetadataValue,
}

/// Defines an instance of an NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFT {
    id: NFTId,
    collection_id: NFTCollectionId,
}
