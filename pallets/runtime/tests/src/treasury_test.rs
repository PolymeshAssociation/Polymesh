use super::{
    exec_noop, exec_ok,
    storage::{make_account_without_cdd, root, TestStorage, User},
    ExtBuilder,
};

use polymesh_primitives::{AccountId, Beneficiary, IdentityId};
use sp_runtime::DispatchError;
use test_client::AccountKeyring;

pub type Balances = pallet_balances::Module<TestStorage>;
pub type Treasury = pallet_treasury::Module<TestStorage>;
type TreasuryError = pallet_treasury::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;

fn beneficiary<Balance>(id: IdentityId, amount: Balance) -> Beneficiary<Balance> {
    Beneficiary { id, amount }
}

#[test]
fn reimbursement_and_disbursement() {
    ExtBuilder::default()
        .balance_factor(10)
        .build()
        .execute_with(reimbursement_and_disbursement_we);
}

fn reimbursement_and_disbursement_we() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let charlie_acc = AccountKeyring::Charlie.to_account_id();
    let (_, charlie_did) = make_account_without_cdd(charlie_acc.clone()).unwrap();

    let total_issuance = Balances::total_issuance();

    // Verify reimbursement.
    assert_eq!(Treasury::balance(), 0);
    exec_ok!(Treasury::reimbursement(alice.origin(), 1_000));
    assert_eq!(Treasury::balance(), 1_000);
    assert_eq!(total_issuance, Balances::total_issuance());

    // Disbursement: Only root can do that.
    exec_noop!(
        Treasury::disbursement(alice.origin(), vec![]),
        DispatchError::BadOrigin
    );

    let beneficiaries = vec![
        // Valid identities.
        beneficiary(alice.did, 100),
        beneficiary(bob.did, 500),
        // no-CDD identitiy.
        beneficiary(charlie_did, 100),
    ];

    // Save balances before disbursement.
    let before_alice_balance = Balances::free_balance(&alice.acc());
    let before_bob_balance = Balances::free_balance(&bob.acc());
    let before_charlie_balance = Balances::free_balance(&charlie_acc);

    // Try disbursement.
    exec_ok!(Treasury::disbursement(root(), beneficiaries.clone()));

    // Check balances after disbursement.
    assert_eq!(Treasury::balance(), 400);
    assert_eq!(
        Balances::free_balance(&alice.acc()),
        before_alice_balance + 100
    );
    assert_eq!(Balances::free_balance(&bob.acc()), before_bob_balance + 500);
    assert_eq!(Balances::free_balance(&charlie_acc), before_charlie_balance);
    assert_eq!(total_issuance, Balances::total_issuance());

    // Alice cannot make a disbursement to herself.
    exec_noop!(
        Treasury::disbursement(alice.origin(), vec![beneficiary(alice.did, 500)]),
        DispatchError::BadOrigin
    );
    assert_eq!(total_issuance, Balances::total_issuance());

    // Repeat disbursement.  This time there is not enough POLYX in the treasury.
    exec_noop!(
        Treasury::disbursement(root(), beneficiaries),
        TreasuryError::InsufficientBalance,
    );
}

#[test]
fn bad_disbursement_did() {
    ExtBuilder::default()
        .balance_factor(10)
        .build()
        .execute_with(bad_disbursement_did_we);
}

fn bad_disbursement_did_we() {
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);
    let default_key = AccountId::default();

    let total_issuance = Balances::total_issuance();
    let treasury_balance = 10_000;

    // Give the Treasury some POLYX.
    assert_eq!(Treasury::balance(), 0);
    exec_ok!(Treasury::reimbursement(alice.origin(), treasury_balance));
    assert_eq!(Treasury::balance(), treasury_balance);
    assert_eq!(total_issuance, Balances::total_issuance());

    let beneficiaries = vec![
        // Valid identities.
        beneficiary(alice.did, 100),
        beneficiary(bob.did, 500),
        // Invalid identities.
        beneficiary(0x00001u128.into(), 100),
        beneficiary(0x00002u128.into(), 200),
        beneficiary(0x00003u128.into(), 300),
    ];

    // Save balances before disbursement.
    let before_alice_balance = Balances::free_balance(&alice.acc());
    let before_bob_balance = Balances::free_balance(&bob.acc());
    let before_default_key_balance = Balances::free_balance(&default_key);

    // Try disbursement.
    exec_noop!(
        Treasury::disbursement(root(), beneficiaries),
        TreasuryError::InvalidIdentity,
    );

    // Check balances after disbursement.
    assert_eq!(Treasury::balance(), treasury_balance);
    assert_eq!(Balances::free_balance(&alice.acc()), before_alice_balance);
    assert_eq!(Balances::free_balance(&bob.acc()), before_bob_balance);
    assert_eq!(
        Balances::free_balance(&default_key),
        before_default_key_balance
    );

    // Make sure total POLYX issuance hasn't changed.
    assert_eq!(total_issuance, Balances::total_issuance());
}
