use crate::{identity, identity::LinkedKeyInfo, runtime, Runtime};
use primitives::{AccountKey, IdentityId, TransactionError};

use sp_runtime::{
    traits::SignedExtension,
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
};

use codec::{Decode, Encode};
use core::convert::TryFrom;
use frame_support::dispatch::DispatchInfo;
use sp_std::marker::PhantomData;

type Identity = identity::Module<Runtime>;
type Call = runtime::Call;

/// This signed extension double-checks and updates the current identifier extracted from
/// the caller account in each transaction.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct UpdateDid<T: frame_system::Trait + Send + Sync>(PhantomData<T>);

impl<T: frame_system::Trait + Send + Sync> UpdateDid<T> {
    pub fn new() -> Self {
        UpdateDid(PhantomData)
    }

    /// It extracts the `IdentityId` associated with `who` account.
    fn identity_from_key(who: &T::AccountId) -> Option<IdentityId> {
        if let Ok(who_key) = AccountKey::try_from(who.encode()) {
            if let Some(linked_key_info) = Identity::key_to_identity_ids(&who_key) {
                if let LinkedKeyInfo::Unique(id) = linked_key_info {
                    return Some(id);
                }
            }
        }
        None
    }
}

impl<T: frame_system::Trait + Send + Sync> sp_std::fmt::Debug for UpdateDid<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "UpdateDid")
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: frame_system::Trait + Send + Sync> SignedExtension for UpdateDid<T> {
    type AccountId = T::AccountId;
    type Call = runtime::Call;
    type AdditionalSigned = ();
    type DispatchInfo = DispatchInfo;
    type Pre = ();

    fn additional_signed(&self) -> sp_std::result::Result<(), TransactionValidityError> {
        Ok(())
    }

    /// It ensures that transaction caller account has an associated identity and
    /// that identity has been validated by any KYC.
    /// The current identity will be accesible through `Identity::current_did`.
    ///
    /// Only the following methods can be called with no identity:
    ///     - `identity::register_did`
    fn validate(
        &self,
        who: &Self::AccountId,
        call: &Self::Call,
        _: DispatchInfo,
        _: usize,
    ) -> TransactionValidity {
        match call {
            // Add here any function from any module which does *not* need a current identity.
            Call::Identity(identity::Call::register_did(..))
            | Call::Identity(identity::Call::authorize_join_to_identity(..))
            | Call::Identity(identity::Call::accept_master_key(..))
            | Call::Identity(identity::Call::accept_authorization(..))
            | Call::Identity(identity::Call::batch_accept_authorization(..)) => {
                Ok(ValidTransaction::default())
            }
            // Other calls should be identified
            _ => {
                let id_opt = Self::identity_from_key(who);
                if let Some(id) = id_opt.clone() {
                    if Identity::has_valid_kyc(id) {
                        Identity::set_current_did(id_opt);
                        Ok(ValidTransaction::default())
                    } else {
                        Err(InvalidTransaction::Custom(TransactionError::RequiredKYC as u8).into())
                    }
                } else {
                    sp_runtime::print("ERROR: This transaction requires an Identity");
                    Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
                }
            }
        }
    }

    /// It clears the `Identity::current_did` after transaction.
    fn post_dispatch(_pre: Self::Pre, _info: Self::DispatchInfo, _len: usize) {
        Identity::set_current_did(None);
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateDid;
    use crate::{
        identity, runtime,
        test::storage::{build_ext, register_keyring_account_with_balance, Identity, TestStorage},
        Runtime,
    };
    use core::default::Default;
    use frame_support::dispatch::DispatchInfo;
    use primitives::TransactionError;
    use sp_runtime::{
        traits::SignedExtension,
        transaction_validity::{InvalidTransaction, ValidTransaction},
    };
    use test_client::AccountKeyring;

    type Call = runtime::Call;
    type IdentityCall = identity::Call<Runtime>;

    #[test]
    fn update_did_tests() {
        build_ext().execute_with(&update_did_tests_with_externalities);
    }

    fn update_did_tests_with_externalities() {
        let update_did_se = <UpdateDid<TestStorage>>::new();
        let dispatch_info = DispatchInfo::default();

        let alice_signed = AccountKeyring::Alice.public();
        let alice_id =
            register_keyring_account_with_balance(AccountKeyring::Alice, 10_000).unwrap();

        // let bob_id = register_keyring_account_with_balance( AccountKeyring::Bob, 5_000).unwrap();
        let charlie_signed = AccountKeyring::Charlie.public();

        let valid_transaction_ok = Ok(ValidTransaction::default());

        // `Identity::register_did` does not need an DID associated and check `current_did` is
        // none.
        let register_did_call_1 = Call::Identity(IdentityCall::register_did(vec![]));
        assert_eq!(
            update_did_se.validate(&alice_signed, &register_did_call_1, dispatch_info, 0usize),
            valid_transaction_ok
        );
        assert_eq!(Identity::current_did(), None);

        // `Identity::add_signing_items` needs DID. `validate` updates `current_did` and
        // `post_dispatch` clears it.
        let add_signing_items_1 = Call::Identity(IdentityCall::add_signing_items(alice_id, vec![]));
        assert_eq!(
            update_did_se.validate(&alice_signed, &add_signing_items_1, dispatch_info, 0),
            valid_transaction_ok
        );
        assert_eq!(Identity::current_did(), Some(alice_id));
        <UpdateDid<TestStorage>>::post_dispatch((), dispatch_info, 0);
        assert_eq!(Identity::current_did(), None);

        // `Identity::freeze_signing_keys` fails because `c_acc` account has not a DID.
        let freeze_call1 = Call::Identity(IdentityCall::freeze_signing_keys(alice_id));
        assert_eq!(
            update_did_se.validate(&charlie_signed, &freeze_call1, dispatch_info, 0usize),
            Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
        );
    }
}
