use rstd::prelude::Vec;

use crate::entity::{IdentityRole, KRole, Key, RoledKey};

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct DidRecord<U> {
    pub roles: Vec<IdentityRole>,
    pub master_key: Key,
    pub signing_keys: Vec<RoledKey>,
    pub balance: U,
}

impl<U> DidRecord<U> {
    pub fn has_role(&self, role: IdentityRole) -> bool {
        self.roles.contains(&role)
    }

    pub fn has_signing_keys_role(&self, role: KRole) -> bool {
        self.signing_keys
            .iter()
            .find(|&rk| rk.has_role(role))
            .is_some()
    }
}
