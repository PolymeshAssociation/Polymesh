mod common;
use common::{storage::TestStorage, ExtBuilder};

use polymesh_runtime_balances as balances;
use polymesh_runtime_treasury::{self as treasury, Error};

use frame_support::{assert_err, assert_ok};
use sp_runtime::DispatchError;
use test_client::AccountKeyring;

pub type Balances = balances::Module<TestStorage>;
pub type Treasury = treasury::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn reimbursement_and_disbursement() {
    ExtBuilder::default()
        .existential_deposit(10)
        .build()
        .execute_with(reimbursement_and_disbursement_we);
}

fn reimbursement_and_disbursement_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let treasury = Treasury::treasury_account();

    assert_eq!(Balances::free_balance(treasury), 0);
    assert_eq!(Treasury::balance(), 0);

    assert_ok!(Treasury::reimbursement(root.clone(), 1_000));
    assert_eq!(Treasury::balance(), 1_000);

    assert_err!(
        Treasury::reimbursement(Origin::signed(treasury), 5_000),
        DispatchError::BadOrigin
    );

    let alice = AccountKeyring::Alice.public();
    let alice_signed = Origin::signed(alice.clone());

    assert_ok!(Treasury::disbursement(root.clone(), alice.clone(), 100));
    assert_eq!(Treasury::balance(), 900);
    assert_eq!(Balances::free_balance(alice.clone()), 100);

    assert_err!(
        Treasury::disbursement(alice_signed, alice.clone(), 500),
        DispatchError::BadOrigin
    );
}
