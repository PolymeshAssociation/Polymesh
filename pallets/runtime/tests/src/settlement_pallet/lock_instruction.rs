use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_std::collections::btree_set::BTreeSet;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioAssetBalances;
use pallet_settlement::{
    Error, Event, InstructionRelockCount, InstructionStatuses, LockedTimestamp,
};
use polymesh_primitives::settlement::{InstructionId, InstructionStatus, Leg, SettlementType};
use polymesh_primitives::TrustedFor;
use polymesh_primitives::{Claim, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber};
use polymesh_primitives::{ClaimType, Condition, ConditionType, CountryCode, Scope, TrustedIssuer};
use polymesh_runtime_common::Weight;

use super::setup::{add_and_affirm_simple_instruction, create_and_issue_sample_asset_with_venue};
use crate::storage::{default_asset_holder_set, EventTest, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type AssetError = pallet_asset::Error<TestStorage>;
type IdentityError = pallet_identity::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;

#[test]
fn invalid_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio.into(),
            receiver: bob_default_portfolio.into(),
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        assert_noop!(
            Settlement::lock_instruction(alice.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::CallerIsNotAMediator
        );
    });
}

#[test]
fn mediator_has_not_affirmed() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio.into(),
            receiver: bob_default_portfolio.into(),
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleAfterLock,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::NotAllAffirmationsHaveBeenReceived
        );
    });
}

#[test]
fn invalid_type() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio.into(),
            receiver: bob_default_portfolio.into(),
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction_as_mediator(dave.origin(), inst_id, None).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::UnexpectedSettlementType
        );
    });
}

#[test]
fn missing_affirmation() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio.into(),
            receiver: bob_default_portfolio.into(),
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleAfterLock,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction_as_mediator(dave.origin(), inst_id, None).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::NotAllAffirmationsHaveBeenReceived
        );
    });
}

#[test]
fn invalid_inst_status() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        // Force an error
        InstructionStatuses::<TestStorage>::insert(InstructionId(0), InstructionStatus::Unknown);

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::InvalidInstructionStatusForExecution
        );
    });
}

#[test]
fn expired_mediator_affirmation() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Timestamp::set_timestamp(Timestamp::get() + 2);

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::MediatorAffirmationExpired
        );
    });
}

#[test]
fn unauthorized_venue() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::set_venue_filtering(dave.origin(), asset_id, true).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::UnauthorizedVenue
        );
    });
}

#[test]
fn frozen_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);
        Asset::freeze(dave.origin(), asset_id).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn receivers_missing_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_portfolio = PortfolioId::new(bob.did, PortfolioKind::User(PortfolioNumber(1)));
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        Portfolio::create_portfolio(bob.origin(), PortfolioName(b"MyPid".into())).unwrap();

        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio.clone().into(),
            receiver: bob_portfolio.clone().into(),
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleAfterLock,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction(
            alice.origin(),
            inst_id,
            default_asset_holder_set(alice.did),
        )
        .unwrap();

        Settlement::affirm_instruction_as_mediator(dave.origin(), inst_id, None).unwrap();

        Portfolio::delete_portfolio(bob.origin(), PortfolioNumber(1)).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn receivers_not_compliant() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        ComplianceManager::add_compliance_requirement(
            dave.origin(),
            asset_id,
            Default::default(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Asset(asset_id),
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction]),
                }],
            }],
        )
        .unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn sender_tokens_are_locked() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Asset::remove_locked_balance(
            PortfolioId::default_portfolio(alice.did).into(),
            asset_id,
            1,
        )
        .unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            AssetError::InsufficientTokensLocked
        );
    });
}

#[test]
fn sender_invalid_portfolio_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        PortfolioAssetBalances::<TestStorage>::insert(
            &PortfolioId::default_portfolio(alice.did),
            &asset_id,
            999,
        );

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn sender_invalid_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        BalanceOf::<TestStorage>::insert(asset_id, alice.did, 999);

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn senders_not_compliant() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        ComplianceManager::add_compliance_requirement(
            dave.origin(),
            asset_id,
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Asset(asset_id),
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction]),
                }],
            }],
            Default::default(),
        )
        .unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn invalid_weight() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_noop!(
            Settlement::lock_instruction(
                dave.origin(),
                InstructionId(0),
                Settlement::lock_instruction_minimum_weight()
            ),
            Error::<TestStorage>::WeightLimitExceeded
        );
    });
}

#[test]
fn success() {
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

        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::LockedForExecution
        );

        assert!(LockedTimestamp::<TestStorage>::get(InstructionId(0)).is_some());

        let mut system_events = System::events();
        assert_eq!(
            system_events.pop().unwrap().event,
            EventTest::Settlement(Event::InstructionLocked(dave.did, InstructionId(0)))
        );
    });
}

#[test]
fn lock_while_already_locked_fails() {
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

        // Second lock while already locked must fail (lock period + cooldown not elapsed)
        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::InstructionAlreadyLocked
        );
    });
}

#[test]
fn relock_without_unlock_after_expiry_and_cooldown() {
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

        // Advance past MaximumLockPeriod (2) + RelockCooldown (1) = 3
        Timestamp::set_timestamp(Timestamp::get() + 4);

        // Re-affirm as mediator (previous affirmation expired)
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            Some(Timestamp::get() + 1),
        ));

        // Re-lock without explicit unlock should succeed
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            Weight::MAX
        ));

        assert_eq!(
            InstructionStatuses::<TestStorage>::get(InstructionId(0)),
            InstructionStatus::LockedForExecution
        );
        assert_eq!(
            InstructionRelockCount::<TestStorage>::get(InstructionId(0)),
            1
        );
    });
}
