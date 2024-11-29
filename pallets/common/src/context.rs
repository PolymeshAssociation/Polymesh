use crate::traits::identity::IdentityFnTrait;
use sp_std::marker::PhantomData;

/// Helper class to access to some context information.
/// Currently it allows to access to
///     - `current_payer throught an `IdentityFnTrait`, because it is stored using extrinsics.
#[derive(Default)]
pub struct Context<AccountId> {
    _marker: PhantomData<AccountId>,
}

impl<AccountId> Context<AccountId> {
    #[inline]
    pub fn current_payer<I: IdentityFnTrait<AccountId>>() -> Option<AccountId> {
        I::current_payer()
    }

    #[inline]
    pub fn set_current_payer<I: IdentityFnTrait<AccountId>>(payer: Option<AccountId>) {
        I::set_current_payer(payer)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polymesh_primitives::{AccountId, IdentityId};

    use sp_keyring::AccountKeyring;
    use std::{collections::BTreeMap, convert::From};

    struct IdentityTest {}

    impl IdentityFnTrait<AccountId> for IdentityTest {
        fn get_identity(key: &AccountId) -> Option<IdentityId> {
            let keys: BTreeMap<AccountId, u128> = vec![
                (AccountId::from(AccountKeyring::Alice.public().0), 1u128),
                (AccountId::from(AccountKeyring::Bob.public().0), 2u128),
                (AccountId::from(AccountKeyring::Charlie.public().0), 3u128),
            ]
            .into_iter()
            .collect();

            if let Some(id) = keys.get(key) {
                Some(IdentityId::from(*id))
            } else {
                None
            }
        }

        fn current_payer() -> Option<AccountId> {
            None
        }

        fn set_current_payer(_payer: Option<AccountId>) {}

        fn has_valid_cdd(_target_did: IdentityId) -> bool {
            true
        }
    }
}
