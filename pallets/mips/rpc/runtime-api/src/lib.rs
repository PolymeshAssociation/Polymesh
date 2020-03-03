//! Runtime API definition for mips module.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Codec, Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::traits::{SaturatedConversion, UniqueSaturatedInto};
use sp_std::{prelude::*, vec::Vec};

/// A result of execution of get_votes.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum VoteCount<Balance> {
    /// Proposal was found and has the following votes.
    Success {
        /// Stake for
        ayes: Balance,
        /// Stake against
        nays: Balance,
    },
    /// Proposal was not for given index.
    ProposalNotFound,
}

/// A capped version of `VoteCount`.
///
/// The `Balance` is capped (or expanded) to `u64` to avoid serde issues with `u128`.
#[derive(Eq, PartialEq, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
pub enum CappedVoteCount {
    /// Proposal was found and has the following votes.
    Success {
        /// Stake for
        ayes: u64,
        /// Stake against
        nays: u64,
    },
    /// Proposal was not for given index.
    ProposalNotFound,
}

impl CappedVoteCount {
    /// Create a new `CappedVoteCount` from `VoteCount`.
    pub fn new<Balance: UniqueSaturatedInto<u64>>(count: VoteCount<Balance>) -> Self {
        match count {
            VoteCount::Success { ayes, nays } => CappedVoteCount::Success {
                ayes: ayes.saturated_into(),
                nays: nays.saturated_into(),
            },
            VoteCount::ProposalNotFound => CappedVoteCount::ProposalNotFound,
        }
    }
}

sp_api::decl_runtime_apis! {
    /// The API to interact with Mips governance.
    pub trait MipsApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec,
    {
        /// Retrieve votes for a proposal for a given `mips_index`.
        fn get_votes(mips_index: u32) -> VoteCount<Balance>;

        /// Retrieve proposals started by `address`.
        fn proposed_by(address: AccountId) -> Vec<u32>;

        /// Retrieve proposals `address` voted on.
        fn voted_on(address: AccountId) -> Vec<u32>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_mips_votes() {
        let votes = MipsVotes::Success {
            ayes: 3141u64,
            nays: 5926u64,
        };

        assert_eq!(
            serde_json::to_string(&votes).unwrap(),
            r#"{"Success":{"ayes":3141,"nays":5926}}"#,
        );

        // should not panic
        serde_json::to_value(&votes).unwrap();
    }
}
