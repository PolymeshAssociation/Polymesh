use codec::{Decode, Encode};
use sp_core::sr25519::Public;
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::From, prelude::Vec};

use crate::{AccountKey, IdentityRole, Signatory, SigningItem};

/// Identity information.
#[allow(missing_docs)]
#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct Identity {
    pub roles: Vec<IdentityRole>,
    pub master_key: AccountKey,
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
    pub fn remove_signing_items(&mut self, signers_to_remove: &[Signatory]) -> &mut Self {
        self.signing_items.retain(|curr_si| {
            signers_to_remove
                .iter()
                .find(|&signer| curr_si.signer == *signer)
                .is_none()
        });

        self
    }
}

impl From<AccountKey> for Identity {
    fn from(acc: AccountKey) -> Self {
        Identity {
            master_key: acc,
            ..Default::default()
        }
    }
}

impl From<Public> for Identity {
    fn from(p: Public) -> Self {
        Identity {
            master_key: AccountKey::from(p.0),
            ..Default::default()
        }
    }
}
