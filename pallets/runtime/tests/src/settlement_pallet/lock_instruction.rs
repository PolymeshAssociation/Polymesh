use frame_support::{assert_ok, assert_noop};
use sp_keyring::AccountKeyring;
use sp_std::collections::btree_set::BTreeSet;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioAssetBalances;
use pallet_settlement::{Error, Event, InstructionStatuses, LockedTimestamp};
use polymesh_primitives::settlement::{InstructionId, InstructionStatus, Leg, SettlementType};
use polymesh_primitives::traits::PortfolioSubTrait;
use polymesh_primitives::TrustedFor;
use polymesh_primitives::{Claim, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber};
use polymesh_primitives::{ClaimType, Condition, ConditionType, CountryCode, Scope, TrustedIssuer};
use polymesh_runtime_common::Weight;

use super::setup::{add_and_affirm_simple_instruction, create_and_issue_sample_asset_with_venue};
use crate::storage::{root, EventTest, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type PortfolioError = pallet_portfolio::Error<TestStorage>;

#[test]
fn invalid_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio,
            receiver: bob_default_portfolio,
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
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio,
            receiver: bob_default_portfolio,
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
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            Error::<TestStorage>::UnexpectedAffirmationStatus
        );
    });
}

#[test]
fn invalid_type() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio,
            receiver: bob_default_portfolio,
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
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio,
            receiver: bob_default_portfolio,
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnComplianceCheck,
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
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

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
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

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
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        Settlement::set_venue_filtering(alice.origin(), asset_id, true).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::UnauthorizedVenue
        );
    });
}

#[test]
fn frozen_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );
        Asset::freeze(alice.origin(), asset_id).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::InstructionWithAFrozenAsset
        );
    });
}

#[test]
fn missing_cdd_claim() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            let bob = User::new(AccountKeyring::Bob);
            let dave = User::new(AccountKeyring::Dave);
            let alice = User::new(AccountKeyring::Alice);

            let _ = add_and_affirm_simple_instruction(
                alice,
                bob,
                dave,
                SettlementType::SettleOnComplianceCheck,
            );

            let cdd_1_id = Identity::get_identity(&AccountKeyring::Eve.to_account_id()).unwrap();
            Identity::invalidate_cdd_claims(root(), cdd_1_id, Timestamp::get(), None).unwrap();

            assert_noop!(
                Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
                Error::<TestStorage>::InstructionWithAnInvalidCDDClaim
            );
        });
}

#[test]
fn receivers_missing_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let inst_id = InstructionId(0);
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob_portfolio = PortfolioId::new(bob.did, PortfolioKind::User(PortfolioNumber(1)));
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        Portfolio::create_portfolio(bob.origin(), PortfolioName(b"MyPid".into())).unwrap();

        let legs = vec![Leg::Fungible {
            sender: alice_default_portfolio,
            receiver: bob_portfolio,
            asset_id,
            amount: 1_000,
        }];

        Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_id,
            SettlementType::SettleOnComplianceCheck,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([dave.did]).try_into().unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction(
            bob.origin(),
            inst_id,
            BTreeSet::from([bob_portfolio]).try_into().unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction(
            alice.origin(),
            inst_id,
            BTreeSet::from([alice_default_portfolio])
                .try_into()
                .unwrap(),
        )
        .unwrap();

        Settlement::affirm_instruction_as_mediator(dave.origin(), inst_id, None).unwrap();

        Portfolio::delete_portfolio(bob.origin(), PortfolioNumber(1)).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), inst_id, Weight::MAX),
            PortfolioError::PortfolioDoesNotExist
        );
    });
}

#[test]
fn receivers_not_compliant() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        ComplianceManager::add_compliance_requirement(
            alice.origin(),
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
            Error::<TestStorage>::IntructionReceiverIsNotCompliant
        );
    });
}

#[test]
fn sender_tokens_are_locked() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        Portfolio::unlock_tokens(&PortfolioId::default_portfolio(alice.did), &asset_id, 1).unwrap();

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            PortfolioError::InsufficientTokensLocked
        );
    });
}

#[test]
fn sender_invalid_portfolio_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        PortfolioAssetBalances::<TestStorage>::insert(
            &PortfolioId::default_portfolio(alice.did),
            &asset_id,
            999,
        );

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            PortfolioError::InsufficientPortfolioBalance
        );
    });
}

#[test]
fn sender_invalid_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        BalanceOf::<TestStorage>::insert(asset_id, alice.did, 999);

        assert_noop!(
            Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX),
            Error::<TestStorage>::SenderHasInsufficientBalance
        );
    });
}

#[test]
fn senders_not_compliant() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        ComplianceManager::add_compliance_requirement(
            alice.origin(),
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
            Error::<TestStorage>::IntructionSenderIsNotCompliant
        );
    });
}

#[test]
fn invalid_weight() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

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

        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

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
