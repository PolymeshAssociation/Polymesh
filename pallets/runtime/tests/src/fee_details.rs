use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_runtime::transaction_validity::InvalidTransaction;

use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::{traits::CurrentFeePayer, Signatory, TransactionError};
use polymesh_primitives::{AuthorizationData, Permissions};
use polymesh_runtime_develop::runtime::{RuntimeCall, TxFeeHandler};

use super::multisig::{create_multisig_default_perms, create_signers};
use super::storage::{add_secondary_key, get_last_auth_id, make_account_without_cdd};
use super::storage::{register_keyring_account, TestStorage};
use super::ExtBuilder;

type MultiSig = multisig::Pallet<TestStorage>;
type Balances = balances::Pallet<TestStorage>;
type Identity = identity::Pallet<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type IdError = identity::Error<TestStorage>;

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
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(alice_account.clone(), None, None))
            );

            // call to accept being a multisig signer should fail when invalid auth
            assert_noop!(
                TxFeeHandler::call_payment_info(
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
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    charlie_account.clone(),
                    Some(alice_auth_id),
                    None
                ))
            );

            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: true
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Charlie.to_account_id(),
                    Some(alice_auth_id),
                    Some(alice_signatory.clone())
                ))
            );

            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: false
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Alice.to_account_id(),
                    None,
                    None,
                ))
            );

            // check that authorisation can be removed correctly
            create_multisig_default_perms(
                charlie_account.clone(),
                create_signers(vec![alice_account.clone()]),
                1,
            );
            let alice_auth_id = get_last_auth_id(&alice_signatory);

            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: false
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Alice.to_account_id(),
                    None,
                    None,
                ))
            );

            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: alice_signatory.clone(),
                        auth_id: alice_auth_id,
                        auth_issuer_pays: true
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Charlie.to_account_id(),
                    Some(alice_auth_id),
                    Some(alice_signatory.clone())
                ))
            );

            // create an authorisation where the target has a CDD claim and the issuer does not
            create_multisig_default_perms(
                alice_account.clone(),
                create_signers(vec![charlie_account.clone()]),
                1,
            );
            let charlie_auth_id = get_last_auth_id(&charlie_signatory);

            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory.clone(),
                        auth_id: charlie_auth_id,
                        auth_issuer_pays: true
                    }),
                    charlie_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Alice.to_account_id(),
                    Some(charlie_auth_id),
                    Some(charlie_signatory.clone())
                ))
            );

            // call to remove authorisation with caller paying should succeed as caller has CDD
            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::Identity(identity::Call::remove_authorization {
                        target: charlie_signatory.clone(),
                        auth_id: charlie_auth_id,
                        auth_issuer_pays: false
                    }),
                    charlie_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Charlie.to_account_id(),
                    None,
                    None
                ))
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
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::MultiSig(multisig::Call::accept_multisig_signer {
                        auth_id: alice_auth_id
                    }),
                    alice_account.clone()
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Charlie.to_account_id(),
                    Some(alice_auth_id),
                    None
                ))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                TxFeeHandler::call_payment_info(
                    &RuntimeCall::MultiSig(multisig::Call::change_sigs_required {
                        sigs_required: 1
                    }),
                    charlie_account
                ),
                Ok(CallPaymentInfo::new(
                    Sr25519Keyring::Charlie.to_account_id(),
                    None,
                    None
                ))
            );
        });
}

#[test]
fn stale_authorizations_not_paid_by_issuer_and_removable() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice_did = register_keyring_account(Sr25519Keyring::Alice).unwrap();
        let charlie_did = register_keyring_account(Sr25519Keyring::Charlie).unwrap();
        let alice = Origin::signed(Sr25519Keyring::Alice.to_account_id());
        let charlie = Origin::signed(Sr25519Keyring::Charlie.to_account_id());
        let bob_account = Sr25519Keyring::Bob.to_account_id();
        let bob = Origin::signed(bob_account.clone());
        let bob_signatory = Signatory::Account(bob_account.clone());
        let invalid_auth = InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8);

        add_secondary_key(alice_did, bob_account.clone());

        // A third party issues two invitations to Bob, who is then removed from Alice's identity.
        let invite_bob = || {
            assert_ok!(Identity::add_authorization(
                charlie.clone(),
                bob_signatory.clone(),
                AuthorizationData::JoinIdentity(Permissions::default()),
                None,
            ));
            get_last_auth_id(&bob_signatory)
        };
        let rejected_auth_id = invite_bob();
        let revoked_auth_id = invite_bob();
        assert_eq!(
            identity::NumberOfGivenAuths::<TestStorage>::get(charlie_did),
            2
        );
        assert_ok!(Identity::remove_secondary_keys(
            alice,
            vec![bob_account.clone()]
        ));

        let remove_call = |auth_id, auth_issuer_pays| {
            RuntimeCall::Identity(identity::Call::remove_authorization {
                target: bob_signatory.clone(),
                auth_id,
                auth_issuer_pays,
            })
        };

        // The issuer pays for nothing involving a stale authorization.
        assert_eq!(
            TxFeeHandler::call_payment_info(
                &remove_call(rejected_auth_id, true),
                bob_account.clone()
            ),
            Err(invalid_auth)
        );
        assert_eq!(
            TxFeeHandler::call_payment_info(
                &RuntimeCall::Identity(identity::Call::join_identity_as_key {
                    auth_id: rejected_auth_id
                }),
                bob_account.clone()
            ),
            Err(invalid_auth)
        );
        // The target can still choose to pay for the removal.
        assert_eq!(
            TxFeeHandler::call_payment_info(
                &remove_call(rejected_auth_id, false),
                bob_account.clone()
            ),
            Ok(CallPaymentInfo::new(bob_account.clone(), None, None))
        );

        // A stale authorization still can't be accepted.
        assert_noop!(
            Identity::join_identity_as_key(bob.clone(), rejected_auth_id),
            IdError::InvalidAuthorization
        );

        // Both the target and the issuer can remove it, releasing the issuer's slot.
        assert_ok!(Identity::remove_authorization(
            bob.clone(),
            bob_signatory.clone(),
            rejected_auth_id,
            false,
        ));
        assert_eq!(
            identity::NumberOfGivenAuths::<TestStorage>::get(charlie_did),
            1
        );
        assert_ok!(Identity::remove_authorization(
            charlie.clone(),
            bob_signatory.clone(),
            revoked_auth_id,
            false,
        ));
        assert_eq!(
            identity::NumberOfGivenAuths::<TestStorage>::get(charlie_did),
            0
        );
        assert!(
            identity::Authorizations::<TestStorage>::iter_prefix(&bob_signatory)
                .next()
                .is_none()
        );

        // An authorization issued after the removal is unaffected: the issuer pays.
        let new_auth_id = invite_bob();
        assert_eq!(
            TxFeeHandler::call_payment_info(&remove_call(new_auth_id, true), bob_account.clone()),
            Ok(CallPaymentInfo::new(
                Sr25519Keyring::Charlie.to_account_id(),
                Some(new_auth_id),
                Some(bob_signatory.clone())
            ))
        );
    });
}
