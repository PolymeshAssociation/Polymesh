use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_asset::Allowances;

use super::setup::create_and_issue_sample_asset;
use crate::storage::{account_from, EventTest, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type IdentityError = pallet_identity::Error<TestStorage>;
type RuntimeOrigin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type System = frame_system::Pallet<TestStorage>;

/// Basic approve stores allowance and emits Approval event.
#[test]
fn approve_stores_allowance_and_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        System::set_block_number(1);
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            500
        );
        assert_eq!(
            System::events().pop().unwrap().event,
            EventTest::Asset(pallet_asset::Event::Approval {
                owner: alice.acc(),
                spender: bob.acc(),
                asset_id,
                amount: 500,
            })
        );
    });
}

/// Second approve replaces (not sums) previous allowance.
#[test]
fn approve_overwrites_previous_allowance() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 200));
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            200
        );
    });
}

/// Approve to 0 removes storage entry, emits Approval with amount 0.
#[test]
fn approve_zero_removes_entry() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));
        System::set_block_number(1);
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 0));
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            0
        );
        assert!(!Allowances::<TestStorage>::contains_key((
            &alice.acc(),
            &bob.acc(),
            asset_id
        )));
        assert_eq!(
            System::events().pop().unwrap().event,
            EventTest::Asset(pallet_asset::Event::Approval {
                owner: alice.acc(),
                spender: bob.acc(),
                asset_id,
                amount: 0,
            })
        );
    });
}

/// No DID rejects with MissingIdentity.
#[test]
fn approve_no_did_fails_missing_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&bob);
        let no_did_account = account_from(999);

        assert_noop!(
            Asset::approve(
                RuntimeOrigin::signed(no_did_account),
                asset_id,
                bob.acc(),
                500
            ),
            IdentityError::MissingIdentity
        );
    });
}

/// Allowance query for non-existent entry returns 0.
#[test]
fn allowance_query_nonexistent_returns_zero() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            0
        );
    });
}

/// Partial spend decrements storage and emits AllowanceSpent with remaining balance.
#[test]
fn spend_allowance_partial_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 1000));
        System::set_block_number(1);
        assert_ok!(Asset::spend_allowance(
            &alice.acc(),
            &bob.acc(),
            asset_id,
            300
        ));
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            700
        );
        assert_eq!(
            System::events().pop().unwrap().event,
            EventTest::Asset(pallet_asset::Event::AllowanceSpent {
                owner: alice.acc(),
                spender: bob.acc(),
                asset_id,
                amount_spent: 300,
                remaining_allowance: 700,
            })
        );
    });
}

/// Spending the full allowance removes the storage entry and emits remaining=0.
#[test]
fn spend_allowance_depletes_removes_entry_and_emits_event() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));
        System::set_block_number(1);
        assert_ok!(Asset::spend_allowance(
            &alice.acc(),
            &bob.acc(),
            asset_id,
            500
        ));
        assert!(!Allowances::<TestStorage>::contains_key((
            &alice.acc(),
            &bob.acc(),
            asset_id
        )));
        assert_eq!(
            System::events().pop().unwrap().event,
            EventTest::Asset(pallet_asset::Event::AllowanceSpent {
                owner: alice.acc(),
                spender: bob.acc(),
                asset_id,
                amount_spent: 500,
                remaining_allowance: 0,
            })
        );
    });
}

/// Infinite allowance (Balance::MAX) is not decremented but still emits AllowanceSpent.
#[test]
fn spend_allowance_infinite_emits_event_without_decrement() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::approve(
            alice.origin(),
            asset_id,
            bob.acc(),
            polymesh_primitives::Balance::MAX
        ));
        System::set_block_number(1);
        assert_ok!(Asset::spend_allowance(
            &alice.acc(),
            &bob.acc(),
            asset_id,
            1000
        ));
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            polymesh_primitives::Balance::MAX
        );
        assert_eq!(
            System::events().pop().unwrap().event,
            EventTest::Asset(pallet_asset::Event::AllowanceSpent {
                owner: alice.acc(),
                spender: bob.acc(),
                asset_id,
                amount_spent: 1000,
                remaining_allowance: polymesh_primitives::Balance::MAX,
            })
        );
    });
}
