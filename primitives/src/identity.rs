use codec::{Decode, Encode};
use rstd::prelude::Vec;

use crate::{IdentityRole, Key, SigningKey};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Identity {
    pub roles: Vec<IdentityRole>,
    pub master_key: Key,
    pub signing_keys: Vec<SigningKey>,
}

impl Identity {
    /// It checks if this entity contains IdentityRole `role`.
    pub fn has_role(&self, role: IdentityRole) -> bool {
        self.roles.contains(&role)
    }

    /// It adds `new_signing_keys` to `self`.
    /// It also keeps its internal list sorted and removes duplicate elements.
    pub fn add_signing_keys(&mut self, new_signing_keys: &[SigningKey]) -> &mut Self {
        self.signing_keys.extend_from_slice(new_signing_keys);
        self.signing_keys.sort();
        self.signing_keys.dedup();

        self
    }

    /// It removes `keys_to_remove` from signing keys.
    pub fn remove_signing_keys(&mut self, keys_to_remove: &[Key]) -> &mut Self {
        self.signing_keys
            .retain(|skey| keys_to_remove.iter().find(|&rk| skey == rk).is_none());

        self
    }
}
