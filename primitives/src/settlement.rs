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
use sp_std::vec::Vec;

use crate::compliance_manager::ComplianceComplexity;

/// An object for tracking the complexity of the `execute_instruction` function.
#[derive(Clone, Debug, Decode, Eq, Encode, PartialEq, TypeInfo)]
pub struct ExecuteInstructionComplexity {
    /// Tracks the `is_compliant` complexity. Each leg in the instruction needs its own tracker.
    compliance_trackers: Vec<ComplianceComplexity>,
    /// Tracks the number of nfts in the instruction.
    non_fungible_tokens: u32,
    /// Tracks the number of fungible tokens in the instruction.
    fungible_tokens: u32,
}

impl ExecuteInstructionComplexity {
    /// Creates a new instance of `ExecuteInstructionComplexity`.
    pub fn new_2(c: Vec<ComplianceComplexity>, n: u32, f: u32) -> Self {
        ExecuteInstructionComplexity {
            compliance_trackers: c,
            non_fungible_tokens: n,
            fungible_tokens: f,
        }
    }

    /// Creates a new instance of `ExecuteInstructionComplexityTracker`.
    pub fn new() -> Self {
        ExecuteInstructionComplexity {
            compliance_trackers: Vec::new(),
            non_fungible_tokens: 0,
            fungible_tokens: 0,
        }
    }

    /// Adds a compliance tracker.
    pub fn add_compliance_tracker(&mut self, compliance_tracker: ComplianceComplexity) {
        self.compliance_trackers.push(compliance_tracker);
    }

    /// Sets the number of non_fungible_tokens.
    pub fn set_non_fungible_tokens(&mut self, non_fungible_tokens: u32) {
        self.non_fungible_tokens = non_fungible_tokens;
    }

    /// Sets the number of fungible_tokens.
    pub fn set_fungible_tokens(&mut self, fungible_tokens: u32) {
        self.fungible_tokens = fungible_tokens;
    }
}
