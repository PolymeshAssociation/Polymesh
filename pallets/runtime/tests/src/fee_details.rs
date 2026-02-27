use frame_support::assert_noop;
use sp_keyring::Sr25519Keyring;
use sp_runtime::transaction_validity::InvalidTransaction;

use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_primitives::{traits::CurrentFeePayer, Signatory, TransactionError};
use polymesh_runtime_develop::runtime::{RuntimeCall, TxFeeHandler};

use super::multisig::{create_multisig_default_perms, create_signers};
use super::storage::{get_last_auth_id, make_account_without_cdd};
use super::storage::{register_keyring_account, TestStorage};
use super::ExtBuilder;

type MultiSig = multisig::Pallet<TestStorage>;
type Balances = balances::Pallet<TestStorage>;
type Identity = identity::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;

#[test]
fn did_checks() {
    ExtBuilder::default()
        .did_registrars(vec![Sr25519Keyring::Bob.to_account_id()])
        .monied(true)
        .build()
        .execute_with(|| {
            // alice does not have a DID registered by a DID registrar
            make_account_without_cdd(Sr25519Keyring::Alice.to_account_id()).unwrap();
            let alice_account = Sr25519Keyring::Alice.to_account_id();
            let alice_signatory = Signatory::Account(alice_account.clone());

            // charlie has a valid DID
            let _ = register_keyring_account(Sr25519Keyring::Charlie).unwrap();
            let charlie_account = Sr25519Keyring::Charlie.to_account_id();
            let charlie_signatory = Signatory::Account(charlie_account.clone());

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Alice.to_account_id()))
            );

            // call to accept being a multisig signer should fail when invalid auth
            assert_noop!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer { auth_id: 0 }),
                    alice_account.clone()
                ),
                InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8)
            );

            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );

            let alice_auth_id = get_last_auth_id(&alice_signatory);
            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: true
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: false
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Alice.to_account_id()))
            );

            // check that authorisation can be removed correctly
            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );
            let alice_auth_id = get_last_auth_id(&alice_signatory);

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: false
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Alice.to_account_id()))
            );

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: true
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );

            // create an authorisation where the target has a CDD claim and the issuer does not
            create_multisig_default_perms(
                alice_account.clone(),
                create_signers(vec![charlie_account.clone()]),
                1,
            );
            let charlie_auth_id = get_last_auth_id(&charlie_signatory);

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory.clone(),
                        auth_id: charlie_auth_id,
                        auth_issuer_pays: true
                    }),
                    charlie_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Alice.to_account_id()))
            );

            // call to remove authorisation with caller paying should succeed as caller has CDD
            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory,
                        auth_id: charlie_auth_id,
                        auth_issuer_pays: false
                    }),
                    charlie_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );

            // call to accept being a multisig signer should succeed when authorizer has a valid cdd but signer key does not
            // fee must be paid by multisig creator
            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );
            let alice_auth_id = get_last_auth_id(&alice_signatory);

            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    alice_account.clone()
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                TxFeeHandler::get_valid_payer(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    charlie_account
                ),
                Ok(Some(Sr25519Keyring::Charlie.to_account_id()))
            );
        });
}
