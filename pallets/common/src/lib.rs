#![cfg_attr(not(feature = "std"), no_std)]

pub mod constants;

pub mod traits;
pub use traits::{asset, balances, group, identity, multisig, CommonTrait};

pub mod context;
pub use context::Context;

pub mod batch_dispatch_info;
pub use batch_dispatch_info::BatchDispatchInfo;

use core::convert::From;
use polymesh_primitives::IdentityId;

#[derive(Debug, Clone, Copy)]
pub enum SystematicIssuers {
    GovernanceCommittee,
    CDDProvider,
}

impl SystematicIssuers {
    pub fn as_bytes(self) -> &'static [u8; 32] {
        use constants::did::{CDD_PROVIDERS_ID, GOVERNANCE_COMMITTEE_ID};

        match self {
            SystematicIssuers::GovernanceCommittee => GOVERNANCE_COMMITTEE_ID,
            SystematicIssuers::CDDProvider => CDD_PROVIDERS_ID,
        }
    }

    pub fn as_id(self) -> IdentityId {
        IdentityId::from(*self.as_bytes())
    }
}
