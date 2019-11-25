use crate::{balances, identity, identity::LinkedKeyInfo, runtime, Runtime};
use primitives::{IdentityId, Key, TransactionError};

use sr_primitives::{
    traits::SignedExtension,
    transaction_validity::{
        InvalidTransaction, TransactionValidity, TransactionValidityError, ValidTransaction,
    },
};

use codec::{Decode, Encode};
use core::convert::TryFrom;
use rstd::marker::PhantomData;
use srml_support::dispatch::DispatchInfo;

type Identity = identity::Module<Runtime>;
type Balance = balances::Module<Runtime>;
type Call = runtime::Call;

/// This signed extension double-checks and updates the current identifier extracted from
/// the caller account in each transaction.
///
/// # TODO
/// - After transaction, Do we need to clean `CurrentDid`?
/// - Move outside `lib.rs`. It needs a small refactor to extract `Runtime` from here.
#[derive(Default, Encode, Decode, Clone, Eq, PartialEq)]
pub struct UpdateDid<T: system::Trait + Send + Sync>(PhantomData<T>);

impl<T: system::Trait + Send + Sync> UpdateDid<T> {
    fn new() -> Self {
        UpdateDid(PhantomData)
    }

    /// It extracts the `IdentityId` associated with `who` account.
    fn identity_from_key(who: &T::AccountId) -> Option<IdentityId> {
        if let Ok(who_key) = Key::try_from(who.encode()) {
            if let Some(linked_key_info) = Identity::key_to_identity_ids(&who_key) {
                if let LinkedKeyInfo::Unique(id) = linked_key_info {
                    return Some(id);
                }
            }
        }
        None
    }
}

#[cfg(feature = "std")]
impl<T: system::Trait + Send + Sync> rstd::fmt::Debug for UpdateDid<T> {
    fn fmt(&self, f: &mut rstd::fmt::Formatter) -> rstd::fmt::Result {
        write!(f, "UpdateDid")
    }
}

impl<T: system::Trait + Send + Sync> SignedExtension for UpdateDid<T> {
    type AccountId = T::AccountId;
    // type AccountId = AccId;
    type Call = runtime::Call;
    type AdditionalSigned = ();
    type Pre = ();

    fn additional_signed(&self) -> rstd::result::Result<(), TransactionValidityError> {
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
            Call::Identity(identity::Call::register_did(..)) => Ok(ValidTransaction::default()),
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
                    sr_primitives::print("ERROR: This transaction requires an Identity");
                    Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::UpdateDid;
    use crate::{
        balances, identity, runtime,
        test::storage::{build_ext, make_account_with_balance, TestStorage},
        Runtime,
    };
    use codec::Encode;
    use core::default::Default;
    use primitives::TransactionError;
    use sr_io::with_externalities;
    use sr_primitives::{
        traits::{SignedExtension, Verify},
        transaction_validity::{InvalidTransaction, ValidTransaction},
        AnySignature,
    };
    use srml_support::dispatch::DispatchInfo;
    use test_client::{self, AccountKeyring};

    type AccountId = <AnySignature as Verify>::Signer;

    type Identity = identity::Module<TestStorage>;
    type Balances = balances::Module<TestStorage>;
    type Call = runtime::Call;
    type BalancesCall = balances::Call<Runtime>;
    type IdentityCall = identity::Call<Runtime>;

    #[test]
    fn update_did_tests() {
        with_externalities(&mut build_ext(), &update_did_tests_with_externalities);
    }

    fn update_did_tests_with_externalities() {
        let update_did_se = <UpdateDid<TestStorage>>::new();
        let dispatch_info = DispatchInfo::default();

        let (a_acc, b_acc, c_acc) = (1u64, 2u64, 3u64);
        let (alice, alice_id) = make_account_with_balance(a_acc, 10_000).unwrap();
        let (bob, _bob_id) = make_account_with_balance(b_acc, 5_000).unwrap();

        let register_did_call_1 = Call::Identity(IdentityCall::register_did(alice_id, vec![]));
        assert_eq!(
            update_did_se.validate(&a_acc, &register_did_call_1, dispatch_info, 0usize),
            Ok(ValidTransaction::default())
        );

        let freeze_call1 = Call::Identity(IdentityCall::freeze_signing_keys(alice_id));
        assert_eq!(
            update_did_se.validate(&c_acc, &freeze_call1, dispatch_info, 0usize),
            Err(InvalidTransaction::Custom(TransactionError::MissingIdentity as u8).into())
        );
    }
}
