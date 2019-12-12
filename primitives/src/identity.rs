use codec::{Decode, Encode};
use rstd::prelude::Vec;

use crate::{IdentityRole, Key, Signer, SigningItem};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct Identity {
    pub roles: Vec<IdentityRole>,
    pub master_key: Key,
    pub signing_items: Vec<SigningItem>,
}

impl Identity {
    /// It checks if this entity contains IdentityRole `role`.
    pub fn has_role(&self, role: IdentityRole) -> bool {
        self.roles.contains(&role)
    }

    /// It adds `new_signing_keys` to `self`.
    /// It also keeps its internal list sorted and removes duplicate elements.
    pub fn add_signing_items(&mut self, new_signing_items: &[SigningItem]) -> &mut Self {
        self.signing_items.extend_from_slice(new_signing_items);
        self.signing_items.sort();
        self.signing_items.dedup();

        self
    }

    /// It removes `keys_to_remove` from signing keys.
    pub fn remove_signing_items(&mut self, signers_to_remove: &[Signer]) -> &mut Self {
        self.signing_items.retain(|curr_si| {
            signers_to_remove
                .iter()
                .find(|&signer| curr_si.signer == *signer)
                .is_none()
        });

        self
    }
}
