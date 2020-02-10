use crate::traits::identity::IdentityTrait;

use polymesh_primitives::{AccountKey, IdentityId};

/*
use lazy_static::lazy_static;
use parking_lot::{RwLock},
use std::sync::{Arc, RwLock};

lazy_static! {
    static ref  CONTEXT: Arc<RwLock<Context>> = Arc::new( RwLock::new( Context::default()));
}

#[derive(Default)]
pub struct Context {
    identity: Option<IdentityId>
}

impl Context {
    pub fn current_identity() -> Option<IdentityId> {
        CONTEXT.read().unwrap().identity.clone()
    }

    pub fn set_current_identity( id: Option<IdentityId>) {
        CONTEXT.write().unwrap().identity = id
    }

    pub fn current_identity_or<I: IdentityTrait>( key: &AccountKey) -> Option<IdentityId> {
        Self::current_identity().or_else( || I::get_identity(key))
    }
}
*/

#[derive(Default)]
pub struct Context {}

impl Context {
    pub fn current_identity<I: IdentityTrait>() -> Option<IdentityId> {
        I::current_identity()
    }

    pub fn set_current_identity<I: IdentityTrait>(id: Option<IdentityId>) {
        I::set_current_identity(id)
    }

    pub fn current_identity_or<I: IdentityTrait>(key: &AccountKey) -> Option<IdentityId> {
        Self::current_identity::<I>().or_else(|| I::get_identity(key))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use polymesh_primitives::{Permission, Signatory};

    use std::{collections::BTreeMap, convert::From, thread};
    use test_client::AccountKeyring;

    struct IdentityTest {}
    // key2Id: BTreeMap<AccountId, IdentityId>,

    impl IdentityTrait for IdentityTest {
        fn get_identity(key: &AccountKey) -> Option<IdentityId> {
            let keys: BTreeMap<AccountKey, u128> = vec![
                (AccountKey::from(AccountKeyring::Alice.public().0), 1u128),
                (AccountKey::from(AccountKeyring::Bob.public().0), 2u128),
                (AccountKey::from(AccountKeyring::Charlie.public().0), 3u128),
            ]
            .into_iter()
            .collect();

            if let Some(id) = keys.get(key) {
                Some(IdentityId::from(*id))
            } else {
                None
            }
        }

        fn is_signer_authorized(_did: IdentityId, _signer: &Signatory) -> bool {
            false
        }

        fn is_master_key(_did: IdentityId, _key: &AccountKey) -> bool {
            false
        }

        fn is_signer_authorized_with_permissions(
            _did: IdentityId,
            _signer: &Signatory,
            _permissions: Vec<Permission>,
        ) -> bool {
            false
        }
    }

    #[test]
    fn context_functions() -> Result<(), &'static str> {
        assert_eq!(Context::current_identity(), None);
        Context::set_current_identity(Some(IdentityId::from(42)));

        let _ = thread::spawn(|| {
            let id = Context::current_identity();
            assert_eq!(id, Some(IdentityId::from(42u128)));
            Context::set_current_identity(None);
        })
        .join()
        .map_err(|_| "Poison error")?;

        assert_eq!(Context::current_identity(), None);

        let _ = thread::spawn(|| {
            let id = Context::current_identity();
            assert_eq!(id, None);
            Context::set_current_identity(Some(IdentityId::from(15)));
        })
        .join()
        .map_err(|_| "Poison error")?;

        assert_eq!(Context::current_identity(), Some(IdentityId::from(15)));

        // Check "or" option.
        let alice = AccountKey::from(AccountKeyring::Alice.public().0);
        assert_eq!(
            Context::current_identity_or::<IdentityTest>(&alice),
            Some(IdentityId::from(15))
        );
        Context::set_current_identity(None);
        assert_eq!(
            Context::current_identity_or::<IdentityTest>(&alice),
            Some(IdentityId::from(1))
        );

        let eve = AccountKey::from(AccountKeyring::Eve.public().0);
        assert_eq!(Context::current_identity_or::<IdentityTest>(&eve), None);

        Ok(())
    }
}
