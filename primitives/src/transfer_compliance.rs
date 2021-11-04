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

use crate::statistics::{Percentage, StatOpType, StatType};
use crate::{Claim, ClaimType, IdentityId, Scope};
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::prelude::*;

/// Transfer condition.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub enum TransferCondition {
    /// Maximum investor count.
    MaxInvestorCount(u64),
    /// Maximum % ownership.
    MaxInvestorOwnership(Percentage),

    /// Restrict investor count based on different claim types.
    /// Can be used for limiting the number of non-accredited investors or the number of investors in a jurisdiction.
    /// (Claim, Issuer, Min, Max)
    ClaimCount(Claim, IdentityId, u64, Option<u64>),

    /// Restrict % ownership based on different claim types.
    /// Use-cases:
    /// * min/max % ownership in a jurisdiction.  (min % ownership for a jurisdiction could be used,
    ///    if the company wants to keep 51% ownership in their home country).
    /// * min/max % ownership for Accredited/non-accredited.
    /// (Claim, Issuer, Min, Max)
    ClaimOwnership(Claim, IdentityId, Percentage, Percentage),
}

impl TransferCondition {
    /// Get the `ClaimType` from this transfer condition.
    pub fn claim_type(&self) -> Option<ClaimType> {
        self.get_claim().map(|claim| claim.claim_type())
    }

    /// The claim of this transfer condtion.
    pub fn get_claim(&self) -> Option<&Claim> {
        match self {
            Self::MaxInvestorCount(_) => None,
            Self::MaxInvestorOwnership(_) => None,
            Self::ClaimCount(claim, _, _, _) => Some(claim),
            Self::ClaimOwnership(claim, _, _, _) => Some(claim),
        }
    }

    /// The claim & issuer of this transfer condtion.
    pub fn get_claim_issuer(&self) -> Option<(&Claim, &IdentityId)> {
        match self {
            Self::MaxInvestorCount(_) => None,
            Self::MaxInvestorOwnership(_) => None,
            Self::ClaimCount(claim, issuer, _, _) => Some((claim, issuer)),
            Self::ClaimOwnership(claim, issuer, _, _) => Some((claim, issuer)),
        }
    }

    /// The scope of this transfer condtion.
    pub fn get_scope(&self) -> Option<&Scope> {
        self.get_claim().and_then(|claim| claim.as_scope())
    }

    /// Get StatType needed by this transfer condition.
    pub fn get_stat_type(&self) -> StatType {
        let (op, claim_issuer) = match self {
            Self::MaxInvestorCount(_) => (StatOpType::Count, None),
            Self::MaxInvestorOwnership(_) => (StatOpType::Balance, None),
            Self::ClaimCount(claim, issuer, _, _) => (
                StatOpType::Count,
                Some((claim.claim_type(), issuer.clone())),
            ),
            Self::ClaimOwnership(claim, issuer, _, _) => (
                StatOpType::Balance,
                Some((claim.claim_type(), issuer.clone())),
            ),
        };
        StatType { op, claim_issuer }
    }
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

impl AssetTransferCompliance {
    /// Get list of StatTypes from the transfer conditions.
    pub fn get_stat_types(&self) -> Vec<StatType> {
        self.requirements
            .iter()
            .map(|cond| cond.get_stat_type())
            .collect()
    }
}
