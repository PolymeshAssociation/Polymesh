use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// A `Ticker`-local Agent Group ID.
/// By *local*, we mean that the same number might be used for a different `Ticker`
/// to uniquely identify a different Agent Group.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Default, Debug)]
pub struct AGId(pub u32);

/// The available set of agent groups.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Encode, Decode, Debug)]
pub enum AgentGroup {
    /// Has all permissions.
    Full,
    /// Custom defined agent group drawn from 2).
    /// The other groups have hard-coded mappings to `Permissions` in code.
    Custom(AGId),
    /// Manages identities of agents themselves.
    Meta,
    /// Agent group corresponding to a Corporate Action Agent (CAA) on Polymesh Mainnet v1.
    PolymeshV1CAA,
    /// Agent group corresponding to a Primary Issuance Agent (PIA) on Polymesh Mainnet v1.
    PolymeshV1PIA,
}
