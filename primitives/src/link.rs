use crate::Document;
use crate::Ticker;
use codec::{Decode, Encode};

/// Authorization data for two step prcoesses.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum LinkData {
    /// Represents a document (name, URI, content_hash)
    DocumentOwned(Document),
    /// Represents a ticker ownership
    TickerOwned(Ticker),
    /// Represents a token ownership
    TokenOwned(Ticker),
    /// No linked data.
    NoData,
}

impl Default for LinkData {
    fn default() -> Self {
        LinkData::NoData
    }
}

/// Link struct. Connects an Identity to some data.
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Link<U> {
    /// Enum that contains the link data
    pub link_data: LinkData,

    /// time when this Link expires. optional.
    pub expiry: Option<U>,

    /// Link id of this link
    pub link_id: u64,
}
