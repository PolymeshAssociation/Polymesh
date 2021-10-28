// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2021 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::statistics::{Percentage, StatType};
use crate::{Claim, ClaimType, IdentityId, Scope};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Transfer condition type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum TransferConditionType {
    /// Maximum investor count.
    MaxInvestorCount(u64),
    /// Maximum % ownership.
    MaxInvestorOwnership(Percentage),

    /// Restrict investor count based on different claim types.
    /// Can be used for limiting the number of non-accredited investors or the number of investors in a jurisdiction.
    /// (Claim, Min, Max)
    ClaimCount(Claim, Option<u64>, Option<u64>),

    /// Restrict % ownership based on different claim types.
    /// Use-cases:
    /// * min/max % ownership in a jurisdiction.  (min % ownership for a jurisdiction could be used,
    ///    if the company wants to keep 51% ownership in their home country).
    /// * min/max % ownership for Accredited/non-accredited.
    /// (Claim, Min, Max)
    ClaimOwnership(Claim, Option<Percentage>, Percentage),
}

impl TransferConditionType {
    /// Get the `StatType` needed by this transfer condition.
    pub fn needed_stat_type(&self) -> StatType {
        match self {
            Self::MaxInvestorCount(_) => StatType::Count(None),
            Self::MaxInvestorOwnership(_) => StatType::Balance(None),
            Self::ClaimCount(claim, _, _) => StatType::Count(Some(claim.claim_type())),
            Self::ClaimOwnership(claim, _, _) => StatType::Balance(Some(claim.claim_type())),
        }
    }

    /// Get the `ClaimType` from this transfer condition.
    pub fn claim_type(&self) -> Option<ClaimType> {
        self.as_claim().and_then(|claim| claim.claim_type())
    }

    /// The claim of this transfer condtion.
    pub fn as_claim(&self) -> Option<&Claim> {
        match self {
            Self::MaxInvestorCount(_) => None,
            Self::MaxInvestorOwnership(_) => None,
            Self::ClaimCount(claim, _, _) => Some(claim),
            Self::ClaimOwnership(claim, _, _) => Some(claim),
        }
    }

    /// The scope of this transfer condtion.
    pub fn as_scope(&self) -> Option<&Scope> {
        self.as_claim().and_then(|claim| claim.as_scope())
    }
}

/// Transfer condition.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct TransferCondition {
    /// Type of transfer condition.
    pub transfer_type: TransferConditionType,
    /// Trusted issuers.
    pub issuers: Vec<IdentityId>,
    // TODO: possibly add list of Exempt claims (i.e. "Affiliate" claim) that this transfer condition doesn't apply to.
    // pub exempt_claims: Vec<Claim>,
}

/// List of transfer compliance requirements associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize, Debug))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
pub struct AssetTransferCompliance {
    /// This flag indicates if asset transfer compliance should be enforced.
    pub paused: bool,
    /// List of transfer compliance requirements.
    pub requirements: Vec<TransferCondition>,
}
