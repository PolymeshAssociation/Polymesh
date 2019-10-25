use parity_scale_codec::{Decode, Encode};
use rstd::prelude::Vec;

use crate::{IdentityRole, Key, SigningKey};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct DidRecord<U> {
    pub roles: Vec<IdentityRole>,
    pub master_key: Key,
    pub signing_keys: Vec<SigningKey>,
    pub balance: U,
}

impl<U> DidRecord<U> {
    /// It checks if this entity contains role `role`.
    pub fn has_role(&self, role: IdentityRole) -> bool {
        self.roles.contains(&role)
    }
}
