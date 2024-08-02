use super::{
    multisig::{create_multisig_default_perms, create_signers},
    storage::{get_last_auth_id, make_account_without_cdd, register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::assert_noop;
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use pallet_test_utils as test_utils;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_common_utilities::Context;
use polymesh_primitives::{Signatory, TransactionError};
use polymesh_runtime_develop::runtime::{CddHandler, RuntimeCall};
use sp_keyring::AccountKeyring;
use sp_runtime::transaction_validity::InvalidTransaction;

type MultiSig = multisig::Pallet<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;

#[test]
fn cdd_checks() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Bob.to_account_id()])
        .monied(true)
        .build()
        .execute_with(|| {
            // alice does not have cdd
            make_account_without_cdd(AccountKeyring::Alice.to_account_id()).unwrap();
            let alice_account = AccountKeyring::Alice.to_account_id();
            let alice_signatory = Signatory::Account(alice_account.clone());

            // charlie has valid cdd
            let _ = register_keyring_account(AccountKeyring::Charlie).unwrap();
            let charlie_account = AccountKeyring::Charlie.to_account_id();
            let charlie_signatory = Signatory::Account(charlie_account.clone());

            // register did bypasses cdd checks
            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::TestUtils(test_utils::Call::register_did {
                        secondary_keys: Default::default()
                    }),
                    &alice_account
                ),
                Ok(Some(AccountKeyring::Alice.to_account_id()))
            );

            // reset current identity context which is set as a side effect of get_valid_payer
            Context::set_current_identity::<Identity>(None);

            // normal tx without cdd should fail
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // reset current identity context which is set as a side effect of get_valid_payer
            Context::set_current_identity::<Identity>(None);

            // call to accept being a multisig signer should fail when invalid auth
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer { auth_id: 0 }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8)
            );

            // call to accept being a multisig signer should fail when authorizer does not have a valid cdd (expired)
            create_multisig_default_perms(
                alice_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );

            let alice_auth_id = get_last_auth_id(&alice_signatory);
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // reset current identity context which is set as a side effect of get_valid_payer
            Context::set_current_identity::<Identity>(None);

            // call to remove authorisation with issuer paying should fail if issuer does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        _auth_issuer_pays: true
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to remove authorisation with caller paying should fail if caller does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        _auth_issuer_pays: false
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // check that authorisation can be removed correctly
            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );
            let alice_auth_id = get_last_auth_id(&alice_signatory);

            // call to remove authorisation with caller paying should fail if caller does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        _auth_issuer_pays: false
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to remove authorisation with issuer paying should succeed as issuer has CDD
            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        _auth_issuer_pays: true
                    }),
                    &alice_account
                ),
                Ok(Some(AccountKeyring::Charlie.to_account_id()))
            );

            // reset current identity context which is set as a side effect of get_valid_payer
            Context::set_current_identity::<Identity>(None);

            // create an authorisation where the target has a CDD claim and the issuer does not
            create_multisig_default_perms(
                alice_account.clone(),
                create_signers(vec![charlie_account.clone()]),
                1,
            );
            let charlie_auth_id = get_last_auth_id(&charlie_signatory);

            // call to remove authorisation with issuer paying should fail if issuer does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory.clone(),
                        auth_id: charlie_auth_id,
                        _auth_issuer_pays: true
                    }),
                    &charlie_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to remove authorisation with caller paying should succeed as caller has CDD
            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory,
                        auth_id: charlie_auth_id,
                        _auth_issuer_pays: false
                    }),
                    &charlie_account
                ),
                Ok(Some(AccountKeyring::Charlie.to_account_id()))
            );

            // reset current identity context which is set as a side effect of get_valid_payer
            Context::set_current_identity::<Identity>(None);

            // call to accept being a multisig signer should succeed when authorizer has a valid cdd but signer key does not
            // fee must be paid by multisig creator
            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );
            let alice_auth_id = get_last_auth_id(&alice_signatory);

            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    &alice_account
                ),
                Ok(Some(AccountKeyring::Charlie.to_account_id()))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    &charlie_account
                ),
                Ok(Some(AccountKeyring::Charlie.to_account_id()))
            );
        });
}
