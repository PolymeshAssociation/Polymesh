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
        if let Some(did) = Identity::get_identity(&key) {
            if Identity::has_valid_cdd(did) {
                return true;
            }
        }
        if <multisig::KeyToMultiSig<Runtime>>::contains_key(&key) {
            let ms = <multisig::KeyToMultiSig<Runtime>>::get(&key);
            if let Ok(ms_key) = AccountKey::try_from(ms.encode()) {
                if let Some(did) = Identity::get_identity(&ms_key) {
                    if Identity::has_valid_cdd(did) {
                        return true;
                    }
                }
            }
            let ms_creator_did = <multisig::MultiSigCreator<Runtime>>::get(ms);
            return Identity::has_valid_cdd(ms_creator_did);
        }
        return false;
    }
}
