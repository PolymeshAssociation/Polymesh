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

use frame_support::dispatch::DispatchResult;

/// Mesh Improvement Proposal id. Used offchain.
pub type PipId = u32;

pub trait EnactProposalMaker<Origin, Hash> {
    fn is_pip_id_valid(id: PipId) -> bool;

    fn propose(origin: Origin, id: PipId) -> DispatchResult;

    fn enact_referendum_hash(id: PipId) -> Hash;
}
