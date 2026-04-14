use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_std::collections::btree_set::BTreeSet;

use polymesh_primitives::settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{AssetHolder, PortfolioId};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::settlement_pallet::setup::create_and_issue_sample_asset_with_venue;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type SettlementError = pallet_settlement::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

#[test]
fn unauthorized_venue() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            Default::default(),
            VenueType::Other
        ));

        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Settlement::set_venue_filtering(
            alice.origin(),
            asset_id,
            true
        ));

        assert_ok!(Settlement::allow_venues(
            alice.origin(),
            asset_id,
            vec![VenueId(0)]
        ));

        assert_noop!(
            Settlement::add_instruction(
                bob.origin(),
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![Leg::Fungible {
                    sender: AssetHolder::Portfolio(alice_default_portfolio),
                    receiver: AssetHolder::Portfolio(bob_default_portfolio),
                    asset_id,
                    amount: 100,
                }],
                None,
            ),
            SettlementError::UnauthorizedVenue
        );
    });
}

#[test]
fn settle_after_lock_without_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);

        assert_noop!(
            Settlement::add_instruction(
                bob.origin(),
                None,
                SettlementType::SettleAfterLock,
                None,
                None,
                vec![Leg::Fungible {
                    sender: AssetHolder::Account(alice.acc()),
                    receiver: AssetHolder::Account(bob.acc()),
                    asset_id,
                    amount: 100,
                }],
                None,
            ),
            SettlementError::MissingInstructionMediators
        );

        assert_noop!(
            Settlement::add_instruction_with_mediators(
                bob.origin(),
                None,
                SettlementType::SettleAfterLock,
                None,
                None,
                vec![Leg::Fungible {
                    sender: AssetHolder::Account(alice.acc()),
                    receiver: AssetHolder::Account(bob.acc()),
                    asset_id,
                    amount: 100,
                }],
                None,
                BTreeSet::new().try_into().unwrap(),
            ),
            SettlementError::MissingInstructionMediators
        );
    });
}

#[test]
fn failing_to_schedule_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let target_block = System::block_number() + 1;

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
            receiver: AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did)),
            asset_id: asset_id,
            amount: 1,
        }];

        // Fill the block
        for _ in 0..50 {
            assert_ok!(Settlement::add_instruction(
                alice.origin(),
                venue_id,
                SettlementType::SettleOnBlock(target_block),
                None,
                None,
                legs.clone(),
                None,
            ));
        }
        assert_eq!(
            pallet_scheduler::Agenda::<TestStorage>::get(target_block).len(),
            50
        );

        let (asset_id2, venue_id2) = create_and_issue_sample_asset_with_venue(&charlie);
        assert_noop!(
            Settlement::add_instruction(
                charlie.origin(),
                venue_id2,
                SettlementType::SettleOnBlock(target_block),
                None,
                None,
                vec![Leg::Fungible {
                    sender: AssetHolder::Portfolio(PortfolioId::default_portfolio(charlie.did)),
                    receiver: AssetHolder::Portfolio(PortfolioId::default_portfolio(dave.did)),
                    asset_id: asset_id2,
                    amount: 1,
                }],
                None,
            ),
            SettlementError::FailedToSchedule
        );
    });
}
