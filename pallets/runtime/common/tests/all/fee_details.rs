use super::{
    ext_builder::PROTOCOL_OP_BASE_FEE,
    storage::{make_account, make_account_without_cdd, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use pallet_transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{Signatory, TransactionError};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call};
use sp_core::crypto::AccountId32;
use sp_runtime::transaction_validity::InvalidTransaction;
use test_client::AccountKeyring;

type MultiSig = multisig::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;

#[test]
fn cdd_checks() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Bob.public()])
        .monied(true)
        .build()
        .execute_with(|| {
            // alice does not have cdd
            let (alice_signed, alice_did) =
                make_account_without_cdd(AccountKeyring::Alice.public()).unwrap();
            let alice_key_signatory = Signatory::Account(AccountKeyring::Alice.public());
            let alice_account_signatory =
                Signatory::Account(AccountId32::from(AccountKeyring::Alice.public().0));
            let alice_did_signatory = Signatory::from(alice_did);
            let musig_address = MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

            // charlie has valid cdd
            let (charlie_signed, charlie_did) =
                make_account(AccountKeyring::Charlie.public()).unwrap();
            let charlie_account_signatory =
                Signatory::Account(AccountId32::from(AccountKeyring::Charlie.public().0));
            let charlie_did_signatory = Signatory::from(charlie_did);

            assert_ok!(Balances::top_up_identity_balance(
                alice_signed.clone(),
                charlie_did,
                PROTOCOL_OP_BASE_FEE * 2
            ));

            // register did bypasses cdd checks
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::Identity(identity::Call::register_did(Default::default())),
                    &alice_did_signatory
                ),
                Ok(Some(alice_did_signatory.clone()))
            );

            // normal tx without cdd should fail
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &alice_did_signatory
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to accept being a multisig signer should fail when invalid auth
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(0)),
                    &alice_account_signatory
                ),
                InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8)
            );

            // call to accept being a multisig signer should fail when authorizer does not have a valid cdd (expired)
            assert_ok!(MultiSig::create_multisig(
                alice_signed.clone(),
                vec![alice_key_signatory],
                1,
            ));
            assert_ok!(MultiSig::make_multisig_signer(
                alice_signed.clone(),
                musig_address.clone(),
            ));
            let alice_auth_id =
                <identity::Authorizations<TestStorage>>::iter_prefix_values(alice_key_signatory)
                    .next()
                    .unwrap()
                    .auth_id;
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_account_signatory
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to accept being a multisig signer should succeed when authorizer has a valid cdd but signer key does not
            // fee must be paid by multisig creator
            let musig_address2 =
                MultiSig::get_next_multisig_address(AccountKeyring::Charlie.public());
            assert_ok!(MultiSig::create_multisig(
                charlie_signed.clone(),
                vec![Signatory::Account(AccountKeyring::Alice.public())],
                1,
            ));
            let alice_auth_id =
                <identity::Authorizations<TestStorage>>::iter_prefix_values(alice_key_signatory)
                    .next()
                    .unwrap()
                    .auth_id;

            assert_ok!(MultiSig::make_multisig_signer(
                charlie_signed.clone(),
                musig_address2.clone(),
            ));

            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_account_signatory
                ),
                Ok(Some(charlie_did_signatory.clone()))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &charlie_account_signatory
                ),
                Ok(Some(charlie_account_signatory.clone()))
            );

            // tx to set did as fee payer should charge fee to did
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::Balances(balances::Call::change_charge_did_flag(true)),
                    &charlie_account_signatory
                ),
                Ok(Some(charlie_did_signatory))
            );
        });
}
