use super::{
    storage::{get_last_auth_id, make_account_without_cdd, register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok};
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_multisig as multisig;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{InvestorUid, Signatory, TransactionError};
use polymesh_runtime_develop::{fee_details::CddHandler, runtime::Call};
use sp_core::crypto::AccountId32;
use sp_runtime::transaction_validity::InvalidTransaction;
use test_client::AccountKeyring;

type MultiSig = multisig::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn cdd_checks() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Bob.public()])
        .monied(true)
        .build()
        .execute_with(|| {
            // alice does not have cdd
            let (alice_signed, _) =
                make_account_without_cdd(AccountKeyring::Alice.public()).unwrap();
            let alice_key_signatory = Signatory::Account(AccountKeyring::Alice.public());
            let alice_account_signatory = AccountId32::from(AccountKeyring::Alice.public().0);
            let _musig_address =
                MultiSig::get_next_multisig_address(AccountKeyring::Alice.public());

            // charlie has valid cdd
            let charlie_signed = Origin::signed(AccountKeyring::Charlie.public());
            let _ = register_keyring_account(AccountKeyring::Charlie).unwrap();
            let charlie_account_signatory = AccountId32::from(AccountKeyring::Charlie.public().0);

            // register did bypasses cdd checks
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::Identity(identity::Call::register_did(
                        InvestorUid::default(),
                        Default::default()
                    )),
                    &alice_account_signatory
                ),
                Ok(Some(AccountId32::from(AccountKeyring::Alice.public().0)))
            );

            // normal tx without cdd should fail
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &alice_account_signatory
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

            let alice_auth_id = get_last_auth_id(&alice_key_signatory);
            assert_err!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_account_signatory
                ),
                InvalidTransaction::Custom(TransactionError::CddRequired as u8)
            );

            // call to accept being a multisig signer should succeed when authorizer has a valid cdd but signer key does not
            // fee must be paid by multisig creator
            let _musig_address2 =
                MultiSig::get_next_multisig_address(AccountKeyring::Charlie.public());
            assert_ok!(MultiSig::create_multisig(
                charlie_signed.clone(),
                vec![Signatory::Account(AccountKeyring::Alice.public())],
                1,
            ));
            let alice_auth_id = get_last_auth_id(&alice_key_signatory);
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::accept_multisig_signer_as_key(alice_auth_id)),
                    &alice_account_signatory
                ),
                Ok(Some(AccountId32::from(AccountKeyring::Charlie.public().0)))
            );

            // normal tx with cdd should succeed
            assert_eq!(
                CddHandler::get_valid_payer(
                    &Call::MultiSig(multisig::Call::change_sigs_required(1)),
                    &charlie_account_signatory
                ),
                Ok(Some(AccountId32::from(AccountKeyring::Charlie.public().0)))
            );
        });
}
