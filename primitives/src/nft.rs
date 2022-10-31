use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_std::vec::IntoIter;
use sp_std::vec::Vec;

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

    /// Returns a reference to the `NFTCollectionId`.
    pub fn id(&self) -> &NFTCollectionId {
        &self.id
    }

    /// Returns a reference to the `Ticker` associated with the collection.
    pub fn ticker(&self) -> &Ticker {
        &self.ticker
    }
}

/// The metadata keys for the NFT collection.
#[derive(Clone, Debug, Decode, Default, Encode, PartialEq, TypeInfo)]
pub struct NFTCollectionKeys(Vec<AssetMetadataKey>);

impl<'a> NFTCollectionKeys {
    /// Returns an iterator, consuming the value, over `AssetMetadataKey`.
    pub fn into_iter(self) -> IntoIter<AssetMetadataKey> {
        self.0.into_iter()
    }

    /// Returns the number of metadata keys.
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

/// Defines a metadata attribute which is a composed of a key and a value.
#[derive(Clone, Debug, Decode, Encode, PartialEq, TypeInfo)]
pub struct NFTMetadataAttribute {
    /// The metadata key.
    pub key: AssetMetadataKey,
    /// The metadata value.
    pub value: AssetMetadataValue,
}
