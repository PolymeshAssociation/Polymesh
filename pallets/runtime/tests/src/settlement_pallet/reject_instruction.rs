use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring;

use pallet_portfolio::PortfolioLockedAssets;
use pallet_settlement::{AffirmsReceived, InstructionAffirmsPending, InstructionLegs};
use pallet_settlement::{Error, InstructionLegStatus, InstructionStatuses};
use pallet_settlement::{InstructionMediatorsAffirmations, UserAffirmations};
use polymesh_primitives::settlement::{AssetCount, InstructionId, InstructionStatus};
use polymesh_primitives::{PortfolioId, PortfolioNumber};
use polymesh_runtime_common::Weight;

use super::lock_instruction::add_and_affirm_simple_instruction;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

#[test]
fn invalid_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let eve = User::new(AccountKeyring::Eve);
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(alice, bob, dave);

        assert_noop!(
            Settlement::reject_instruction_as_mediator(eve.origin(), InstructionId(0), None),
            Error::<TestStorage>::CallerIsNotAParty
        );

        assert_noop!(
            Settlement::reject_instruction(
                bob.origin(),
                InstructionId(0),
                PortfolioId::user_portfolio(bob.did, PortfolioNumber(1))
            ),
            Error::<TestStorage>::CallerIsNotAParty
        );
    });
}

#[test]
fn locked_for_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(alice, bob, dave);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_noop!(
            Settlement::reject_instruction_as_mediator(dave.origin(), InstructionId(0), None),
            Error::<TestStorage>::InvalidInstructionStatusForRejection
        );
    });
}

#[test]
fn invalid_weight() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(alice, bob, dave);

        assert_noop!(
            Settlement::reject_instruction_as_mediator(
                dave.origin(),
                InstructionId(0),
                Some(AssetCount::new(0, 0, 1))
            ),
            Error::<TestStorage>::WeightLimitExceeded
        );
    });
}

#[test]
fn success() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(alice, bob, dave);

        assert_ok!(Settlement::reject_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            Some(AssetCount::new(1, 0, 0))
        ));

        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::Rejected(System::block_number())
        );

        assert_eq!(
            InstructionLegStatus::<TestStorage>::iter_prefix(InstructionId(0)).next(),
            None
        );

        assert_eq!(
            InstructionAffirmsPending::<TestStorage>::get(InstructionId(0)),
            0
        );

        assert_eq!(
            AffirmsReceived::<TestStorage>::iter_prefix(InstructionId(0)).next(),
            None
        );

        assert_eq!(UserAffirmations::<TestStorage>::iter().next(), None);

        assert_eq!(
            InstructionLegs::<TestStorage>::iter_prefix(InstructionId(0)).next(),
            None
        );

        assert_eq!(
            InstructionMediatorsAffirmations::<TestStorage>::iter_prefix(InstructionId(0)).next(),
            None
        );

        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(
                PortfolioId::default_portfolio(alice.did),
                asset_id
            ),
            0
        );
    });
}
