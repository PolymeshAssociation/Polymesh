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
    AssetOwned(Ticker),
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
