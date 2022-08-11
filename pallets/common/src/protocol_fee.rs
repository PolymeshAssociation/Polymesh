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
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// Protocol fee operations.
#[derive(Decode, Encode, TypeInfo)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ProtocolOp {
    AssetRegisterTicker,
    AssetIssue,
    AssetAddDocuments,
    AssetCreateAsset,
    CheckpointCreateSchedule,
    ComplianceManagerAddComplianceRequirement,
    IdentityCddRegisterDid,
    IdentityAddClaim,
    IdentityAddSecondaryKeysWithAuthorization,
    PipsPropose,
    ContractsPutCode,
    CorporateBallotAttachBallot,
    CapitalDistributionDistribute,
}

/// Common interface to protocol fees for runtime modules.
pub trait ChargeProtocolFee<AccountId> {
    /// Computes the fee of the operation and charges it to the given signatory.
    /// Equivalent to `charge_fees(&[op])`.
    fn charge_fee(op: ProtocolOp) -> DispatchResult;

    /// Computes the fee of the operations and charges them to the given signatory.
    /// If all fees cannot be charged, none are. That is, the operation is transactional.
    /// When `ops.is_empty()`, no storage changes may be made.
    fn charge_fees(ops: &[ProtocolOp]) -> DispatchResult;

    /// Computes the fee for `count` similar operations, and charges that fee to the given
    /// signatory.
    fn batch_charge_fee(op: ProtocolOp, count: usize) -> DispatchResult;
}
