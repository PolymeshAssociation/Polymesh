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

/// Mesh Improvement Proposal id. Used offchain.
pub type PipId = u32;

/// Utility maker used to link `Call` type, defined at `Runtime` level, from inside any module.
pub trait EnactProposalMaker<Origin, Call> {
    /// Checks if `id` is a valid PIP identifier.
    fn is_pip_id_valid(id: PipId) -> bool;

    /// It creates the call to enactment the Pip given by `id`.
    fn enact_referendum_call(id: PipId) -> Call;

    /// It creates the call to reject the Pip, given by `id`.
    fn reject_referendum_call(id: PipId) -> Call;
}
