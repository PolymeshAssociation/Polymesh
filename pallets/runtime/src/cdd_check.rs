use crate::{multisig, Runtime};

use polymesh_primitives::{AccountKey, IdentityId};
use polymesh_runtime_common::traits::balances::CheckCdd;
use polymesh_runtime_identity as identity;

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::StorageMap;

type Identity = identity::Module<Runtime>;

#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct CddChecker;

impl CheckCdd for CddChecker {
    fn check_key_cdd(key: &AccountKey) -> bool {
        Self::get_key_cdd_did(key).is_some()
    }

    fn get_key_cdd_did(key: &AccountKey) -> Option<IdentityId> {
        if let Some(did) = Identity::get_identity(&key) {
            if Identity::has_valid_cdd(did) {
                return Some(did);
            }
        }
        // An account that is NOT a signing key or the master key of an Identity
        // but is a signer of a multisig that is a signing/master key of an Identity
        if <multisig::KeyToMultiSig<Runtime>>::contains_key(&key) {
            let ms = <multisig::KeyToMultiSig<Runtime>>::get(&key);
            if let Ok(ms_key) = AccountKey::try_from(ms.encode()) {
                if let Some(did) = Identity::get_identity(&ms_key) {
                    if Identity::has_valid_cdd(did) {
                        return Some(did);
                    }
                }
            }
        }
        return None;
    }
}
