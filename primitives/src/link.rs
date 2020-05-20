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
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Authorization data for two step processes.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
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
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Link<U> {
    /// Enum that contains the link data
    pub link_data: LinkData,

    /// time when this Link expires. optional.
    pub expiry: Option<U>,

    /// Link id of this link
    pub link_id: u64,
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::Moment;
    use frame_support::assert_err;
    use std::convert::TryFrom;

    #[test]
    fn serialize_and_deserialize_link() {
        let link_to_serialize = Link::<Moment> {
            link_data: LinkData::DocumentOwned(Document {
                name: b"abc".into(),
                uri: b"abc.com".into(),
                content_hash: b"hash".into(),
            }),
            expiry: None,
            link_id: 5,
        };
        let serialize_link = serde_json::to_string(&link_to_serialize).unwrap();
        let serialize_data = "{\"link_data\":{\"DocumentOwned\":{\"name\":[97,98,99],\"uri\":[97,98,99,46,99,111,109],\"content_hash\":[104,97,115,104]}},\"expiry\":null,\"link_id\":5}";
        assert_eq!(serialize_link, serialize_data);
        println!("Serialize link: {:?}", serialize_link);
        let deserialize_data = serde_json::from_str::<Link<Moment>>(&serialize_link).unwrap();
        println!("Print the deserialize data {:?}", deserialize_data);
        assert_eq!(link_to_serialize, deserialize_data);
    }
}
