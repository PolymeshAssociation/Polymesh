use super::{
    storage::{get_last_auth_id, make_account_without_cdd, register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok};
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

type MultiSig = multisig::Module<TestStorage>;
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
            let (alice_signed, _) =
                make_account_without_cdd(AccountKeyring::Alice.to_account_id()).unwrap();
            let alice_account = AccountKeyring::Alice.to_account_id();
            let alice_key_signatory = Signatory::Account(AccountKeyring::Alice.to_account_id());
            let alice_account_signatory = Signatory::Account(alice_account.clone());

            // charlie has valid cdd
            let charlie_signed = Origin::signed(AccountKeyring::Charlie.to_account_id());
            let _ = register_keyring_account(AccountKeyring::Charlie).unwrap();
            let charlie_account = AccountKeyring::Charlie.to_account_id();
            let charlie_key_signatory = Signatory::Account(AccountKeyring::Charlie.to_account_id());
            let charlie_account_signatory = Signatory::Account(charlie_account.clone());

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
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer_as_key {
                        auth_id: 0
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8)
            );

            // call to accept being a multisig signer should fail when authorizer does not have a valid cdd (expired)
            assert_ok!(MultiSig::create_multisig(
                alice_signed.clone(),
                vec![alice_key_signatory.clone()],
                1,
            ));

            let alice_auth_id = get_last_auth_id(&alice_key_signatory);
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer_as_key {
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
                        target: alice_account_signatory.clone(),
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
                        target: alice_account_signatory.clone(),
                        auth_id: alice_auth_id,
                        _auth_issuer_pays: false
                    }),
                    &alice_account
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // check that authorisation can be removed correctly
            assert_ok!(MultiSig::create_multisig(
                charlie_signed.clone(),
                vec![alice_key_signatory.clone()],
                1,
            ));
            let alice_auth_id = get_last_auth_id(&alice_key_signatory);

            // call to remove authorisation with caller paying should fail if caller does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_account_signatory.clone(),
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
                        target: alice_account_signatory,
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
            assert_ok!(MultiSig::create_multisig(
                alice_signed.clone(),
                vec![Signatory::Account(AccountKeyring::Charlie.to_account_id())],
                1,
            ));
            let charlie_auth_id = get_last_auth_id(&charlie_key_signatory);

            // call to remove authorisation with issuer paying should fail if issuer does not have a valid cdd
            assert_noop!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_account_signatory.clone(),
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
                        target: charlie_account_signatory,
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
            assert_ok!(MultiSig::create_multisig(
                charlie_signed.clone(),
                vec![Signatory::Account(AccountKeyring::Alice.to_account_id())],
                1,
            ));
            let alice_auth_id = get_last_auth_id(&alice_key_signatory);

            assert_eq!(
                CddHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer_as_key {
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
