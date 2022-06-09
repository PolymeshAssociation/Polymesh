// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

use crate::statistics::{v1, AssetScope, Percentage, StatClaim, StatOpType, StatType};
use crate::{ClaimType, IdentityId};
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::*;

/// Transfer condition.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransferCondition {
    /// Maximum investor count.
    MaxInvestorCount(u64),
    /// Maximum % ownership.
    MaxInvestorOwnership(Percentage),

    /// Restrict investor count based on different claim types.
    /// Can be used for limiting the number of non-accredited investors or the number of investors in a jurisdiction.
    /// (StatClaim, Issuer, Min, Max)
    ClaimCount(StatClaim, IdentityId, u64, Option<u64>),

    /// Restrict % ownership based on different claim types.
    /// Use-cases:
    /// * min/max % ownership in a jurisdiction.  (min % ownership for a jurisdiction could be used,
    ///    if the company wants to keep 51% ownership in their home country).
    /// * min/max % ownership for Accredited/non-accredited.
    /// (StatClaim, Issuer, Min, Max)
    ClaimOwnership(StatClaim, IdentityId, Percentage, Percentage),
}

impl TransferCondition {
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

    /// Get TransferConditionExemptKey needed by this transfer condition.
    pub fn get_exempt_key(&self, asset: AssetScope) -> TransferConditionExemptKey {
        let (op, claim_type) = match self {
            Self::MaxInvestorCount(_) => (StatOpType::Count, None),
            Self::MaxInvestorOwnership(_) => (StatOpType::Balance, None),
            Self::ClaimCount(claim, _, _, _) => (StatOpType::Count, Some(claim.claim_type())),
            Self::ClaimOwnership(claim, _, _, _) => (StatOpType::Balance, Some(claim.claim_type())),
        };
        TransferConditionExemptKey {
            asset,
            op,
            claim_type,
        }
    }
}

impl From<v1::TransferManager> for TransferCondition {
    fn from(old: v1::TransferManager) -> Self {
        match old {
            v1::TransferManager::CountTransferManager(max) => Self::MaxInvestorCount(max),
            v1::TransferManager::PercentageTransferManager(max) => Self::MaxInvestorOwnership(max),
        }
    }
}

/// Result of a transfer condition check.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct TransferConditionResult {
    /// Transfer condition that was checked.
    pub condition: TransferCondition,
    /// Final evaluation result.
    pub result: bool,
}

impl From<v1::TransferManagerResult> for TransferConditionResult {
    fn from(old: v1::TransferManagerResult) -> Self {
        Self {
            condition: old.tm.into(),
            result: old.result,
        }
    }
}

/// Transfer Condition Exempt key.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct TransferConditionExemptKey {
    /// Asset scope.
    pub asset: AssetScope,
    /// Stats operation type.
    pub op: StatOpType,
    /// Claim type.
    pub claim_type: Option<ClaimType>,
}

/// List of transfer compliance requirements associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct AssetTransferCompliance {
    /// This flag indicates if asset transfer compliance should be enforced.
    pub paused: bool,
    /// List of transfer compliance requirements.
    pub requirements: BTreeSet<TransferCondition>,
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
