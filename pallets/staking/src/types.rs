#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode, MaxEncodedLen};
use frame_support::dispatch::DispatchResult;
use scale_info::TypeInfo;
use sp_runtime::{Perbill, RuntimeDebug};

use crate::{ActiveEraInfo, Config};

/// A trait used by the staking pallet for permissioned staking.
///
/// A permissioned Substrate network can be configured to allow only a set of
/// identities to participate in staking. This trait is used to define the
/// behavior of the staking pallet in such a network.
pub trait PermissionedStaking<T: Config> {
    /// On validate hook.
    fn on_validate(_who: &T::AccountId, _commission: Perbill) -> DispatchResult {
        Ok(())
    }

    /// On chill hook.
    fn on_chill(_who: &T::AccountId) {}

    /// On nominate hook.
    fn on_nominate(_who: &T::AccountId) -> DispatchResult {
        Ok(())
    }

    /// Is the validator still compliant?
    fn is_validator_compliant(_who: &T::AccountId) -> bool {
        true
    }

    /// Is the nominator still compliant?
    fn is_nominator_compliant(_who: &T::AccountId) -> bool {
        true
    }

    /// Schedule reward payouts.
    fn schedule_payouts(_active_era: &ActiveEraInfo) {}

    /// Who should be slashed?
    fn who_to_slash() -> Option<WhoToSlash> {
        Some(WhoToSlash::ValidatorAndNominator)
    }

    /// Is slashing enabled?
    fn is_slashing_enabled() -> bool {
        Self::who_to_slash().is_some()
    }

    /// Slash nominators?
    fn slash_nominators() -> bool {
        Self::who_to_slash() == Some(WhoToSlash::ValidatorAndNominator)
    }
}

/// Preference of an identity regarding validation.
#[derive(Decode, Encode, RuntimeDebug, TypeInfo)]
#[derive(Clone, Copy, Eq, PartialEq)]
pub struct PermissionedIdentityPrefs {
    /// Intended number of validators an identity wants to run.
    ///
    /// Act as a hard limit on the number of validators an identity can run.
    /// However, it can be amended using governance.
    ///
    /// The count satisfies `count < MaxValidatorPerIdentity * Self::validator_count()`.
    pub intended_count: u32,
    /// Keeps track of the running number of validators of a DID.
    pub running_count: u32,
}

impl Default for PermissionedIdentityPrefs {
    fn default() -> Self {
        Self {
            intended_count: 1,
            running_count: 0,
        }
    }
}

impl PermissionedIdentityPrefs {
    pub fn new(intended_count: u32) -> Self {
        Self {
            intended_count,
            running_count: 0,
        }
    }
}

/// Who should be slashed.
#[derive(Decode, Encode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[derive(Clone, Copy, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum WhoToSlash {
    /// Allow validators but not nominators to get slashed.
    Validator,
    /// Allow both validators and nominators to get slashed.
    ValidatorAndNominator,
}

/// Switch used to change the "victim" for slashing. Victims can be
/// validators, both validators and nominators, or no-one.
#[derive(Decode, Encode, MaxEncodedLen, RuntimeDebug, TypeInfo)]
#[derive(Clone, Copy, Default, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SlashingSwitch {
    /// Allow validators but not nominators to get slashed.
    Validator,
    /// Allow both validators and nominators to get slashed.
    ValidatorAndNominator,
    /// Forbid slashing.
    #[default]
    None,
}

impl From<SlashingSwitch> for Option<WhoToSlash> {
    fn from(value: SlashingSwitch) -> Self {
        match value {
            SlashingSwitch::Validator => Some(WhoToSlash::Validator),
            SlashingSwitch::ValidatorAndNominator => Some(WhoToSlash::ValidatorAndNominator),
            SlashingSwitch::None => None,
        }
    }
}
