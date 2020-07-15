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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;

pub mod traits;
pub use traits::{
    asset, balances, compliance_manager, exemption, governance_group, group, identity, multisig,
    pip, CommonTrait,
};

pub mod context;
pub use context::Context;

pub mod batch_dispatch_info;
pub use batch_dispatch_info::BatchDispatchInfo;

pub mod protocol_fee;
pub use protocol_fee::ChargeProtocolFee;

use core::convert::From;
use polymesh_primitives::IdentityId;
use sp_runtime::ModuleId;

/// It defines the valid issuers for Systematic Claims.
///
/// Systematic claims are claims generated by the system itself, and they are removed automatically
/// too.
///
/// There is just one Systematic claim at the moment, and It is generated by two issuers.
/// It is a CDD claim that is assigned to members of specific groups:
/// * Governance Committee: Each member of the committee has a CDD claim generated by
/// `SystematicIssuers::Committee` in order to allow it to operate independently of
/// claims generated by CDD trusted providers. Using that claim, GC members could operate even if
/// their CDD claim's issuer has been revoked.
/// * CDD Service Providers: Every CDD providers has a CDD claim generated by
/// `SystematicIssuers::CDDProvider` group, in order to avoid self-generated claim issue.
#[derive(Debug, Clone, Copy)]
pub enum SystematicIssuers {
    Committee,
    CDDProvider,
    Treasury,
    BlockRewardReserve,
    Settlement,
}

impl core::fmt::Display for SystematicIssuers {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let value = match self {
            SystematicIssuers::Committee => "Governance Committee",
            SystematicIssuers::CDDProvider => "CDD Trusted Providers",
            SystematicIssuers::Treasury => "Treasury",
            SystematicIssuers::BlockRewardReserve => "Block Reward Reserve",
            SystematicIssuers::Settlement => "Settlement module",
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
];

impl SystematicIssuers {
    /// It returns the representation of this issuer as a raw public key.
    pub fn as_bytes(self) -> &'static [u8; 32] {
        use constants::did::{
            BLOCK_REWARD_RESERVE_DID, CDD_PROVIDERS_DID, GOVERNANCE_COMMITTEE_DID,
            SETTLEMENT_MODULE_DID, TREASURY_DID,
        };

        match self {
            SystematicIssuers::Committee => GOVERNANCE_COMMITTEE_DID,
            SystematicIssuers::CDDProvider => CDD_PROVIDERS_DID,
            SystematicIssuers::Treasury => TREASURY_DID,
            SystematicIssuers::BlockRewardReserve => BLOCK_REWARD_RESERVE_DID,
            SystematicIssuers::Settlement => SETTLEMENT_MODULE_DID,
        }
    }

    /// It returns the Identity Identifier of this issuer.
    pub fn as_id(self) -> IdentityId {
        IdentityId::from(*self.as_bytes())
    }

    pub fn as_module_id(self) -> ModuleId {
        use constants::{
            BRR_MODULE_ID, CDD_MODULE_ID, GC_MODULE_ID, SETTLEMENT_MODULE_ID, TREASURY_MODULE_ID,
        };

        match self {
            SystematicIssuers::Committee => GC_MODULE_ID,
            SystematicIssuers::CDDProvider => CDD_MODULE_ID,
            SystematicIssuers::Treasury => TREASURY_MODULE_ID,
            SystematicIssuers::BlockRewardReserve => BRR_MODULE_ID,
            SystematicIssuers::Settlement => SETTLEMENT_MODULE_ID,
        }
    }
}
