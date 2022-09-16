use crate::impl_checked_inc;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// A `Ticker`-local Agent Group ID.
/// By *local*, we mean that the same number might be used for a different `Ticker`
/// to uniquely identify a different Agent Group.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Default, Debug)]
pub struct AGId(pub u32);
impl_checked_inc!(AGId);

/// The available set of agent groups.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum AgentGroup {
    /// Has all permissions.
    Full,
    /// Custom defined agent group drawn from 2).
    /// The other groups have hard-coded mappings to `Permissions` in code.
    Custom(AGId),
    /// Manages identities of agents themselves.
    ExceptMeta,
    /// Agent group corresponding to a Corporate Action Agent (CAA) on Polymesh Mainnet v1.
    PolymeshV1CAA,
    /// Agent group corresponding to a Primary Issuance Agent (PIA) on Polymesh Mainnet v1.
    PolymeshV1PIA,
}
