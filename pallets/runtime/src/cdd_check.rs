use crate::{multisig, Runtime};

use polymesh_primitives::AccountKey;
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
        // An account is that is a signing key or the master key of an Identity
        if let Some(did) = Identity::get_identity(&key) {
            if Identity::has_valid_cdd(did) {
                return true;
            }
        }
        // An account that is NOT a signing key or the master key of an Identity
        // but is a signer of a multisig that is a signing/master key of an Identity
        if <multisig::KeyToMultiSig<Runtime>>::contains_key(&key) {
            let ms = <multisig::KeyToMultiSig<Runtime>>::get(&key);
            if let Ok(ms_key) = AccountKey::try_from(ms.encode()) {
                if let Some(did) = Identity::get_identity(&ms_key) {
                    return Identity::has_valid_cdd(did);
                }
            }
        }
        return false;
    }
}
