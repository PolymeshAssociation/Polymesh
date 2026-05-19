use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_portfolio::PortfolioLockedAssets;
use pallet_settlement::{AffirmsReceived, InstructionAffirmsPending, InstructionLegs};
use pallet_settlement::{Error, InstructionLegStatus, InstructionStatuses, LockedTimestamp};
use pallet_settlement::{InstructionMediatorsAffirmations, UserAffirmations};
use polymesh_primitives::settlement::{AssetCount, InstructionId};
use polymesh_primitives::settlement::{InstructionStatus, SettlementType};
use polymesh_primitives::{PortfolioId, PortfolioNumber};
use polymesh_runtime_common::Weight;

use super::setup::add_and_affirm_simple_instruction;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

#[test]
fn invalid_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let eve = User::new(Sr25519Keyring::Eve);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_noop!(
            Settlement::reject_instruction_as_mediator(eve.origin(), InstructionId(0), None),
            Error::<TestStorage>::CallerIsNotAParty
        );

        assert_noop!(
            Settlement::reject_instruction(
                bob.origin(),
                InstructionId(0),
                PortfolioId::user_portfolio(bob.did, PortfolioNumber(1)).into()
            ),
            Error::<TestStorage>::CallerIsNotAParty
        );
    });
}

#[test]
fn invalid_caller_locked_for_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_noop!(
            Settlement::reject_instruction_as_mediator(bob.origin(), InstructionId(0), None),
            Error::<TestStorage>::CallerIsNotAMediator
        );

        assert_noop!(
            Settlement::reject_instruction_with_count(
                bob.origin(),
                InstructionId(0),
                PortfolioId::default_portfolio(bob.did).into(),
                Some(AssetCount::new(1, 1, 0))
            ),
            Error::<TestStorage>::CallerIsNotAMediator
        );
    });
}

#[test]
fn invalid_weight() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_noop!(
            Settlement::reject_instruction_as_mediator(
                dave.origin(),
                InstructionId(0),
                Some(AssetCount::new(0, 0, 1))
            ),
            Error::<TestStorage>::NumberOfTransferredNFTsUnderestimated
        );
    });
}

#[test]
fn success() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_ok!(Settlement::reject_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            Some(AssetCount::new(1, 1, 0))
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

        assert!(LockedTimestamp::<TestStorage>::get(InstructionId(0)).is_none());
    });
}

#[test]
fn success_expired_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        Timestamp::set_timestamp(Timestamp::get() + 3);

        assert_ok!(Settlement::reject_instruction_with_count(
            bob.origin(),
            InstructionId(0),
            PortfolioId::default_portfolio(bob.did).into(),
            Some(AssetCount::new(1, 1, 0))
        ));
    });
}
