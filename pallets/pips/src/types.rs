// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::cmp::Ordering;
use frame_support::traits::schedule::v3::TaskName;
use frame_support::traits::schedule::{Priority, HARD_DEADLINE};
use frame_support::traits::LockIdentifier;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_core::H256;
use sp_io::hashing::blake2_256;
use sp_std::convert::From;
use sp_std::vec::Vec;

use polymesh_primitives::constants::{PIP_EXECUTION, PIP_EXPIRY};
use polymesh_primitives::{impl_checked_inc, Balance, MaybeBlock, Url};
use polymesh_primitives_derive::VecU8StrongTyped;

/// The highest priorities, from `HIGHEST_PRIORITY`(=0) to `HARD_DEADLINE`(=63), enforce the execution of
/// the scheduler task even if there is not enough free space in the block to allocate it.
/// Scheduler also enforces the execution of the first priority task for the corresponding
/// block, and it will re-schedule any pending task into the next block if there is not enough space in
/// the current one.
/// This `MAX_NORMAL_PRIORITY` is the highest priority where:
///     - Scheduler will NOT enforce the execution of the scheduled task, and
///     - The task could be re-scheduled to the next bock.
/// In substrate, normal priorities come from `HARD_DEADLINE + 1`(=64) to `LOWEST_PRIORITY`(=255).
pub const MAX_NORMAL_PRIORITY: Priority = HARD_DEADLINE + 1;

pub(crate) const PIPS_LOCK_ID: LockIdentifier = *b"pips    ";

/// A wrapper for a proposal description.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct PipDescription(pub Vec<u8>);

/// The global and unique identitifer of a Polymesh Improvement Proposal (PIP).
#[derive(Serialize, Deserialize)]
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct PipId(pub u32);
impl_checked_inc!(PipId);

impl PipId {
    /// Converts [`PipId`] into a [`TaskName`] of a PIP scheduled for execution.
    pub fn execution_name(&self) -> TaskName {
        let encoded_task = (PIP_EXECUTION, self.0).encode();
        blake2_256(&encoded_task[..])
    }

    /// Converts [`PipId`] into a [`TaskName`] of a PIP scheduled for expiry.
    pub fn expiry_name(&self) -> TaskName {
        let encoded_task = (PIP_EXPIRY, self.0).encode();
        blake2_256(&encoded_task[..])
    }
}

/// Represents a proposal
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Eq, PartialEq)]
pub struct Pip<Proposal, AccountId> {
    /// The proposal's unique id.
    pub id: PipId,
    /// The proposal being voted on.
    pub proposal: Proposal,
    /// The issuer of `propose`.
    pub proposer: Proposer<AccountId>,
}

/// A result of execution of get_votes.

#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen, Eq, PartialEq)]
pub enum VoteCount {
    /// Proposal was found and has the following votes.
    ProposalFound {
        /// Stake for
        ayes: Balance,
        /// Stake against
        nays: Balance,
    },
    /// Proposal was not for given index.
    ProposalNotFound,
}

/// Either the entire proposal encoded as a byte vector or its hash. The latter represents large
/// proposals.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo)]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum ProposalData {
    /// The hash of the proposal.
    Hash(H256),
    /// The entire proposal.
    Proposal(Vec<u8>),
}

/// The various sorts of committees that can make a PIP.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Committee {
    /// The technical committee.
    Technical,
    /// The upgrade committee tends to propose chain upgrades.
    Upgrade,
}

/// The proposer of a certain PIP.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Proposer<AccountId> {
    /// The proposer is of the community.
    Community(AccountId),
    /// The proposer is a committee.
    Committee(Committee),
}

/// Represents a proposal metadata
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Eq, PartialEq)]
pub struct PipsMetadata<BlockNumber> {
    /// The proposal's unique id.
    pub id: PipId,
    /// The proposal url for proposal discussion.
    pub url: Option<Url>,
    /// The proposal description.
    pub description: Option<PipDescription>,
    /// The block when the PIP was made.
    pub created_at: BlockNumber,
    /// Assuming the runtime has a given `rv: RuntimeVersion` at the point of `Pips::propose`,
    /// then this field contains `rv.transaction_version`.
    ///
    /// Currently, this is only used for off-chain purposes to highlight any differences
    /// in the proposal's transaction version from the current one.
    pub transaction_version: u32,
    /// The point, if any, at which this PIP, if still in a `Pending` state,
    /// is expired, and thus no longer valid.
    ///
    /// This field has no operational on-chain effect and is provided for UI purposes only.
    /// On-chain effects are instead handled via scheduling.
    pub expiry: MaybeBlock<BlockNumber>,
}

/// For keeping track of proposal being voted on.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
pub struct VotingResult {
    /// The current set of voters that approved with their stake.
    pub ayes_count: u32,
    pub ayes_stake: Balance,
    /// The current set of voters that rejected with their stake.
    pub nays_count: u32,
    pub nays_stake: Balance,
}

/// A "vote" or "signal" on a PIP to move it up or down the review queue.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Serialize, Deserialize)]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct Vote(
    /// `true` if there's agreement.
    pub bool,
    /// How strongly do they feel about it?
    pub Balance,
);

/// The state a PIP is in.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum ProposalState {
    /// Initial state. Proposal is open to voting.
    #[default]
    Pending,
    /// Proposal was rejected by the GC.
    Rejected,
    /// Proposal has been approved by the GC and scheduled for execution.
    Scheduled,
    /// Proposal execution was attempted by failed.
    Failed,
    /// Proposal was successfully executed.
    Executed,
    /// Proposal has expired. Only previously `Pending` PIPs may end up here.
    Expired,
}

/// Information about deposit.
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Eq, PartialEq)]
pub struct DepositInfo<AccountId> {
    /// Owner of the deposit.
    pub owner: AccountId,
    /// Amount deposited.
    pub amount: Balance,
}

/// ID of the taken snapshot in a sequence.
#[derive(Serialize, Deserialize)]
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Default, Eq, Ord, PartialEq, PartialOrd)]
pub struct SnapshotId(pub u32);
impl_checked_inc!(SnapshotId);

/// A snapshot's metadata, containing when it was created and who triggered it.
/// The priority queue is stored separately (see `SnapshottedPip`).
#[cfg_attr(feature = "std", derive(Debug))]
#[derive(Decode, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Eq, PartialEq)]
pub struct SnapshotMetadata<BlockNumber, AccountId> {
    /// The block when the snapshot was made.
    pub created_at: BlockNumber,
    /// Who triggered this snapshot? Should refer to someone in the GC.
    pub made_by: AccountId,
    /// Unique ID of this snapshot.
    pub id: SnapshotId,
}

/// A PIP in the snapshot's priority queue for consideration by the GC.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SnapshottedPip {
    /// Identifies the PIP this refers to.
    pub id: PipId,
    /// Weight of the proposal in the snapshot's priority queue.
    /// Higher weights come before lower weights.
    /// The `bool` denotes the sign, where `true` siginfies a positive number.
    pub weight: (bool, Balance),
}

/// Defines sorting order for PIP priority queues, with highest priority *last*.
/// Having higher prio last allows efficient tail popping, so we have a LIFO structure.
pub(crate) fn compare_spip(l: &SnapshottedPip, r: &SnapshottedPip) -> Ordering {
    let (l_dir, l_stake) = l.weight;
    let (r_dir, r_stake) = r.weight;
    l_dir
        .cmp(&r_dir) // Negative has lower prio.
        .then_with(|| match l_dir {
            true => l_stake.cmp(&r_stake), // Higher stake, higher prio...
            // Unless negative stake, in which case lower abs stake, higher prio.
            false => r_stake.cmp(&l_stake),
        })
        // Lower id was made first, so assigned higher prio.
        // This also gives us sorting stability through a total order.
        // Moreover, as `queue` should be in by-id order originally.
        .then(r.id.cmp(&l.id))
}

/// A result to enact for one or many PIPs in the snapshot queue.
// This type is only here due to `enact_snapshot_results`.
#[derive(Decode, DecodeWithMemTracking, Encode, Eq, PartialEq, TypeInfo)]
#[derive(Clone, Copy, Debug)]
pub enum SnapshotResult {
    /// Approve the PIP and move it to the execution queue.
    Approve,
    /// Reject the PIP, removing it from future consideration.
    Reject,
    /// Skip the PIP, bumping the `skipped_count`,
    /// or fail if the threshold for maximum skips is exceeded.
    Skip,
}

#[cfg(test)]
mod test {
    use crate::PipId;

    #[test]
    fn compare_spip_works() {
        let mk = |id, sign, power| super::SnapshottedPip {
            id: PipId(id),
            weight: (sign, power),
        };
        let a = mk(4, true, 50);
        let b = mk(3, true, 50);
        let c = mk(5, true, 50);
        let d = mk(6, false, 0);
        let e = mk(7, true, 0);
        let f = mk(8, false, 50);
        let g = mk(9, true, 100);
        let mut queue = vec![a, c, d, b, e, g, f];
        queue.sort_unstable_by(super::compare_spip);
        assert_eq!(queue, vec![f, d, e, c, a, b, g]);
    }
}
