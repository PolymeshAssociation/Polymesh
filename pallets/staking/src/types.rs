#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

use codec::{Decode, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_runtime::RuntimeDebug;

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
