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

use codec::{Decode, Encode};
use scale_info::TypeInfo;

/// Details of a multisig proposal.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub struct ProposalDetails<T> {
    /// Number of yes votes
    pub approvals: u64,
    /// Number of no votes
    pub rejections: u64,
    /// Status of the proposal
    pub status: ProposalStatus,
    /// Expiry of the proposal
    pub expiry: Option<T>,
    /// Should the proposal be closed after getting inverse of sign required reject votes
    pub auto_close: bool,
}

impl<T: Default> ProposalDetails<T> {
    /// Create a new [`ProposalDetails`] object with the given config.
    pub fn new(expiry: Option<T>, auto_close: bool) -> Self {
        Self {
            approvals: 0,
            rejections: 0,
            status: ProposalStatus::ActiveOrExpired,
            expiry,
            auto_close,
        }
    }
}

/// Status of a multisig proposal.
#[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
pub enum ProposalStatus {
    /// Proposal does not exist
    #[default]
    Invalid,
    /// Proposal has not been closed yet. This means that it's either expired or open for voting.
    ActiveOrExpired,
    /// Proposal was accepted and executed successfully
    ExecutionSuccessful,
    /// Proposal was accepted and execution was tried but it failed
    ExecutionFailed,
    /// Proposal was rejected
    Rejected,
}
