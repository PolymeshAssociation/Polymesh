use super::{
    storage::{root, TestStorage, User},
    ExtBuilder,
};

use frame_support::{assert_noop, assert_ok};
use pallet_balances as balances;
use pallet_identity as identity;
use pallet_treasury::{self as treasury};
use polymesh_common_utilities::Context;
use polymesh_primitives::Beneficiary;
use sp_runtime::DispatchError;
use test_client::AccountKeyring;

pub type Balances = balances::Module<TestStorage>;
pub type Treasury = treasury::Module<TestStorage>;
type Identity = identity::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;

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

    let total_issuance = Balances::total_issuance();

    // Verify reimbursement.
    assert_eq!(Treasury::balance(), 0);
    assert_ok!(Treasury::reimbursement(alice.origin(), 1_000));
    assert_eq!(Treasury::balance(), 1_000);
    assert_eq!(total_issuance, Balances::total_issuance());

    // Disbursement: Only root can do that.
    assert_noop!(
        Treasury::disbursement(alice.origin(), vec![]),
        DispatchError::BadOrigin
    );

    let beneficiary = |u: User, amount| Beneficiary { id: u.did, amount };
    let beneficiaries = vec![beneficiary(alice, 100), beneficiary(bob, 500)];

    // Providing a random DID to Root, In an ideal world root will have a valid DID
    let before_alice_balance = Balances::free_balance(&alice.acc());
    let before_bob_balance = Balances::free_balance(&bob.acc());
    assert_ok!(Treasury::disbursement(root(), beneficiaries));
    Context::set_current_identity::<Identity>(None);
    assert_eq!(Treasury::balance(), 400);
    assert_eq!(
        Balances::free_balance(&alice.acc()),
        before_alice_balance + 100
    );
    assert_eq!(Balances::free_balance(&bob.acc()), before_bob_balance + 500);
    assert_eq!(total_issuance, Balances::total_issuance());

    // Alice cannot make a disbursement to herself.
    assert_noop!(
        Treasury::disbursement(alice.origin(), vec![beneficiary(alice, 500)]),
        DispatchError::BadOrigin
    );
    assert_eq!(total_issuance, Balances::total_issuance());
}
