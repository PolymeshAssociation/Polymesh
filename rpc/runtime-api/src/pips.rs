// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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
use pallet_pips::{PipId, VoteCount};
use sp_std::vec::Vec;

/// This module contains some types which require transformations to avoid serde issues with
/// `u128` type.
/// For instance, `Balance` is capped (or expanded) to `u64` in `VoteCount`.
pub mod capped {
    use pallet_pips::{Vote as CoreVote, VoteCount as CoreVoteCount};

    use codec::{Decode, Encode};
    use sp_runtime::traits::SaturatedConversion;

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

    impl From<CoreVoteCount> for VoteCount {
        fn from(vote_count: CoreVoteCount) -> Self {
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
        Yes(u64),
        No(u64),
    }

    impl From<CoreVote> for Vote {
        fn from(core_vote: CoreVote) -> Self {
            match core_vote {
                CoreVote(true, amount) => Vote::Yes(amount.saturated_into()),
                CoreVote(false, amount) => Vote::No(amount.saturated_into()),
            }
        }
    }
}

sp_api::decl_runtime_apis! {
    /// The API to interact with Pips governance.
    pub trait PipsApi<AccountId>
    where
        AccountId: Codec,
    {
        /// Retrieve votes for a proposal for a given `id`.
        fn get_votes(id: PipId) -> VoteCount;

        /// Retrieve proposals started by `address`.
        fn proposed_by(address: AccountId) -> Vec<PipId>;

        /// Retrieve proposals `address` voted on.
        fn voted_on(address: AccountId) -> Vec<PipId>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_serialize_pips_votes() {
        let votes = VoteCount::ProposalFound {
            ayes: 3141,
            nays: 5926,
        };

        assert_eq!(
            serde_json::to_string(&votes).unwrap(),
            r#"{"ProposalFound":{"ayes":3141,"nays":5926}}"#,
        );

        // should not panic
        serde_json::to_value(&votes).unwrap();
    }
}
