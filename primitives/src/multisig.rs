// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2024 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Count of approvals and rejections of a multisig proposal.
#[derive(Clone, Debug, Default, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub struct ProposalVoteCount {
    /// Number of yes votes
    pub approvals: u64,
    /// Number of no votes
    pub rejections: u64,
}

/// State of a multisig proposal.
#[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
pub enum ProposalState<Moment> {
    /// Proposal is active.
    Active {
        /// Optional time limit.
        until: Option<Moment>,
    },
    /// Proposal was accepted and executed successfully
    ExecutionSuccessful,
    /// Proposal was accepted and execution was tried but it failed
    ExecutionFailed,
    /// Proposal was rejected
    Rejected,
}

impl<Moment> ProposalState<Moment>
where
    Moment: PartialOrd<Moment>,
{
    /// Create new proposal state with optional time limit.
    pub fn new(until: Option<Moment>) -> Self {
        Self::Active { until }
    }

    /// Return `true` if the proposal state is active.
    pub fn is_active<T: FnOnce() -> Moment>(&self, get_timestamp: T) -> bool {
        match self {
            Self::Active { until: None } => true,
            Self::Active { until: Some(until) } => {
                let current = get_timestamp();
                // The proposal is still active, when the current time is
                // less then `until`.
                *until > current
            }
            _ => false,
        }
    }
}
