//! Runtime API definition for Identity module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
pub use polymesh_primitives::{IdentityId, Link, Moment};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_std::{prelude::*, vec::Vec};

pub type Error = Vec<u8>;
pub type CddStatus = Result<IdentityId, Error>;
pub type AssetDidResult = Result<IdentityId, Error>;

/// A result of execution of get_votes.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum DidRecords<AccountKey, SigningItem> {
    /// Id was found and has the following master key and signing keys.
    Success {
        master_key: AccountKey,
        signing_items: Vec<SigningItem>,
    },
    /// Error.
    IdNotFound,
}

#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum LinkType {
    DocumentOwnership,
    TickerOwnership,
    AssetOwnership,
    NoData,
}
#[derive(Encode, Decode, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
pub enum DidStatus {
    Unknown,
    Exists,
    CddVerified,
}

sp_api::decl_runtime_apis! {
    pub trait IdentityApi<IdentityId, Ticker, AccountKey, SigningItem, Signatory, Moment> where
        IdentityId: Codec,
        Ticker: Codec,
        AccountKey: Codec,
        SigningItem: Codec,
        Signatory: Codec,
        Moment: Codec
    {
        /// Returns CDD status of an identity
        fn is_identity_has_valid_cdd(did: IdentityId, buffer_time: Option<u64>) -> CddStatus;

        /// Returns DID of an asset
        fn get_asset_did(ticker: Ticker) -> AssetDidResult;

        /// Retrieve DidRecord for a given `did`.
        fn get_did_records(did: IdentityId) -> DidRecords<AccountKey, SigningItem>;

        /// Retrieve list of a link for a given signatory
        fn get_filtered_links(signatory: Signatory, allow_expired: bool, link_type: Option<LinkType>) -> Vec<Link<Moment>>;

        /// Retrieve the status of the DID
        fn get_did_status(dids: Vec<IdentityId>) -> Vec<DidStatus>;

    }
}
