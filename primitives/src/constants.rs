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

#![allow(missing_docs)]

use crate::IdentityId;
use frame_support::PalletId;

/// Money matters.
pub mod currency {
    use crate::Balance;
    pub const POLY: Balance = 1_000_000;
    pub const ONE_POLY: Balance = POLY;
    pub const DOLLARS: Balance = POLY;
    pub const CENTS: Balance = DOLLARS / 100;
    pub const MILLICENTS: Balance = CENTS / 1_000;
    pub const ONE_UNIT: Balance = 1_000_000;
    pub const MAX_SUPPLY: Balance = ONE_UNIT * 1_000_000_000_000;
}

/// DID-related.
pub mod did {
    /// prefix for user dids
    pub const USER: &[u8; 5] = b"USER:";

    /// Governance Committee DID. It is used in systematic CDD claim for Governance Committee members.
    pub const GOVERNANCE_COMMITTEE_DID: &[u8; 32] = b"system:governance_committee\0\0\0\0\0";
    /// CDD Providers DID. It is used in systematic CDD claim for CDD Providers.
    pub const CDD_PROVIDERS_DID: &[u8; 32] = b"system:customer_due_diligence\0\0\0";
    /// Treasury module DID. It is used in systematic CDD claim for the Treasury module.
    pub const TREASURY_DID: &[u8; 32] = b"system:treasury_module_did\0\0\0\0\0\0";
    /// Block Reward Reserve DID.
    pub const BLOCK_REWARD_RESERVE_DID: &[u8; 32] = b"system:block_reward_reserve_did\0";
    /// Settlement module DID
    pub const SETTLEMENT_MODULE_DID: &[u8; 32] = b"system:settlement_module_did\0\0\0\0";
    /// Polymath Classic / Ethereum migration DID.
    pub const CLASSIC_MIGRATION_DID: &[u8; 32] = b"system:polymath_classic_mig\0\0\0\0\0";
    /// Fiat Currency Reservation DID
    pub const FIAT_TICKERS_RESERVATION_DID: &[u8; 32] = b"system:fiat_tickers_reservation\0";
    /// Technical Committee DID.
    pub const TECHNICAL_COMMITTEE_DID: &[u8; 32] = b"system:technical_committee\0\0\0\0\0\0";
    /// Upgrade Committee DID.
    pub const UPGRADE_COMMITTEE_DID: &[u8; 32] = b"system:upgrade_committee\0\0\0\0\0\0\0\0";
}

/// Priorities for the task that get scheduled.
pub mod queue_priority {
    use frame_support::traits::schedule::Priority;

    /// Queue priority for the settlement instruction execution.
    pub const SETTLEMENT_INSTRUCTION_EXECUTION_PRIORITY: Priority = 100;
}

// ERC1400 transfer status codes
pub const ERC1400_TRANSFER_FAILURE: u8 = 0x50;
pub const ERC1400_INSUFFICIENT_BALANCE: u8 = 0x52;
pub const ERC1400_INSUFFICIENT_ALLOWANCE: u8 = 0x53;
pub const ERC1400_FUNDS_LOCKED: u8 = 0x55;
pub const ERC1400_INVALID_SENDER: u8 = 0x56;
pub const ERC1400_INVALID_RECEIVER: u8 = 0x57;
pub const ERC1400_INVALID_OPERATOR: u8 = 0x58;

// Application-specific status codes
pub const INVALID_SENDER_DID: u8 = 0xa0;
pub const INVALID_RECEIVER_DID: u8 = 0xa1;
pub const INVALID_GRANULARITY: u8 = 0xa4;
pub const APP_TX_VOLUME_LIMIT_REACHED: u8 = 0xa5;
pub const APP_BLOCKED_TX: u8 = 0xa6;
pub const APP_FUNDS_LOCKED: u8 = 0xa7;
pub const APP_FUNDS_LIMIT_REACHED: u8 = 0xa8;
pub const CUSTODIAN_ERROR: u8 = 0xaa;

// PIP pallet constants.
pub const PIP_MAX_REPORTING_SIZE: usize = 1024;

/// Module ids, used for deriving sovereign account IDs for modules.
pub const TREASURY_PALLET_ID: PalletId = PalletId(*b"pm/trsry");
pub const BRR_PALLET_ID: PalletId = PalletId(*b"pm/blrwr");
pub const GC_PALLET_ID: PalletId = PalletId(*b"pm/govcm");
pub const CDD_PALLET_ID: PalletId = PalletId(*b"pm/cusdd");
pub const SETTLEMENT_PALLET_ID: PalletId = PalletId(*b"pm/setmn");
pub const CLASSIC_MIGRATION_PALLET_ID: PalletId = PalletId(*b"pm/ehmig");
pub const FIAT_TICKERS_RESERVATION_PALLET_ID: PalletId = PalletId(*b"pm/ftres");

/// Base module constants
pub const ENSURED_MAX_LEN: u32 = 2048;

/// SystematicIssuers (poorly named - should be SystematicIdentities) are identities created and maintained by the chain itself.
/// These identities are associated with a primary key derived from their name, and for which there is
/// no possible known private key.
/// Some of these identities are considered DID registrars:
/// - Committee: Issues CDD claims to members of committees (i.e. technical, GC) and is used for GC initiated CDD claims.
/// - CDDProvider: Issues CDD claims to other identities that need to transact POLYX (treasury, brr, rewards) as well as DID Registrars themselves
/// Committee members have a systematic CDD claim to ensure they can operate independently of permissioned DID registrars if needed.
/// DID Registrars have a systematic CDD claim to avoid a circular root of trust
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
            SystematicIssuers::Committee => GC_PALLET_ID,
            SystematicIssuers::CDDProvider => CDD_PALLET_ID,
            SystematicIssuers::Treasury => TREASURY_PALLET_ID,
            SystematicIssuers::BlockRewardReserve => BRR_PALLET_ID,
            SystematicIssuers::Settlement => SETTLEMENT_PALLET_ID,
            SystematicIssuers::ClassicMigration => CLASSIC_MIGRATION_PALLET_ID,
            SystematicIssuers::FiatTickersReservation => FIAT_TICKERS_RESERVATION_PALLET_ID,
        }
    }
}

pub const GC_DID: IdentityId = SystematicIssuers::Committee.as_id();
pub const TECHNICAL_DID: IdentityId = IdentityId(*did::TECHNICAL_COMMITTEE_DID);
pub const UPGRADE_DID: IdentityId = IdentityId(*did::UPGRADE_COMMITTEE_DID);

/// Prefixes for scheduled actions
pub const SETTLEMENT_INSTRUCTION_EXECUTION: [u8; 27] = *b"SETTLEMENT_INSTRUCTION_EXEC";
pub const PIP_EXECUTION: [u8; 8] = *b"PIP_EXEC";
pub const PIP_EXPIRY: [u8; 10] = *b"PIP_EXPIRY";
