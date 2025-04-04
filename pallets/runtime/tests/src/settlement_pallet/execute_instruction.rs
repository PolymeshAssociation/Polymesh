use frame_support::assert_ok;
use sp_keyring::AccountKeyring;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioLockedAssets;
use pallet_settlement::{
    AffirmsReceived, Error, Event, InstructionAffirmsPending, InstructionDetails,
    InstructionLegStatus, InstructionLegs, InstructionMediatorsAffirmations, InstructionStatuses,
    OffChainAffirmations, UserAffirmations, VenueInstructions,
};
use polymesh_primitives::settlement::{
    AffirmationStatus, Instruction, InstructionId, InstructionStatus, Leg, SettlementType,
};
use polymesh_primitives::PortfolioId;

use super::setup::create_and_issue_sample_asset_with_venue;
use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::storage::{default_portfolio_btreeset, User};
use crate::{next_block, ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

#[test]
fn execute_instruction_storage_pruning() {
    ExtBuilder::default().build().execute_with(|| {
        let instruction_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs: Vec<Leg> = vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did),
            receiver: PortfolioId::default_portfolio(bob.did),
            asset_id,
            amount: 1_000,
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            default_portfolio_btreeset(alice.did),
        ));
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            default_portfolio_btreeset(bob.did),
        ));
        next_block();

        // Asserts all storage have been pruned
        assert_eq!(
            InstructionAffirmsPending::<TestStorage>::get(instruction_id),
            0
        );
        assert_eq!(
            VenueInstructions::<TestStorage>::iter_prefix_values(venue_id.unwrap()).next(),
            None
        );
        assert_eq!(
            InstructionLegs::<TestStorage>::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            InstructionDetails::<TestStorage>::get(instruction_id),
            Instruction::default()
        );
        assert_eq!(
            InstructionLegStatus::<TestStorage>::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            OffChainAffirmations::<TestStorage>::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            AffirmsReceived::<TestStorage>::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            InstructionMediatorsAffirmations::<TestStorage>::iter_prefix_values(instruction_id)
                .next(),
            None
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(alice_default_portfolio, instruction_id),
            AffirmationStatus::Unknown
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(bob_default_portfolio, instruction_id),
            AffirmationStatus::Unknown
        );
        assert_eq!(
            InstructionStatuses::<TestStorage>::get(instruction_id),
            InstructionStatus::Success(1)
        );
    });
}

#[test]
fn execute_instruction_storage_rollback() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let instruction_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = create_and_issue_sample_asset(&alice);
        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did),
                receiver: PortfolioId::default_portfolio(bob.did),
                asset_id,
                amount: 1_000,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did),
                receiver: PortfolioId::default_portfolio(bob.did),
                asset_id: asset_id2,
                amount: 1_000,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            default_portfolio_btreeset(alice.did),
        ));
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            default_portfolio_btreeset(bob.did),
        ));
        // Removes asset_id2 balance to force an error
        BalanceOf::<TestStorage>::insert(asset_id2, alice.did, 0);
        next_block();
        // Asserts storage has not changed
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(alice_default_portfolio, asset_id),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(alice_default_portfolio, asset_id2),
            1_000
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(alice_default_portfolio, instruction_id),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(bob_default_portfolio, instruction_id),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            InstructionStatuses::<TestStorage>::get(instruction_id),
            InstructionStatus::Failed
        );
        let all_legs =
            InstructionLegs::<TestStorage>::iter_prefix_values(instruction_id).collect::<Vec<_>>();
        assert_eq!(all_legs.len(), 2);
        // Asserts the events are being emitted
        let mut system_events = System::events();
        system_events.pop().unwrap();
        assert_eq!(
            system_events.pop().unwrap().event,
            crate::storage::EventTest::Settlement(Event::FailedToExecuteInstruction(
                instruction_id,
                Error::<TestStorage>::SenderHasInsufficientBalance.into()
            ))
        );
    });
}
