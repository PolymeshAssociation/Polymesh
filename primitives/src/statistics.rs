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

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{hash::Hash, hash::Hasher, ops::Deref, ops::DerefMut, prelude::*};

pub type Counter = u64;
pub type Percentage = HashablePermill;

/// Transfer managers that can be attached to a Token for compliance.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferManager {
    /// CTM limits the number of active investors in a Token.
    CountTransferManager(Counter),
    /// PTM limits the percentage of token owned by a single Identity.
    PercentageTransferManager(Percentage),
}

/// Result of a transfer manager check
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransferManagerResult {
    pub tm: TransferManager,
    pub failed: bool,
}

/// Wrapper around `sp_arithmetic::Permill`
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct HashablePermill(pub sp_arithmetic::Permill);

impl Hash for HashablePermill {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write(&self.0.deconstruct().to_le_bytes())
    }
}

impl Deref for HashablePermill {
    type Target = sp_arithmetic::Permill;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for HashablePermill {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}
