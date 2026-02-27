// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2024 Polymesh Association

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
use scale_info::TypeInfo;

use crate::settlement::ReceiptMetadata;
use crate::{impl_checked_inc, IdentityId, Ticker};

/// The per-AssetId ID of a fundraiser.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct FundraiserId(pub u64);
impl_checked_inc!(FundraiserId);

/// An offchain fundraiser receipt.
#[derive(
    Encode,
    Decode,
    MaxEncodedLen,
    Clone,
    PartialEq,
    Eq,
    Debug,
    PartialOrd,
    Ord
)]
pub struct FundraiserReceipt<Balance> {
    /// The [`FundraiserId`] of the STO fundraiser for which the receipt is for.
    fundraiser_id: FundraiserId,
    /// The [`IdentityId`] of the sender.
    sender_identity: IdentityId,
    /// The [`IdentityId`] of the receiver.
    receiver_identity: IdentityId,
    /// [`Ticker`] of the asset being transferred.
    ticker: Ticker,
    /// The amount transferred.
    amount: Balance,
}

impl<Balance> FundraiserReceipt<Balance> {
    /// Creates a new [`FundraiserReceipt`].
    pub fn new(
        fundraiser_id: FundraiserId,
        sender_identity: IdentityId,
        receiver_identity: IdentityId,
        ticker: Ticker,
        amount: Balance,
    ) -> Self {
        Self {
            fundraiser_id,
            sender_identity,
            receiver_identity,
            ticker,
            amount,
        }
    }
}

/// Details about an offchain transaction receipt.
#[derive(Decode, DecodeWithMemTracking, Encode, Ord, PartialOrd)]
#[derive(Clone, Debug, Eq, MaxEncodedLen, PartialEq, TypeInfo)]
pub struct FundraiserReceiptDetails<AccountId, OffChainSignature, Moment> {
    /// Unique receipt number set by the signer for their receipts
    pub uid: u64,
    /// The [`AccountId`] of the Signer for this receipt.
    pub signer: AccountId,
    /// Signature confirming the receipt details.
    pub signature: OffChainSignature,
    /// The moment at which the receipt expires and can no longer be used for a STO.
    pub expires_at: Moment,
    /// The [`ReceiptMetadata`] that can be used to attach messages to receipts.
    pub metadata: Option<ReceiptMetadata>,
}

impl<AccountId, OffChainSignature, Moment>
    FundraiserReceiptDetails<AccountId, OffChainSignature, Moment>
{
    /// Creates a new [`FundraiserReceiptDetails`].
    pub fn new(
        uid: u64,
        signer: AccountId,
        signature: OffChainSignature,
        expires_at: Moment,
        metadata: Option<ReceiptMetadata>,
    ) -> Self {
        Self {
            uid,
            signer,
            signature,
            expires_at,
            metadata,
        }
    }
}
