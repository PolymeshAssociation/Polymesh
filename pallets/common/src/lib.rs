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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;

pub mod traits;
pub use traits::{
    asset, balances, base, compliance_manager, governance_group, group, identity, multisig, nft,
    portfolio, transaction_payment, CommonConfig, TestUtilsFn,
};
pub mod context;
pub use context::Context;

pub mod protocol_fee;
pub use protocol_fee::ChargeProtocolFee;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchs;

use core::ops::Add;
use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use frame_support::PalletId;
use polymesh_primitives::IdentityId;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};
use sp_runtime::{DispatchError, DispatchResult};

/// Use `GetExtra` as the trait bounds for pallet `Config` parameters
/// that will be used for bounded collections.
pub trait GetExtra<T>: Get<T> + Clone + core::fmt::Debug + Default + PartialEq + Eq {}

/// ConstSize type wrapper.
///
/// This allows the use of Bounded collections in extrinsic parameters.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstSize<const T: u32>;

impl<const T: u32> Get<u32> for ConstSize<T> {
    fn get() -> u32 {
        T
    }
}

impl<const T: u32> GetExtra<u32> for ConstSize<T> {}

/// SystematicIssuers (poorly named - should be SystematicIdentities) are identities created and maintained by the chain itself.
/// These identities are associated with a primary key derived from their name, and for which there is
/// no possible known private key.
/// Some of these identities are considered CDD providers:
/// - Committee: Issues CDD claims to members of committees (i.e. technical, GC) and is used for GC initiated CDD claims.
/// - CDDProvider: Issues CDD claims to other identities that need to transact POLYX (treasury, brr, rewards) as well as CDD Providers themselves
/// Committee members have a systematic CDD claim to ensure they can operate independently of permissioned CDD providers if needed.
/// CDD Providers have a systematic CDD claim to avoid a circular root of trust
#[derive(Debug, Clone, Copy)]
pub enum SystematicIssuers {
    Committee,
    CDDProvider,
    Treasury,
    BlockRewardReserve,
    Settlement,
    ClassicMigration,
    FiatTickersReservation,
}

impl core::fmt::Display for SystematicIssuers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value = match self {
            SystematicIssuers::Committee => "Committee",
            SystematicIssuers::CDDProvider => "CDD Trusted Providers",
            SystematicIssuers::Treasury => "Treasury",
            SystematicIssuers::BlockRewardReserve => "Block Reward Reserve",
            SystematicIssuers::Settlement => "Settlement module",
            SystematicIssuers::ClassicMigration => "Polymath Classic Imports and Reservations",
            SystematicIssuers::FiatTickersReservation => "Fiat Ticker Reservation",
        };

        write!(f, "'{}'", value)
    }
}

pub const SYSTEMATIC_ISSUERS: &[SystematicIssuers] = &[
    SystematicIssuers::Treasury,
    SystematicIssuers::Committee,
    SystematicIssuers::CDDProvider,
    SystematicIssuers::BlockRewardReserve,
    SystematicIssuers::Settlement,
    SystematicIssuers::ClassicMigration,
    SystematicIssuers::FiatTickersReservation,
];

impl SystematicIssuers {
    /// Returns the representation of this issuer as a raw public key.
    pub const fn as_bytes(self) -> &'static [u8; 32] {
        use constants::did;
        match self {
            SystematicIssuers::Committee => did::GOVERNANCE_COMMITTEE_DID,
            SystematicIssuers::CDDProvider => did::CDD_PROVIDERS_DID,
            SystematicIssuers::Treasury => did::TREASURY_DID,
            SystematicIssuers::BlockRewardReserve => did::BLOCK_REWARD_RESERVE_DID,
            SystematicIssuers::Settlement => did::SETTLEMENT_MODULE_DID,
            SystematicIssuers::ClassicMigration => did::CLASSIC_MIGRATION_DID,
            SystematicIssuers::FiatTickersReservation => did::FIAT_TICKERS_RESERVATION_DID,
        }
    }

    /// It returns the Identity Identifier of this issuer.
    pub const fn as_id(self) -> IdentityId {
        IdentityId(*self.as_bytes())
    }

    pub const fn as_pallet_id(self) -> PalletId {
        match self {
            SystematicIssuers::Committee => constants::GC_PALLET_ID,
            SystematicIssuers::CDDProvider => constants::CDD_PALLET_ID,
            SystematicIssuers::Treasury => constants::TREASURY_PALLET_ID,
            SystematicIssuers::BlockRewardReserve => constants::BRR_PALLET_ID,
            SystematicIssuers::Settlement => constants::SETTLEMENT_PALLET_ID,
            SystematicIssuers::ClassicMigration => constants::CLASSIC_MIGRATION_PALLET_ID,
            SystematicIssuers::FiatTickersReservation => {
                constants::FIAT_TICKERS_RESERVATION_PALLET_ID
            }
        }
    }
}

pub const GC_DID: IdentityId = SystematicIssuers::Committee.as_id();

/// Execute the supplied function in a new storage transaction,
/// committing on `Ok(_)` and rolling back on `Err(_)`, returning the result.
///
/// Transactions can be arbitrarily nested with commits happening to the parent.
pub fn with_transaction<T, E: From<DispatchError>>(
    tx: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    use frame_support::storage::{with_transaction, TransactionOutcome};
    with_transaction(|| match tx() {
        r @ Ok(_) => TransactionOutcome::Commit(r),
        r @ Err(_) => TransactionOutcome::Rollback(r),
    })
}

/// In one transaction, execute the supplied function `tx` on each element in `iter`.
///
/// See `with_transaction` for details.
pub fn with_each_transaction<A>(
    iter: impl IntoIterator<Item = A>,
    tx: impl FnMut(A) -> DispatchResult,
) -> DispatchResult {
    with_transaction(|| iter.into_iter().try_for_each(tx))
}

/// Either a block number, or nothing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MaybeBlock<BlockNumber> {
    Some(BlockNumber),
    None,
}

impl<T> Default for MaybeBlock<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: Add<Output = T>> Add<T> for MaybeBlock<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        match self {
            MaybeBlock::Some(lhs) => MaybeBlock::Some(lhs + rhs),
            MaybeBlock::None => MaybeBlock::None,
        }
    }
}
