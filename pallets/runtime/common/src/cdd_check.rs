
use pallet_multisig as multisig;
use pallet_identity as identity;
use polymesh_common_utilities::traits::balances::CheckCdd;
use polymesh_primitives::AccountKey;

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::StorageMap;

//#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct CddChecker<R>(sp_std::marker::PhantomData<R>);

impl<R> CheckCdd for CddChecker<R>
where
    R: identity::Trait + multisig::Trait,
{
    fn check_key_cdd(key: &AccountKey) -> bool {
        // An account is that is a signing key or the master key of an Identity
        if let Some(did) = identity::Module::<R>::get_identity(&key) {
            if identity::Module::<R>::has_valid_cdd(did) {
                return true;
            }
        }
        // An account that is NOT a signing key or the master key of an Identity
        // but is a signer of a multisig that is a signing/master key of an Identity
        if <multisig::KeyToMultiSig<R>>::contains_key(&key) {
            let ms = <multisig::KeyToMultiSig<R>>::get(&key);
            if let Ok(ms_key) = AccountKey::try_from(ms.encode()) {
                if let Some(did) = identity::Module::<R>::get_identity(&ms_key) {
                    return identity::Module::<R>::has_valid_cdd(did);
                }
            }
        }
        return false;
    }
}
