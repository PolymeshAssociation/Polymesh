use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::prelude::Vec;

use crate::asset_metadata::AssetMetadataKey;
use crate::{impl_checked_inc, Ticker};

/// Controls the next available id for an NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollectionId(u64);
impl_checked_inc!(NFTCollectionId);

/// Controls the next available id for an NFT within a collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
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

/// Defines an instance of an NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFT {
    id: NFTId,
    collection_id: NFTCollectionId,
}

/// The metadata keys for the NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollectionKeys(pub(crate) Vec<AssetMetadataKey>);

impl NFTCollectionKeys {
    /// Returns the number of metadata keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Returns a slice of all metadata keys.
    pub fn metadata_keys(&self) -> &[AssetMetadataKey] {
        &self.0
    }
}
