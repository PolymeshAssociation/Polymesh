use crate::{Trait};
use codec::{Encode, Decode};
use frame_support::{decl_module, decl_storage};
use sp_std::prelude::*;

/// Old storage that needs to be deprecated
#[derive(Encode, Decode, Clone, PartialOrd, Ord, Eq, PartialEq, Debug)]
pub enum Compliance {
    /// Compliance requirements not met.
    Pending,
    /// CDD compliant. Eligible to participate in validation.
    Active,
}

impl Default for Compliance {
    fn default() -> Self {
        Compliance::Pending
    }
}

/// Represents a requirement that must be met to be eligible to become a validator.
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Encode, Decode)]
pub struct PermissionedValidator {
    /// Indicates the status of CDD compliance.
    pub compliance: Compliance,
}

impl Default for PermissionedValidator {
    fn default() -> Self {
        Self {
            compliance: Compliance::default(),
        }
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin { }
}

decl_storage! {
    pub trait Store for Module<T: Trait> as Staking {
        /// The map from (wannabe) validators to the status of compliance.
        pub PermissionedValidators get(permissioned_validators):
            linked_map hasher(twox_64_concat) T::AccountId => Option<PermissionedValidator>;
    }
}
