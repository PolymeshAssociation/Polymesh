use crate::Ticker;
use codec::{Decode, Encode};
use rstd::prelude::Vec;

/// Authorization data for two step prcoesses.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum LinkData {
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

    // Extra data to allow iterating over the Links.
    /// Link number of the next Link.
    /// Link number starts with 1.
    pub next_link: u64,
    /// Link number of the previous Link.
    /// Link number starts with 1.
    pub previous_link: u64,
}
