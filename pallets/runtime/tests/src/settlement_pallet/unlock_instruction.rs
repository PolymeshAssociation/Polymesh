use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_settlement::{Error, Event, InstructionStatuses, LockedTimestamp};
use pallet_settlement::{InstructionRelockCount, UnlockedTimestamp};
use polymesh_primitives::settlement::{InstructionId, InstructionStatus, SettlementType};
use polymesh_runtime_common::Weight;

use super::setup::add_and_affirm_simple_instruction;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

#[test]
fn unlock_non_mediator_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        // Bob is not a mediator — should fail
        assert_noop!(
            Settlement::unlock_instruction(bob.origin(), InstructionId(0)),
            Error::<TestStorage>::CallerIsNotAMediator
        );
    });
}

#[test]
fn unlock_not_locked_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        // Instruction is Pending, not LockedForExecution — should fail
        assert_noop!(
            Settlement::unlock_instruction(dave.origin(), InstructionId(0)),
            Error::<TestStorage>::InstructionNotLocked
        );
    });
}

#[test]
fn unlock_success_and_parties_can_reject() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_ok!(Settlement::unlock_instruction(
            dave.origin(),
            InstructionId(0)
        ));

        // Status should be back to Pending
        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::Pending
        );

        // LockedTimestamp should be cleared, UnlockedTimestamp should be set
        assert!(LockedTimestamp::<TestStorage>::get(InstructionId(0)).is_none());
        assert!(UnlockedTimestamp::<TestStorage>::get(InstructionId(0)).is_some());

        // Event should be emitted
        let mut system_events = System::events();
        assert_eq!(
            system_events.pop().unwrap().event,
            crate::storage::EventTest::Settlement(Event::InstructionUnlocked(
                dave.did,
                InstructionId(0)
            ))
        );

        // Parties can reject while in Pending state (the escape path)
        assert_ok!(Settlement::reject_instruction(
            bob.origin(),
            InstructionId(0),
            polymesh_primitives::PortfolioId::default_portfolio(bob.did).into(),
        ));

        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::Rejected(System::block_number())
        );
    });
}

#[test]
fn relock_before_cooldown_fails() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        // Lock, then unlock
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));
        assert_ok!(Settlement::unlock_instruction(
            dave.origin(),
            InstructionId(0)
        ));

        // Immediately try to relock — should fail (cooldown not expired)
        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::RelockCooldownNotExpired
        );
    });
}

#[test]
fn relock_after_cooldown_success() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        // Lock, then unlock
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));
        assert_ok!(Settlement::unlock_instruction(
            dave.origin(),
            InstructionId(0)
        ));

        // Advance past cooldown (test runtime RelockCooldown = 1)
        Timestamp::set_timestamp(Timestamp::get() + 2);

        // Re-affirm as mediator (previous affirmation may have expired)
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            Some(Timestamp::get() + 1),
        ));

        // Relock should now succeed
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::LockedForExecution
        );

        // Relock count should be 1
        assert_eq!(
            InstructionRelockCount::<TestStorage>::get(InstructionId(0)),
            1
        );
    });
}

#[test]
fn relock_exceeds_max_count() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        // Lock then unlock (sets UnlockedTimestamp)
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));
        assert_ok!(Settlement::unlock_instruction(
            dave.origin(),
            InstructionId(0)
        ));

        // Set relock count to MaxRelockCount (3) directly
        InstructionRelockCount::<TestStorage>::insert(InstructionId(0), 3u32);

        // Advance past cooldown so the count check is what fails
        Timestamp::set_timestamp(Timestamp::get() + 2);

        // Re-affirm as mediator (previous affirmation expired)
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            Some(Timestamp::get() + 1),
        ));

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::MaxRelockCountExceeded
        );
    });
}
