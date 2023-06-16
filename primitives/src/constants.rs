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

//! Shareable types.

#![allow(missing_docs)]

/// Prefixes for scheduled actions
pub const SETTLEMENT_INSTRUCTION_EXECUTION: [u8; 27] = *b"SETTLEMENT_INSTRUCTION_EXEC";
pub const MULTISIG_PROPOSAL_EXECUTION: [u8; 22] = *b"MULTISIG_PROPOSAL_EXEC";
pub const PIP_EXECUTION: [u8; 8] = *b"PIP_EXEC";
pub const PIP_EXPIRY: [u8; 10] = *b"PIP_EXPIRY";
