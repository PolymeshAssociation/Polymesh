use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring;

use polymesh_primitives::settlement::{InstructionId, SettlementType};
use polymesh_primitives::PortfolioId;
use polymesh_runtime_common::Weight;

use super::setup::add_and_affirm_simple_instruction;
use crate::storage::{vec_to_btreeset, User};
use crate::{ExtBuilder, TestStorage};

type Error = pallet_settlement::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;

#[test]
fn withdraw_after_locking() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

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
                vec_to_btreeset(vec![PortfolioId::default_portfolio(bob.did)]),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );

        assert_noop!(
            Settlement::withdraw_affirmation(
                dave.origin(),
                InstructionId(0),
                vec_to_btreeset(vec![PortfolioId::default_portfolio(dave.did)]),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );

        assert_noop!(
            Settlement::withdraw_affirmation(
                alice.origin(),
                InstructionId(0),
                vec_to_btreeset(vec![PortfolioId::default_portfolio(alice.did)]),
            ),
            Error::InvalidInstructionStatusForWithdrawal
        );
    });
}
