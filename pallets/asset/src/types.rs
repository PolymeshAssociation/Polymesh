#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode};
use scale_info::TypeInfo;

use polymesh_primitives::asset::AssetType;
use polymesh_primitives::{Balance, IdentityId};

/// Ownership status of a ticker/token.
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo, PartialEq, Eq)]
pub enum AssetOwnershipRelation {
    #[default]
    NotOwned,
    TickerOwned,
    AssetOwned,
}

/// struct to store the token details.
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo, PartialEq, Eq)]
pub struct SecurityToken {
    pub total_supply: Balance,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
}

/// struct to store the ticker registration details.
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo, PartialEq, Eq)]
pub struct TickerRegistration<T> {
    pub owner: IdentityId,
    pub expiry: Option<T>,
}

/// struct to store the ticker registration config.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo, PartialEq, Eq)]
pub struct TickerRegistrationConfig<T> {
    pub max_ticker_length: u8,
    pub registration_length: Option<T>,
}

/// Enum that represents the current status of a ticker.
#[derive(Clone, Debug, Decode, Encode, PartialEq, Eq)]
pub enum TickerRegistrationStatus {
    RegisteredByOther,
    Available,
    RegisteredByDid,
}
