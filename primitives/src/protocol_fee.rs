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

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};

/// Protocol fee operations.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, MaxEncodedLen)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub enum ProtocolOp {
    /// Fee charged when registering a new ticker.
    AssetRegisterTicker,
    /// Fee charged when issuing assets.
    AssetIssue,
    /// Fee charged when adding documents to an asset.
    AssetAddDocuments,
    /// Fee charged when creating a new asset.
    AssetCreateAsset,
    /// Fee charged when creating a checkpoint schedule.
    CheckpointCreateSchedule,
    /// Fee charged when adding compliance requirements.
    ComplianceManagerAddComplianceRequirement,
    /// Fee charged when registering a DID.
    IdentityRegisterDid,
    /// Fee charged when adding a claim to an identity.
    IdentityAddClaim,
    /// Fee charged when adding secondary keys with authorization.
    IdentityAddSecondaryKeysWithAuthorization,
    /// Fee charged when making a proposal in PIPs.
    PipsPropose,
    /// Fee charged when uploading contract code.
    ContractsPutCode,
    /// Fee charged when attaching a ballot to corporate actions.
    CorporateBallotAttachBallot,
    /// Fee charged when distributing capital.
    CapitalDistributionDistribute,
    /// Fee charged when creating an NFT collection.
    NFTCreateCollection,
    /// Fee charged when minting NFTs.
    NFTMint,
    /// Fee charged when creating a child identity.
    IdentityCreateChildIdentity,
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
