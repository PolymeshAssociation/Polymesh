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

//! Runtime API definition for pips module.
use codec::Codec;
use pallet_pips::{HistoricalVotingByAddress, HistoricalVotingById, Vote, VoteCount};
use polymesh_primitives::IdentityId;

use sp_std::{prelude::*, vec::Vec};

/// This module contains some types which require transformations to avoid serde issues with
/// `u128` type.
/// For instance, `Balance` is capped (or expanded) to `u64` in `VoteCount`.
pub mod capped {
    use pallet_pips::{Vote as CoreVote, VoteCount as CoreVoteCount};

    use codec::{Decode, Encode};
    use sp_runtime::traits::{SaturatedConversion, UniqueSaturatedInto};

    #[cfg(feature = "std")]
    use serde::{Deserialize, Serialize};

    #[derive(Eq, PartialEq, Encode, Decode)]
    #[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
    #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
    pub enum VoteCount {
        /// Proposal was found and has the following votes.
        ProposalFound {
            /// Stake for
            ayes: u64,
            /// Stake against
            nays: u64,
        },
        /// Proposal was not for given index.
        ProposalNotFound,
    }

    impl<Balance> From<CoreVoteCount<Balance>> for VoteCount
    where
        Balance: UniqueSaturatedInto<u64>,
    {
        fn from(vote_count: CoreVoteCount<Balance>) -> Self {
            match vote_count {
                CoreVoteCount::ProposalFound { ayes, nays } => VoteCount::ProposalFound {
                    ayes: ayes.saturated_into(),
                    nays: nays.saturated_into(),
                },
                CoreVoteCount::ProposalNotFound => VoteCount::ProposalNotFound,
            }
        }
    }

    #[derive(Eq, PartialEq)]
    #[cfg_attr(feature = "std", derive(Debug, Serialize, Deserialize))]
    #[cfg_attr(feature = "std", serde(rename_all = "camelCase"))]
    pub enum Vote {
        None,
        Yes(u64),
        No(u64),
    }

    impl<Balance> From<CoreVote<Balance>> for Vote
    where
        Balance: UniqueSaturatedInto<u64>,
    {
        fn from(core_vote: CoreVote<Balance>) -> Self {
            match core_vote {
                CoreVote::None => Vote::None,
                CoreVote::Yes(amount) => Vote::Yes(amount.saturated_into()),
                CoreVote::No(amount) => Vote::No(amount.saturated_into()),
            }
        }
    }
}

sp_api::decl_runtime_apis! {
    /// The API to interact with Pips governance.
    pub trait PipsApi<AccountId, Balance>
    where
        AccountId: Codec,
        Balance: Codec // + UniqueSaturatedInto<u128>
    {
        /// Retrieve votes for a proposal for a given `pips_index`.
        fn get_votes(pips_index: u32) -> VoteCount<Balance>;

        /// Retrieve proposals started by `address`.
        fn proposed_by(address: AccountId) -> Vec<u32>;

        /// Retrieve proposals `address` voted on.
        fn voted_on(address: AccountId) -> Vec<u32>;

        /// Retrieve referendums voted on information by `address` account.
        fn voting_history_by_address(address: AccountId) -> HistoricalVotingByAddress<Vote<Balance>>;

        /// Retrieve referendums voted on information by `id` identity (and its signing items).
        fn voting_history_by_id(id: IdentityId) -> HistoricalVotingById<Vote<Balance>>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_pips_votes() {
        let votes = VoteCount::ProposalFound {
            ayes: 3141u64,
            nays: 5926u64,
        };

        assert_eq!(
            serde_json::to_string(&votes).unwrap(),
            r#"{"ProposalFound":{"ayes":3141,"nays":5926}}"#,
        );

        // should not panic
        serde_json::to_value(&votes).unwrap();
    }
}
