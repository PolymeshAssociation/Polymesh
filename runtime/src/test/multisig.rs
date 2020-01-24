use crate::{
    balances, identity, multi_sig,
    test::storage::{build_ext, register_keyring_account, TestStorage},
};
use codec::Encode;
use primitives::{Key, Signer};
use sr_io::with_externalities;
use srml_support::{assert_err, assert_ok};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type System = system::Module<TestStorage>;
type Timestamp = timestamp::Module<TestStorage>;
type MultiSig = multi_sig::Module<TestStorage>;

type Origin = <TestStorage as system::Trait>::Origin;

#[test]
fn create_multi_sig() {
    with_externalities(&mut build_ext(), || {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());

        let musig_address = MultiSig::get_next_multi_sig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multi_sig(
            alice.clone(),
            vec![Signer::from(alice_did), Signer::from(bob_did)],
            1,
        ));

        assert_eq!(MultiSig::ms_signs_required(musig_address), 1);

        assert_err!(
            MultiSig::create_multi_sig(alice.clone(), vec![], 10,),
            "No signers provided"
        );

        assert_err!(
            MultiSig::create_multi_sig(
                alice.clone(),
                vec![Signer::from(alice_did), Signer::from(bob_did)],
                0,
            ),
            "Sigs required out of bounds"
        );

        assert_err!(
            MultiSig::create_multi_sig(
                alice.clone(),
                vec![Signer::from(alice_did), Signer::from(bob_did)],
                10,
            ),
            "Sigs required out of bounds"
        );
    });
}

#[test]
fn join_multi_sig() {
    with_externalities(&mut build_ext(), || {
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let alice = Origin::signed(AccountKeyring::Alice.public());
        let bob = Origin::signed(AccountKeyring::Bob.public());
        let bob_signer =
            Signer::from(Key::try_from(AccountKeyring::Bob.public().encode()).unwrap());

        let musig_address = MultiSig::get_next_multi_sig_address(AccountKeyring::Alice.public());

        assert_ok!(MultiSig::create_multi_sig(
            alice.clone(),
            vec![Signer::from(alice_did), bob_signer],
            1,
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signer::from(alice_did))),
            false
        );
        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            false
        );

        assert_ok!(MultiSig::accept_multi_sig_signer_as_identity(
            alice.clone(),
            Identity::last_authorization(Signer::from(alice_did))
        ));

        assert_ok!(MultiSig::accept_multi_sig_signer_as_key(
            bob.clone(),
            Identity::last_authorization(bob_signer)
        ));

        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), Signer::from(alice_did))),
            true
        );
        assert_eq!(
            MultiSig::ms_signers((musig_address.clone(), bob_signer)),
            true
        );
    });
}
