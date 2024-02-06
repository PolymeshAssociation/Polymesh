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

/// Stores the details of a security token.
#[derive(Clone, Debug, Decode, Default, Encode, TypeInfo, PartialEq, Eq)]
pub struct SecurityToken {
    pub total_supply: Balance,
    pub owner_did: IdentityId,
    pub divisible: bool,
    pub asset_type: AssetType,
}

impl SecurityToken {
    /// Creates a new [`SecurityToken`] instance.
    pub fn new(
        total_supply: Balance,
        owner_did: IdentityId,
        divisible: bool,
        asset_type: AssetType,
    ) -> Self {
        Self {
            total_supply,
            owner_did,
            divisible,
            asset_type,
        }
    }
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
