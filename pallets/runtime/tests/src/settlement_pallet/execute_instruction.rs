use frame_support::{assert_ok, StorageDoubleMap, StorageMap};
use sp_keyring::AccountKeyring;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioLockedAssets;
use pallet_settlement::{
    AffirmsReceived, InstructionAffirmsPending, InstructionDetails, InstructionLegStatus,
    InstructionLegs, InstructionMediatorsAffirmations, InstructionStatuses, OffChainAffirmations,
    UserAffirmations, VenueInstructions,
};
use polymesh_primitives::settlement::{
    AffirmationStatus, Instruction, InstructionId, InstructionStatus, Leg, SettlementType,
};
use polymesh_primitives::{PortfolioId, Ticker};

use crate::settlement_test::{create_token, create_token_and_venue};
use crate::storage::User;
use crate::{next_block, ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Module<TestStorage>;

const TICKER: Ticker = Ticker::new_unchecked([b'A', b'C', b'M', b'E', 0, 0, 0, 0, 0, 0, 0, 0]);
const TICKER2: Ticker = Ticker::new_unchecked([b'A', b'C', b'M', b'Y', 0, 0, 0, 0, 0, 0, 0, 0]);

#[test]
fn execute_instruction_storage_pruning() {
    ExtBuilder::default().build().execute_with(|| {
        let instruction_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let venue_id = create_token_and_venue(TICKER, alice);
        let legs: Vec<Leg> = vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did),
            receiver: PortfolioId::default_portfolio(bob.did),
            ticker: TICKER,
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
            vec![alice_default_portfolio]
        ));
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            vec![bob_default_portfolio]
        ));
        next_block();

        // Asserts all storage have been pruned
        assert_eq!(InstructionAffirmsPending::get(instruction_id), 0);
        assert_eq!(VenueInstructions::iter_prefix_values(venue_id).next(), None);
        assert_eq!(
            InstructionLegs::iter_prefix_values(instruction_id).next(),
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
            OffChainAffirmations::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            AffirmsReceived::iter_prefix_values(instruction_id).next(),
            None
        );
        assert_eq!(
            InstructionMediatorsAffirmations::<TestStorage>::iter_prefix_values(instruction_id)
                .next(),
            None
        );
        assert_eq!(
            UserAffirmations::get(alice_default_portfolio, instruction_id),
            AffirmationStatus::Unknown
        );
        assert_eq!(
            UserAffirmations::get(bob_default_portfolio, instruction_id),
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
        let instruction_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let venue_id = create_token_and_venue(TICKER, alice);
        create_token(TICKER2, alice);
        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did),
                receiver: PortfolioId::default_portfolio(bob.did),
                ticker: TICKER,
                amount: 1_000,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did),
                receiver: PortfolioId::default_portfolio(bob.did),
                ticker: TICKER2,
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
            vec![alice_default_portfolio]
        ));
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            vec![bob_default_portfolio]
        ));
        // Removes TICKER2 balance to force an error
        BalanceOf::insert(TICKER2, alice.did, 0);
        InstructionStatuses::<TestStorage>::insert(instruction_id, InstructionStatus::Failed);
        // Asserts storage has not changed
        assert_eq!(
            PortfolioLockedAssets::get(alice_default_portfolio, TICKER),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::get(alice_default_portfolio, TICKER2),
            1_000
        );
        assert_eq!(
            UserAffirmations::get(alice_default_portfolio, instruction_id),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            UserAffirmations::get(bob_default_portfolio, instruction_id),
            AffirmationStatus::Affirmed
        );
        let all_legs = InstructionLegs::iter_prefix_values(instruction_id).collect::<Vec<_>>();
        assert_eq!(all_legs.len(), 2);
    });
}
