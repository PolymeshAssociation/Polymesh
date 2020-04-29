use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_common_utilities::traits::{balances::CheckCdd, identity::Trait as IdentityTrait};
use polymesh_primitives::{AccountKey, IdentityId};

use codec::Encode;
use core::convert::TryFrom;
use frame_support::StorageMap;

pub struct CddChecker<R>(sp_std::marker::PhantomData<R>);

impl<R> CheckCdd for CddChecker<R>
where
    R: IdentityTrait + multisig::Trait,
{
    fn check_key_cdd(key: &AccountKey) -> bool {
        Self::get_key_cdd_did(key).is_some()
    }

    fn get_key_cdd_did(key: &AccountKey) -> Option<IdentityId> {
        if let Some(did) = identity::Module::<R>::get_identity(&key) {
            if identity::Module::<R>::has_valid_cdd(did) {
                return Some(did);
            }
        }
        // An account that is NOT a signing key or the master key of an Identity
        // but is a signer of a multisig that is a signing/master key of an Identity
        if <multisig::KeyToMultiSig<R>>::contains_key(&key) {
            let ms = <multisig::KeyToMultiSig<R>>::get(&key);
            if let Ok(ms_key) = AccountKey::try_from(ms.encode()) {
                if let Some(did) = identity::Module::<R>::get_identity(&ms_key) {
                    if identity::Module::<R>::has_valid_cdd(did) {
                        return Some(did);
                    }
                }
            }
        }
        return None;
    }
}
