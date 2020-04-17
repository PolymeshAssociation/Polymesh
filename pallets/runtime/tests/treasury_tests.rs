mod common;
use common::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};

use polymesh_runtime_balances as balances;
use polymesh_runtime_treasury::{self as treasury};

use frame_support::{assert_err, assert_ok};
use sp_runtime::DispatchError;
use test_client::AccountKeyring;

pub type Balances = balances::Module<TestStorage>;
pub type Treasury = treasury::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn reimbursement_and_disbursement() {
    ExtBuilder::default()
        .treasury(1_000_000)
        .existential_deposit(10)
        .build()
        .execute_with(reimbursement_and_disbursement_we);
}

fn reimbursement_and_disbursement_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice = register_keyring_account(AccountKeyring::Alice).unwrap();
    let alice_acc = Origin::signed(AccountKeyring::Alice.public());

    // Verify reimburstement from root.
    assert_eq!(Treasury::balance(), 1_000_000);
    assert_ok!(Treasury::reimbursement(root.clone(), 1_000));
    assert_eq!(Treasury::balance(), 1_001_000);

    // Nobody can make reimbursments.
    assert_err!(
        Treasury::reimbursement(alice_acc.clone(), 5_000),
        DispatchError::BadOrigin
    );

    assert_ok!(Treasury::disbursement(root.clone(), alice, 100));
    assert_eq!(Treasury::balance(), 1_000_900);
    assert_eq!(Balances::identity_balance(alice), 100);

    // Alice cannot make a disbursement to herself.
    assert_err!(
        Treasury::disbursement(alice_acc, alice, 500),
        DispatchError::BadOrigin
    );
}
