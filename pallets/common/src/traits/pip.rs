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

/// Polymesh Improvement Proposal (PIP) id.
pub type PipId = u32;

/// A result to enact for one or many PIPs in the snapshot queue.
// This type is only here due to `enact_snapshot_results`.
#[derive(codec::Encode, codec::Decode, Copy, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum SnapshotResult {
    /// Approve the PIP and move it to the execution queue.
    Approve,
    /// Reject the PIP, removing it from future consideration.
    Reject,
    /// Skip the PIP, bumping the `skipped_count`,
    /// or fail if the threshold for maximum skips is exceeded.
    Skip,
}

/// Utility maker used to link `Call` type, defined at `Runtime` level, from inside any module.
pub trait PipsCommitteeBridge<Call> {
    fn approve_committee_proposal(id: PipId) -> Call;
    fn reject_proposal(id: PipId) -> Call;
    fn prune_proposal(id: PipId) -> Call;
    fn enact_snapshot_results(results: Vec<(u8, SnapshotResult)>) -> Call;
}
