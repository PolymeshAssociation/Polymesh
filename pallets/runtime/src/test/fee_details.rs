use crate::{
    fee_details::CddHandler,
    multisig,
    runtime::Call,
    test::{
        storage::{register_keyring_account, register_keyring_account_without_cdd, TestStorage},
        ExtBuilder,
    },
};

use codec::Encode;
use frame_support::{assert_err, assert_ok, StorageDoubleMap};
use pallet_transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{AccountKey, Signatory, TransactionError};
use polymesh_runtime_identity as identity;
use sp_runtime::transaction_validity::InvalidTransaction;
use std::convert::TryFrom;
use test_client::AccountKeyring;

type MultiSig = multisig::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn cdd_checks() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Bob.public()])
        .build()
        .execute_with(|| {
            // alice does not have cdd
            let alice_did = register_keyring_account_without_cdd(AccountKeyring::Alice).unwrap();
            let alice_key_signatory = Signatory::from(
                AccountKey::try_from(AccountKeyring::Alice.public().encode()).unwrap(),
            );
            let alice_did_signatory = Signatory::from(alice_did);

            // charlie has valid cdd
            let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
            let charlie_key_signatory = Signatory::from(
                AccountKey::try_from(AccountKeyring::Charlie.public().encode()).unwrap(),
            );
            let charlie_did_signatory = Signatory::from(charlie_did);

            // register did bypasses cdd checks
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::Identity(identity::Call::register_did(Default::default())),
                    &alice_did_signatory
                ),
                Ok(Some(alice_did_signatory))
            );

            // normal tx without cdd should fail
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &alice_did_signatory
                ),
                InvalidTransaction::Custom(TransactionError::CDDRequired as u8)
            );

            // call to accept being a multisig signer should fail when invalid auth
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(0)),
                    &alice_key_signatory
                ),
                InvalidTransaction::Custom(TransactionError::InvalidAuthorization as u8)
            );

            // call to accept being a multisig signer should fail when authorizer does not have valid cdd (expired)
            assert_ok!(MultiSig::create_multisig(
                Origin::signed(AccountKeyring::Alice.public()),
                vec![alice_key_signatory],
                1,
            ));
            let alice_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(
                Signatory::from(alice_key_signatory),
            )
            .next()
            .unwrap()
            .auth_id;
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_key_signatory
                ),
                InvalidTransaction::Custom(TransactionError::CDDRequired as u8)
            );

            // call to accept being a multisig signer should succeed when authorizer has a valid cdd but signer key does not
            // fee must be paid by multisig creator
            assert_ok!(MultiSig::create_multisig(
                Origin::signed(AccountKeyring::Charlie.public()),
                vec![alice_key_signatory],
                1,
            ));
            let alice_auth_id = <identity::Authorizations<TestStorage>>::iter_prefix(
                Signatory::from(alice_key_signatory),
            )
            .next()
            .unwrap()
            .auth_id;
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_key_signatory
                ),
                Ok(Some(charlie_did_signatory))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &charlie_key_signatory
                ),
                Ok(Some(charlie_key_signatory))
            );
        });
}
