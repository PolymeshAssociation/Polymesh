use frame_support::assert_ok;
use sp_keyring::Sr25519Keyring;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioLockedAssets;
use pallet_settlement::{AffirmsReceived, Error, Event, InstructionAffirmsPending};
use pallet_settlement::{InstructionDetails, InstructionLegStatus, InstructionLegs};
use pallet_settlement::{InstructionMediatorsAffirmations, InstructionStatuses};
use pallet_settlement::{OffChainAffirmations, UserAffirmations, VenueInstructions};
use polymesh_primitives::settlement::{AffirmationStatus, Instruction, InstructionId};
use polymesh_primitives::settlement::{InstructionStatus, Leg, LegId, SettlementType};
use polymesh_primitives::SystematicIssuers::Settlement as SettlementDID;
use polymesh_primitives::{AssetHolder, PortfolioId};

use super::setup::{add_and_affirm_simple_instruction, create_and_issue_sample_asset_with_venue};
use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::storage::{default_asset_holder_set, User};
use crate::{next_block, ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type PortfolioError = pallet_portfolio::Error<TestStorage>;

#[test]
fn storage_pruning() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnAffirmation,
        );
        next_block();

        // Asserts all storage have been pruned
        assert_eq!(InstructionAffirmsPending::<TestStorage>::get(inst_id), 0);
        assert_eq!(VenueInstructions::<TestStorage>::iter().next(), None);
        assert_eq!(
            InstructionLegs::<TestStorage>::iter_prefix_values(inst_id).next(),
            None
        );
        assert_eq!(
            InstructionDetails::<TestStorage>::get(inst_id),
            Instruction::default()
        );
        assert_eq!(
            InstructionLegStatus::<TestStorage>::iter_prefix_values(inst_id).next(),
            None
        );
        assert_eq!(
            OffChainAffirmations::<TestStorage>::iter_prefix_values(inst_id).next(),
            None
        );
        assert_eq!(
            AffirmsReceived::<TestStorage>::iter_prefix_values(inst_id).next(),
            None
        );
        assert_eq!(
            InstructionMediatorsAffirmations::<TestStorage>::iter_prefix_values(inst_id).next(),
            None
        );
        assert_eq!(UserAffirmations::<TestStorage>::iter().next(), None);
        assert_eq!(
            InstructionStatuses::<TestStorage>::get(inst_id),
            InstructionStatus::Success(1)
        );
    });
}

#[test]
fn storage_rollback() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let instruction_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = create_and_issue_sample_asset(&alice);
        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount: 1_000,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
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
            default_asset_holder_set(alice.did),
        ));
        // Removes asset_id2 balance to force an error
        BalanceOf::<TestStorage>::insert(asset_id2, alice.did, 0);
        next_block();
        // Asserts storage has not changed
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(&alice_default_portfolio, asset_id),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(&alice_default_portfolio, asset_id2),
            1_000
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(
                &AssetHolder::from(alice_default_portfolio),
                instruction_id
            ),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            UserAffirmations::<TestStorage>::get(
                &AssetHolder::from(bob_default_portfolio),
                instruction_id
            ),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            InstructionStatuses::<TestStorage>::get(&instruction_id),
            InstructionStatus::Failed
        );
        let all_legs =
            InstructionLegs::<TestStorage>::iter_prefix_values(&instruction_id).collect::<Vec<_>>();
        assert_eq!(all_legs.len(), 2);
        // Asserts the events are being emitted
        let mut system_events = System::events();
        system_events.pop().unwrap();
        assert_eq!(
            system_events.pop().unwrap().event,
            crate::storage::EventTest::Settlement(Event::FailedToExecuteInstruction(
                instruction_id,
                Error::<TestStorage>::FailedAssetTransferringConditions.into()
            ))
        );
        assert_eq!(
            system_events.pop().unwrap().event,
            crate::storage::EventTest::Settlement(Event::LegFailedExecution(
                SettlementDID.as_id(),
                instruction_id,
                LegId(1)
            ))
        );
    });
}
