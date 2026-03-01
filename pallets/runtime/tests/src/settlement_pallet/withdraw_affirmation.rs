use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use polymesh_primitives::settlement::{InstructionId, SettlementType};
use polymesh_runtime_common::Weight;

use super::setup::add_and_affirm_simple_instruction;
use crate::storage::{default_asset_holder_set, User};
use crate::{ExtBuilder, TestStorage};

type Error = pallet_settlement::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;

#[test]
fn withdraw_after_locking() {
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

        assert_noop!(
            Settlement::withdraw_affirmation(
                bob.origin(),
                InstructionId(0),
                default_asset_holder_set(bob.did),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );

        assert_noop!(
            Settlement::withdraw_affirmation(
                dave.origin(),
                InstructionId(0),
                default_asset_holder_set(dave.did),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );

        assert_noop!(
            Settlement::withdraw_affirmation(
                alice.origin(),
                InstructionId(0),
                default_asset_holder_set(alice.did),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );
    });
}
