use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;

use codec::Encode;
use frame_support::dispatch::DispatchErrorWithPostInfo;
use frame_support::{
    assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop, traits::TryCollect,
    BoundedBTreeSet,
};
use frame_system::pallet_prelude::BlockNumberFor;
use rand::prelude::*;
use sp_runtime::{AccountId32, AnySignature};
use sp_std::collections::btree_set::BTreeSet;

use pallet_asset::BalanceOf;
use pallet_nft::NumberOfNFTs;
use pallet_portfolio::{
    NextPortfolioNumber, PortfolioLockedAssets, PortfolioLockedNFT, PortfolioNFT,
};
use pallet_scheduler as scheduler;
use pallet_settlement::{
    AffirmsReceived, Details, Event, InstructionAffirmsPending, InstructionCounter,
    InstructionDetails, InstructionLegStatus, InstructionLegs, InstructionMediatorsAffirmations,
    InstructionMemos, InstructionStatuses, NumberOfVenueSigners, OffChainAffirmations,
    UserAffirmations, UserVenues, VenueCounter, VenueInfo, VenueInstructions, VenueSigners,
};
use polymesh_primitives::asset::{AssetId, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataValue,
};
use polymesh_primitives::checked_inc::CheckedInc;
use polymesh_primitives::constants::currency::ONE_UNIT;
use polymesh_primitives::crypto::{ChainScopedMessage, SETTLEMENT_RECEIPT_LABEL};
use polymesh_primitives::settlement::{
    AffirmationCount, AffirmationRequirement, AffirmationStatus, AssetCount, Instruction,
    InstructionId, InstructionStatus, Leg, LegId, LegStatus, MediatorAffirmationStatus, Receipt,
    ReceiptDetails, SettlementType, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{
    AccountId, AssetHolder, AssetHolderKind, AuthorizationData, Balance, Claim, ClaimType,
    Condition, ConditionType, CountryCode, Fund, FundDescription, IdentityId, Memo,
    NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs, PortfolioId, PortfolioKind,
    PortfolioName, PortfolioNumber, Scope, Signatory, Ticker, TrustedFor, TrustedIssuer,
    WeightMeter,
};
use sp_keyring::Sr25519Keyring;

use super::asset_pallet::setup::{create_and_issue_sample_asset, ISSUE_AMOUNT};
use super::asset_test::max_len_bytes;
use super::nft::{create_nft_collection, mint_nft};
use super::settlement_pallet::setup::create_and_issue_sample_asset_with_venue;
use polymesh_primitives::traits::AffirmationFnTrait;

use super::storage::{
    default_asset_holder_set, make_account_with_balance, root, user_asset_holder_set,
    vec_to_btreeset, TestStorage, User,
};
use super::{next_block, ExtBuilder};

type Identity = pallet_identity::Pallet<TestStorage>;
type Balances = pallet_balances::Pallet<TestStorage>;
type Asset = pallet_asset::Pallet<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type Moment = <TestStorage as pallet_timestamp::Config>::Moment;
type BlockNumber = BlockNumberFor<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Error = pallet_settlement::Error<TestStorage>;
type Scheduler = pallet_scheduler::Pallet<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;

macro_rules! assert_add_claim {
    ($signer:expr, $target:expr, $claim:expr) => {
        assert_ok!(Identity::add_claim($signer, $target, $claim, None,));
    };
}

macro_rules! assert_affirm_instruction {
    ($signer:expr, $instruction_id:expr, $did:expr) => {
        assert_ok!(Settlement::affirm_instruction(
            $signer,
            $instruction_id,
            default_asset_holder_set($did),
        ));
    };
}

struct UserWithBalance {
    user: User,
    init_balances: Vec<(AssetId, Balance)>,
}

impl UserWithBalance {
    fn new(user: User, assets: &[AssetId]) -> Self {
        Self {
            init_balances: assets
                .iter()
                .map(|asset_id| (*asset_id, BalanceOf::<TestStorage>::get(asset_id, user.did)))
                .collect(),
            user,
        }
    }

    fn refresh_init_balances(&mut self) {
        for (asset_id, balance) in &mut self.init_balances {
            *balance = BalanceOf::<TestStorage>::get(asset_id, self.user.did);
        }
    }

    #[track_caller]
    fn init_balance(&self, asset_id: &AssetId) -> Balance {
        self.init_balances
            .iter()
            .find(|bs| bs.0 == *asset_id)
            .unwrap()
            .1
    }

    #[track_caller]
    fn assert_all_balances_unchanged(&self) {
        for (t, balance) in &self.init_balances {
            assert_balance(t, &self.user, *balance);
        }
    }

    #[track_caller]
    fn assert_balance_unchanged(&self, asset_id: &AssetId) {
        assert_balance(asset_id, &self.user, self.init_balance(asset_id));
    }

    #[track_caller]
    fn assert_balance_increased(&self, asset_id: &AssetId, amount: Balance) {
        assert_balance(asset_id, &self.user, self.init_balance(asset_id) + amount);
    }

    #[track_caller]
    fn assert_balance_decreased(&self, asset_id: &AssetId, amount: Balance) {
        assert_balance(asset_id, &self.user, self.init_balance(asset_id) - amount);
    }

    #[track_caller]
    fn assert_portfolio_bal(&self, num: PortfolioNumber, balance: Balance, asset_id: &AssetId) {
        assert_eq!(
            Asset::get_holders_balance(
                &PortfolioId::new(self.user.did, PortfolioKind::User(num)).into(),
                &asset_id
            ),
            balance,
        );
    }

    #[track_caller]
    fn assert_default_portfolio_bal(&self, balance: Balance, asset_id: &AssetId) {
        assert_eq!(
            Asset::get_holders_balance(
                &PortfolioId::new(self.user.did, PortfolioKind::Default).into(),
                &asset_id
            ),
            balance,
        );
    }

    #[track_caller]
    fn assert_default_portfolio_bal_unchanged(&self, asset_id: &AssetId) {
        self.assert_default_portfolio_bal(self.init_balance(asset_id), asset_id);
    }

    #[track_caller]
    fn assert_default_portfolio_bal_decreased(&self, amount: Balance, asset_id: &AssetId) {
        self.assert_default_portfolio_bal(self.init_balance(asset_id) - amount, asset_id);
    }

    #[track_caller]
    fn assert_default_portfolio_bal_increased(&self, amount: Balance, asset_id: &AssetId) {
        self.assert_default_portfolio_bal(self.init_balance(asset_id) + amount, asset_id);
    }
}

impl Deref for UserWithBalance {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

pub fn set_current_block_number(block: u32) {
    System::set_block_number(block);
}

#[test]
fn venue_details_length_limited() {
    ExtBuilder::default().build().execute_with(|| {
        let actor = User::new(Sr25519Keyring::Alice);
        let id = VenueCounter::<TestStorage>::get();
        let create =
            |d| Settlement::create_venue(actor.origin(), d, BTreeSet::new(), VenueType::Exchange);
        let update = |d| Settlement::update_venue_details(actor.origin(), id, d);
        assert_too_long!(create(max_len_bytes(1)));
        assert_ok!(create(max_len_bytes(0)));
        assert_too_long!(update(max_len_bytes(1)));
        assert_ok!(update(max_len_bytes(0)));
    });
}

fn venue_instructions(id: VenueId) -> Vec<InstructionId> {
    VenueInstructions::<TestStorage>::iter_prefix(id)
        .map(|(i, _)| i)
        .collect()
}

fn user_venues(did: IdentityId) -> Vec<VenueId> {
    let mut venues = UserVenues::<TestStorage>::iter_prefix(did)
        .map(|(i, _)| i)
        .collect::<Vec<_>>();
    venues.sort();
    venues
}

#[test]
fn venue_registration() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let venue_counter = VenueCounter::<TestStorage>::get();
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([
                Sr25519Keyring::Alice.to_account_id(),
                Sr25519Keyring::Bob.to_account_id()
            ]),
            VenueType::Exchange
        ));
        let venue_info = VenueInfo::<TestStorage>::get(venue_counter).unwrap();
        assert_eq!(
            VenueCounter::<TestStorage>::get(),
            venue_counter.checked_inc().unwrap()
        );
        assert_eq!(user_venues(alice.did), [venue_counter]);
        assert_eq!(venue_info.creator, alice.did);
        assert_eq!(venue_instructions(venue_counter).len(), 0);
        assert_eq!(
            Details::<TestStorage>::get(venue_counter),
            VenueDetails::default()
        );
        assert_eq!(venue_info.venue_type, VenueType::Exchange);
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, alice.acc()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, Sr25519Keyring::Bob.to_account_id()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(
                venue_counter,
                Sr25519Keyring::Charlie.to_account_id()
            ),
            false
        );

        // Creating a second venue
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc(), Sr25519Keyring::Bob.to_account_id()]),
            VenueType::Exchange
        ));
        assert_eq!(
            user_venues(alice.did),
            [venue_counter, venue_counter.checked_inc().unwrap()]
        );

        // Editing venue details
        assert_ok!(Settlement::update_venue_details(
            alice.origin(),
            venue_counter,
            [0x01].into(),
        ));
        let venue_info = VenueInfo::<TestStorage>::get(venue_counter).unwrap();
        assert_eq!(venue_info.creator, alice.did);
        assert_eq!(venue_instructions(venue_counter).len(), 0);
        assert_eq!(Details::<TestStorage>::get(venue_counter), [0x01].into());
        assert_eq!(venue_info.venue_type, VenueType::Exchange);
    });
}

fn test_with_did_registrar(test: impl FnOnce(AccountId)) {
    let registrar = Sr25519Keyring::Eve.to_account_id();
    ExtBuilder::default()
        .did_registrars(vec![registrar.clone()])
        .build()
        .execute_with(|| test(registrar));
}

#[test]
fn basic_settlement() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);

        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount
            }],
            None,
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        set_current_block_number(5);
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        // Advances the block no. to execute the instruction.
        next_block();
        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
    });
}

#[test]
fn create_and_affirm_instruction() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let add_and_affirm_tx = |affirm_from_portfolio| {
            Settlement::add_and_affirm_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount,
                }],
                affirm_from_portfolio,
                None,
            )
        };

        // If affirmation fails, the instruction should be rolled back.
        // i.e. this tx should be a no-op.
        assert_noop!(
            add_and_affirm_tx(user_asset_holder_set(alice.did, 1u64.into())),
            Error::UnexpectedAffirmationStatus
        );

        assert_ok!(add_and_affirm_tx(default_asset_holder_set(alice.did)));

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);

        // Advances the block no.
        next_block();
        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
    });
}

#[test]
fn overdraft_failure() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = ISSUE_AMOUNT + 1;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount
            }],
            None,
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        assert_noop!(
            Settlement::affirm_instruction(
                alice.origin(),
                instruction_id,
                default_asset_holder_set(alice.did),
            ),
            AssetError::InsufficientBalance
        );
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
    });
}

#[test]
fn token_swap() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = create_and_issue_sample_asset(&bob);

        let mut alice = UserWithBalance::new(alice, &[asset_id, asset_id2]);
        let mut bob = UserWithBalance::new(bob, &[asset_id, asset_id2]);

        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(bob.did).into(),
                receiver: PortfolioId::default_portfolio(alice.did).into(),
                asset_id: asset_id2,
                amount,
            },
        ];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
        ));

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                InstructionLegs::<TestStorage>::get(&instruction_id, &LegId(i as u64)),
                legs[i].clone().into()
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            settlement_type: SettlementType::SettleOnAffirmation,
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_instruction_status(instruction_id, InstructionStatus::Pending);
        assert_instruction_details(instruction_id, instruction_details);

        assert_affirms_pending(instruction_id, 2);
        assert_eq!(
            venue_instructions(venue_counter.unwrap()),
            vec![instruction_id]
        );

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);
        assert_affirms_pending(instruction_id, 1);

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        assert_locked_assets(&asset_id, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        set_current_block_number(500);

        assert_affirm_instruction!(bob.origin(), instruction_id, bob.did);

        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&asset_id, &alice, 0);
        alice.assert_balance_decreased(&asset_id, amount);
        alice.assert_balance_increased(&asset_id2, amount);
        bob.assert_balance_increased(&asset_id, amount);
        bob.assert_balance_decreased(&asset_id2, amount);
    });
}

#[test]
fn settle_on_block() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = create_and_issue_sample_asset(&bob);

        let mut alice = UserWithBalance::new(alice, &[asset_id, asset_id2]);
        let mut bob = UserWithBalance::new(bob, &[asset_id, asset_id2]);

        let instruction_id = InstructionCounter::<TestStorage>::get();
        let block_number = System::block_number() + 1;
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(bob.did).into(),
                receiver: PortfolioId::default_portfolio(alice.did).into(),
                asset_id: asset_id2,
                amount,
            },
        ];

        assert_eq!(0, scheduler::Agenda::<TestStorage>::get(block_number).len());
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));
        assert_eq!(1, scheduler::Agenda::<TestStorage>::get(block_number).len());

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                InstructionLegs::<TestStorage>::get(&instruction_id, &LegId(i as u64)),
                legs[i].clone().into()
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            settlement_type: SettlementType::SettleOnBlock(block_number),
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_instruction_status(instruction_id, InstructionStatus::Pending);
        assert_eq!(
            InstructionDetails::<TestStorage>::get(instruction_id),
            instruction_details
        );

        assert_affirms_pending(instruction_id, 2);
        assert_eq!(
            venue_instructions(venue_counter.unwrap()),
            vec![instruction_id]
        );

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&asset_id, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction!(bob.origin(), instruction_id, bob.did);

        assert_affirms_pending(instruction_id, 0);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Affirmed);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::ExecutionPending);
        assert_locked_assets(&asset_id, &alice, amount);
        assert_locked_assets(&asset_id2, &bob, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Instruction should've settled
        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&asset_id, &alice, 0);
        assert_locked_assets(&asset_id, &bob, 0);

        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
        alice.assert_balance_increased(&asset_id2, amount);
        bob.assert_balance_decreased(&asset_id2, amount);
    });
}

#[test]
fn failed_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let dave: User = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = create_and_issue_sample_asset(&bob);

        let mut alice = UserWithBalance::new(alice, &[asset_id, asset_id2]);
        let mut bob = UserWithBalance::new(bob, &[asset_id, asset_id2]);

        let instruction_id = InstructionCounter::<TestStorage>::get();
        assert_ok!(ComplianceManager::reset_asset_compliance(
            Origin::signed(Sr25519Keyring::Bob.to_account_id()),
            asset_id2,
        ));
        assert_ok!(ComplianceManager::add_compliance_requirement(
            bob.origin(),
            asset_id2,
            Default::default(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(alice.did)
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction])
                }]
            }],
        ));
        let block_number = System::block_number() + 1;
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount,
            },
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(bob.did).into(),
                receiver: PortfolioId::default_portfolio(alice.did).into(),
                asset_id: asset_id2,
                amount,
            },
        ];

        assert_eq!(0, scheduler::Agenda::<TestStorage>::get(block_number).len());
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));
        assert_eq!(1, scheduler::Agenda::<TestStorage>::get(block_number).len());

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                InstructionLegs::<TestStorage>::get(&instruction_id, &LegId(i as u64)),
                legs[i].clone().into()
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            settlement_type: SettlementType::SettleOnBlock(block_number),
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_instruction_status(instruction_id, InstructionStatus::Pending);
        assert_eq!(
            InstructionDetails::<TestStorage>::get(instruction_id),
            instruction_details
        );
        assert_affirms_pending(instruction_id, 2);
        assert_eq!(
            venue_instructions(venue_counter.unwrap()),
            vec![instruction_id]
        );

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        // Ensure affirms are in correct state.
        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        // Ensure legs are in a correct state.
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        // Check that tokens are locked for settlement execution.
        assert_locked_assets(&asset_id, &alice, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction!(bob.origin(), instruction_id, bob.did);

        // Ensure all affirms were successful.
        assert_affirms_pending(instruction_id, 0);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Affirmed);

        // Ensure legs are in a pending state.
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::ExecutionPending);

        // Check that tokens are locked for settlement execution.
        assert_locked_assets(&asset_id, &alice, amount);
        assert_locked_assets(&asset_id2, &bob, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_instruction_status(instruction_id, InstructionStatus::Pending);

        // Instruction should execute on the next block and settlement should fail,
        // since the tokens are still locked for settlement execution.
        next_block();

        assert_instruction_status(instruction_id, InstructionStatus::Failed);

        // Check that tokens stay locked after settlement execution failure.
        assert_locked_assets(&asset_id, &alice, amount);
        assert_locked_assets(&asset_id2, &bob, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                alice.origin(),
                instruction_id,
                None,
                2,
                0,
                0,
                None,
            ),
            Error::FailedAssetTransferringConditions
        ));
    });
}

#[test]
fn venue_filtering() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        // Opt-in so Bob must explicitly affirm
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let block_number = System::block_number() + 1;
        let instruction_id = InstructionCounter::<TestStorage>::get();

        let legs = vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            asset_id,
            amount: 10,
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));
        assert_ok!(Settlement::set_venue_filtering(
            alice.origin(),
            asset_id,
            true
        ));
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                None,
                legs.clone(),
                None,
            ),
            Error::UnauthorizedVenue
        );
        assert_ok!(Settlement::allow_venues(
            alice.origin(),
            asset_id,
            vec![venue_counter.unwrap()]
        ));
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number + 1),
            None,
            None,
            legs.clone(),
            default_asset_holder_set(alice.did),
            None,
        ));

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);
        assert_affirm_instruction!(bob.origin(), instruction_id, bob.did);
        assert_affirm_instruction!(bob.origin(), instruction_id.checked_inc().unwrap(), bob.did);

        next_block();
        assert_eq!(BalanceOf::<TestStorage>::get(&asset_id, bob.did), 10);
        assert_ok!(Settlement::disallow_venues(
            alice.origin(),
            asset_id,
            vec![venue_counter.unwrap()]
        ));
        next_block();
        // Second instruction fails to settle due to venue being not whitelisted
        assert_balance(&asset_id, &bob, 10)
    });
}

#[test]
fn basic_fuzzing() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let dave = User::new(Sr25519Keyring::Dave);
        let venue_counter = VenueCounter::<TestStorage>::get();
        assert_ok!(Settlement::create_venue(
            Origin::signed(Sr25519Keyring::Alice.to_account_id()),
            VenueDetails::default(),
            BTreeSet::from([Sr25519Keyring::Alice.to_account_id()]),
            VenueType::Other
        ));
        let mut assets = Vec::with_capacity(40);
        let mut balances = HashMap::with_capacity(320);
        let users = vec![alice, bob, charlie, dave];

        for _ in 0..10 {
            assets.push(create_and_issue_sample_asset(&alice));
            assets.push(create_and_issue_sample_asset(&bob));
            assets.push(create_and_issue_sample_asset(&charlie));
            assets.push(create_and_issue_sample_asset(&dave));
        }

        let block_number = System::block_number() + 1;
        let instruction_id = InstructionCounter::<TestStorage>::get();

        // initialize balances
        for i in 0..10 {
            for user_id in 0..4 {
                balances.insert(
                    (assets[i * 4 + user_id], users[user_id].did, "init").encode(),
                    ISSUE_AMOUNT,
                );
                balances.insert(
                    (assets[i * 4 + user_id], users[user_id].did, "final").encode(),
                    ISSUE_AMOUNT,
                );
                for k in 0..4 {
                    if user_id == k {
                        continue;
                    }
                    balances.insert((assets[i * 4 + user_id], users[k].did, "init").encode(), 0);
                    balances.insert((assets[i * 4 + user_id], users[k].did, "final").encode(), 0);
                }
            }
        }

        let mut legs = Vec::with_capacity(100);
        let mut legs_count: HashMap<IdentityId, u32> = HashMap::with_capacity(100);
        let mut locked_assets = HashMap::with_capacity(100);
        for i in 0..10 {
            for user_id in 0..4 {
                let mut final_i = ISSUE_AMOUNT;
                balances.insert(
                    (assets[i * 4 + user_id], users[user_id].did, "init").encode(),
                    ISSUE_AMOUNT,
                );
                for k in 0..4 {
                    if user_id == k {
                        continue;
                    }
                    balances.insert((assets[i * 4 + user_id], users[k].did, "init").encode(), 0);
                    if random() {
                        // This leg should happen
                        balances
                            .insert((assets[i * 4 + user_id], users[k].did, "final").encode(), 1);
                        final_i -= 1;
                        *locked_assets
                            .entry((users[user_id].did, assets[i * 4 + user_id]))
                            .or_insert(0) += 1;
                        legs.push(Leg::Fungible {
                            sender: PortfolioId::default_portfolio(users[user_id].did).into(),
                            receiver: PortfolioId::default_portfolio(users[k].did).into(),
                            asset_id: assets[i * 4 + user_id],
                            amount: 1,
                        });
                        *legs_count.entry(users[user_id].did).or_insert(0) += 1;
                        if legs.len() >= 100 {
                            break;
                        }
                    }
                }
                balances.insert(
                    (assets[i * 4 + user_id], users[user_id].did, "final").encode(),
                    final_i,
                );
                if legs.len() >= 100 {
                    break;
                }
            }
            if legs.len() >= 100 {
                break;
            }
        }
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(venue_counter),
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));

        // Authorize instructions
        for (_, user) in users.clone().iter().enumerate() {
            assert_affirm_instruction!(user.origin(), instruction_id, user.did);
        }

        fn check_locked_assets(
            locked_assets: &HashMap<(IdentityId, AssetId), i32>,
            assets: &Vec<AssetId>,
            users: &Vec<User>,
        ) {
            for ((did, asset_id), balance) in locked_assets {
                assert_eq!(
                    PortfolioLockedAssets::<TestStorage>::get(
                        PortfolioId::default_portfolio(*did),
                        asset_id
                    ),
                    *balance as u128
                );
            }
            for asset_id in assets {
                for user in users {
                    assert_eq!(
                        PortfolioLockedAssets::<TestStorage>::get(
                            PortfolioId::default_portfolio(user.did),
                            &asset_id
                        ),
                        locked_assets
                            .get(&(user.did, *asset_id))
                            .cloned()
                            .unwrap_or(0) as u128
                    );
                }
            }
        }

        check_locked_assets(&locked_assets, &assets, &users);

        next_block();

        for asset_id in &assets {
            for user in &users {
                assert_eq!(
                    BalanceOf::<TestStorage>::get(&asset_id, user.did),
                    u128::try_from(
                        *balances
                            .get(&(asset_id, user.did, "final").encode())
                            .unwrap()
                    )
                    .unwrap()
                );
                assert_eq!(
                    PortfolioLockedAssets::<TestStorage>::get(
                        PortfolioId::default_portfolio(user.did),
                        &asset_id
                    ),
                    0
                );
            }
        }

        for asset_id in &assets {
            for user in &users {
                assert_eq!(
                    PortfolioLockedAssets::<TestStorage>::get(
                        PortfolioId::default_portfolio(user.did),
                        asset_id
                    ),
                    0
                );
            }
        }
    });
}

#[test]
fn claim_multiple_receipts_during_authorization() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let ticker2 = Ticker::from_slice_truncated(b"TICKER2".as_ref());
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let id = InstructionCounter::<TestStorage>::get();
        alice.refresh_init_balances();
        bob.refresh_init_balances();
        let amount = 100;

        let legs = vec![
            Leg::OffChain {
                sender_identity: alice.did,
                receiver_identity: bob.did,
                ticker,
                amount,
            },
            Leg::OffChain {
                sender_identity: alice.did,
                receiver_identity: bob.did,
                ticker: ticker2,
                amount,
            },
        ];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
        ));

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        let expires_at = 100u64;
        let msg1 = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(0), alice.did, bob.did, ticker, amount),
        );
        let msg2 = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(1), alice.did, bob.did, ticker2, amount),
        );
        let msg3 = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            1,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(1), alice.did, bob.did, ticker2, amount),
        );

        assert_noop!(
            Settlement::affirm_with_receipts(
                alice.origin(),
                id,
                vec![
                    ReceiptDetails::new(
                        0,
                        id,
                        LegId(0),
                        Sr25519Keyring::Alice.to_account_id(),
                        msg1.sign(&Sr25519Keyring::Alice)
                            .expect("Failed to sign message")
                            .into(),
                        expires_at,
                        None
                    ),
                    ReceiptDetails::new(
                        0,
                        id,
                        LegId(0),
                        Sr25519Keyring::Alice.to_account_id(),
                        msg2.sign(&Sr25519Keyring::Alice)
                            .expect("Failed to sign message")
                            .into(),
                        expires_at,
                        None
                    ),
                ],
                Default::default(),
            ),
            Error::DuplicateReceiptUid
        );

        assert_ok!(Settlement::affirm_with_receipts(
            alice.origin(),
            id,
            vec![
                ReceiptDetails::new(
                    0,
                    id,
                    LegId(0),
                    Sr25519Keyring::Alice.to_account_id(),
                    msg1.sign(&Sr25519Keyring::Alice)
                        .expect("Failed to sign message")
                        .into(),
                    expires_at,
                    None
                ),
                ReceiptDetails::new(
                    1,
                    id,
                    LegId(1),
                    Sr25519Keyring::Alice.to_account_id(),
                    msg3.sign(&Sr25519Keyring::Alice)
                        .expect("Failed to sign message")
                        .into(),
                    expires_at,
                    None
                ),
            ],
            Default::default(),
        ));

        assert_affirms_pending(id, 0);
        assert_eq!(
            OffChainAffirmations::<TestStorage>::get(id, LegId(0)),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            OffChainAffirmations::<TestStorage>::get(id, LegId(1)),
            AffirmationStatus::Affirmed
        );
        assert_leg_status(
            id,
            LegId(0),
            LegStatus::ExecutionToBeSkipped(Sr25519Keyring::Alice.to_account_id(), 0),
        );
        assert_leg_status(
            id,
            LegId(1),
            LegStatus::ExecutionToBeSkipped(Sr25519Keyring::Alice.to_account_id(), 1),
        );
        assert_locked_assets(&asset_id, &alice, 0);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        set_current_block_number(1);

        // Advances block
        next_block();
        assert_user_affirms(id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&asset_id, &alice, 0);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
    });
}

#[test]
fn overload_instruction() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let leg_limit =
            <TestStorage as pallet_settlement::Config>::MaxNumberOfFungibleAssets::get() as usize;

        let mut legs = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount: 1,
            };
            leg_limit + 1
        ];

        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs.clone(),
                None,
            ),
            Error::MaxNumberOfFungibleAssetsExceeded
        );
        legs.truncate(leg_limit);
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            None,
        ));
    });
}

#[test]
fn encode_receipt() {
    ExtBuilder::default().build().execute_with(|| {
        let id = InstructionId(0);
        let identity_id = IdentityId::try_from(
            "did:poly:0600000000000000000000000000000000000000000000000000000000000000",
        )
        .unwrap();
        let expires_at = 100u64;
        let msg1 = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(
                id,
                LegId(0),
                identity_id,
                identity_id,
                Ticker::from_slice_truncated(b"TICKER".as_ref()),
                100,
            ),
        );
        println!("{:?}", Sr25519Keyring::Alice.sign(&msg1.encode()));
    });
}

#[test]
fn test_weights_for_settlement_transaction() {
    ExtBuilder::default()
        .did_registrars(vec![Sr25519Keyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            let alice = User::new(Sr25519Keyring::Alice);
            let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

            let bob = Sr25519Keyring::Bob.to_account_id();
            let (bob_signed, bob_did) = make_account_with_balance(bob, 10_000).unwrap();
            // Opt-in so Bob must explicitly affirm
            assert_ok!(Settlement::set_mandatory_receiver_affirmation(
                bob_signed.clone(),
                AffirmationRequirement::Required
            ));

            let dave = Sr25519Keyring::Dave.to_account_id();
            let (dave_signed, dave_did) = make_account_with_balance(dave, 10_000).unwrap();

            let instruction_id = InstructionCounter::<TestStorage>::get();

            // Add claim rules for settlement
            assert_ok!(ComplianceManager::add_compliance_requirement(
                alice.origin().clone(),
                asset_id,
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(asset_id.into())),
                        &[dave_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAbsent(Claim::BuyLockup(asset_id.into())),
                        &[dave_did]
                    )
                ],
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(asset_id.into())),
                        &[dave_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAnyOf(vec![
                            Claim::BuyLockup(asset_id.into()),
                            Claim::KnowYourCustomer(asset_id.into())
                        ]),
                        &[dave_did]
                    )
                ]
            ));

            // Providing claim to sender and receiver
            // For Alice
            assert_add_claim!(
                dave_signed.clone(),
                alice.did,
                Claim::Accredited(asset_id.into())
            );
            // For Bob
            assert_add_claim!(
                dave_signed.clone(),
                bob_did,
                Claim::Accredited(asset_id.into())
            );
            assert_add_claim!(
                dave_signed.clone(),
                bob_did,
                Claim::KnowYourCustomer(asset_id.into())
            );

            // Create instruction
            let legs = vec![Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob_did).into(),
                asset_id,
                amount: 100,
            }];

            assert_ok!(Settlement::add_instruction(
                alice.origin().clone(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs.clone(),
                None,
            ));

            assert_affirm_instruction!(alice.origin().clone(), instruction_id, alice.did);
            set_current_block_number(100);
            assert_affirm_instruction!(bob_signed.clone(), instruction_id, bob_did);

            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            assert_ok!(Asset::validate_asset_transfer(
                asset_id,
                &PortfolioId::default_portfolio(alice.did).into(),
                &PortfolioId::default_portfolio(bob_did).into(),
                100,
                false,
                &mut weight_meter
            ),);
        });
}

#[test]
fn cross_portfolio_settlement() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        // Opt-in so Bob must explicitly affirm
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);

        let name = PortfolioName::from([42u8].to_vec());
        let num = NextPortfolioNumber::<TestStorage>::get(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Instruction referencing a user defined portfolio is created
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::user_portfolio(bob.did, num).into(),
                asset_id,
                amount,
            }],
            None,
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(num, 0, &asset_id);

        assert_locked_assets(&asset_id, &alice, 0);
        set_current_block_number(10);

        // Approved by Alice
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        assert_locked_assets(&asset_id, &alice, amount);
        // Bob fails to approve the instruction with a
        // different portfolio than the one specified in the instruction
        next_block();
        assert_noop!(
            Settlement::affirm_instruction(
                bob.origin(),
                instruction_id,
                default_asset_holder_set(bob.did),
            ),
            Error::UnexpectedAffirmationStatus
        );

        next_block();
        // Bob approves the instruction with the correct portfolio
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            user_asset_holder_set(bob.did, num),
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
        alice.assert_default_portfolio_bal_decreased(amount, &asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(num, amount, &asset_id);
        assert_locked_assets(&asset_id, &alice, 0);
    });
}

#[test]
fn multiple_portfolio_settlement() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        // Opt-in so Bob must explicitly affirm
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);

        let name = PortfolioName::from([42u8].to_vec());
        let alice_num = NextPortfolioNumber::<TestStorage>::get(&alice.did);
        let bob_num = NextPortfolioNumber::<TestStorage>::get(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        assert_ok!(Portfolio::create_portfolio(alice.origin(), name.clone()));
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // An instruction is created with multiple legs referencing multiple portfolios
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![
                Leg::Fungible {
                    sender: PortfolioId::user_portfolio(alice.did, alice_num).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount,
                },
                Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::user_portfolio(bob.did, bob_num).into(),
                    asset_id,
                    amount,
                }
            ],
            None,
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(bob_num, 0, &asset_id);
        assert_locked_assets(&asset_id, &alice, 0);

        // Alice approves the instruction from her default portfolio
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(bob_num, 0, &asset_id);
        assert_locked_assets(&asset_id, &alice, amount);

        // Alice fails to approve the instruction from her user specified portfolio due to lack of funds
        assert_noop!(
            Settlement::affirm_instruction(
                alice.origin(),
                instruction_id,
                user_asset_holder_set(alice.did, alice_num),
            ),
            AssetError::InsufficientBalance
        );

        // Alice moves her funds to the correct portfolio
        assert_ok!(Portfolio::move_portfolio_funds(
            alice.origin(),
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(alice.did, alice_num),
            vec![Fund {
                description: FundDescription::Fungible { asset_id, amount },
                memo: None,
            }]
        ));
        set_current_block_number(15);
        // Alice is now able to approve the instruction with the user portfolio
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            user_asset_holder_set(alice.did, alice_num),
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount, &asset_id);
        alice.assert_portfolio_bal(alice_num, amount, &asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(bob_num, 0, &asset_id);
        assert_locked_assets(&asset_id, &alice, amount);
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(
                PortfolioId::user_portfolio(alice.did, alice_num),
                &asset_id
            ),
            amount
        );

        // Bob approves the instruction with both of his portfolios in a single transaction
        let portfolios_set: BoundedBTreeSet<_, _> = [
            PortfolioId::default_portfolio(bob.did).into(),
            PortfolioId::user_portfolio(bob.did, bob_num).into(),
        ]
        .into_iter()
        .try_collect()
        .expect("Two portfolios isn't too many");

        next_block();
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            portfolios_set,
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&asset_id, amount * 2);
        bob.assert_balance_increased(&asset_id, amount * 2);
        alice.assert_default_portfolio_bal_decreased(amount * 2, &asset_id);
        bob.assert_default_portfolio_bal_increased(amount, &asset_id);
        bob.assert_portfolio_bal(bob_num, amount, &asset_id);
        assert_locked_assets(&asset_id, &alice, 0);
    });
}

#[test]
fn multiple_custodian_settlement() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        // Both opt-in: Bob governs his default portfolio; Alice will be assigned custodian of
        // Bob's user portfolio, so her opt-in governs that portfolio's affirmation requirement.
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            alice.origin(),
            AffirmationRequirement::Required
        ));
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);

        // Create portfolios
        let name = PortfolioName::from([42u8].to_vec());
        let alice_num = NextPortfolioNumber::<TestStorage>::get(&alice.did);
        let bob_num = NextPortfolioNumber::<TestStorage>::get(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        assert_ok!(Portfolio::create_portfolio(alice.origin(), name.clone()));

        // Give custody of Bob's user portfolio to Alice
        let auth_id = Identity::add_auth(
            bob.did,
            Signatory::from(alice.did),
            AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(bob.did, bob_num)),
            None,
        )
        .unwrap();
        assert_ok!(Portfolio::accept_portfolio_custody(alice.origin(), auth_id));

        // Create a token
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Portfolio::move_portfolio_funds(
            alice.origin(),
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(alice.did, alice_num),
            vec![Fund {
                description: FundDescription::Fungible { asset_id, amount },
                memo: None,
            }]
        ));

        // An instruction is created with multiple legs referencing multiple portfolios
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![
                Leg::Fungible {
                    sender: PortfolioId::user_portfolio(alice.did, alice_num).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount,
                },
                Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::user_portfolio(bob.did, bob_num).into(),
                    asset_id,
                    amount,
                }
            ],
            None,
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount, &asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(bob_num, 0, &asset_id);
        assert_locked_assets(&asset_id, &alice, 0);

        // Alice approves the instruction from both of her portfolios
        let portfolios_set: BoundedBTreeSet<_, _> = [
            PortfolioId::default_portfolio(alice.did).into(),
            PortfolioId::user_portfolio(alice.did, alice_num).into(),
        ]
        .into_iter()
        .try_collect()
        .expect("Number of portfolios under limit");
        set_current_block_number(10);
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            portfolios_set.clone(),
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount, &asset_id);
        bob.assert_default_portfolio_bal_unchanged(&asset_id);
        bob.assert_portfolio_bal(bob_num, 0, &asset_id);
        assert_locked_assets(&asset_id, &alice, amount);
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(
                PortfolioId::user_portfolio(alice.did, alice_num),
                &asset_id
            ),
            amount
        );

        // Alice transfers custody of her portfolios but it won't affect any already approved instruction
        let auth_id2 = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(alice.did, alice_num)),
            None,
        )
        .unwrap();
        assert_ok!(Portfolio::accept_portfolio_custody(bob.origin(), auth_id2));

        // Bob fails to approve the instruction with both of his portfolios since he doesn't have custody for the second one
        let portfolios_bob: BoundedBTreeSet<_, _> = [
            PortfolioId::default_portfolio(bob.did).into(),
            PortfolioId::user_portfolio(bob.did, bob_num).into(),
        ]
        .into_iter()
        .try_collect()
        .expect("Number of portfolios under limit");
        assert_noop!(
            Settlement::affirm_instruction(bob.origin(), instruction_id, portfolios_bob),
            PortfolioError::UnauthorizedCustodian
        );

        next_block();
        // Bob can approve instruction from the portfolio he has custody of
        assert_affirm_instruction!(bob.origin(), instruction_id, bob.did);

        // Alice can authorize instruction from remaining portfolios since she has the custody
        let portfolios_final: BoundedBTreeSet<_, _> =
            [PortfolioId::user_portfolio(bob.did, bob_num).into()]
                .into_iter()
                .try_collect()
                .expect("Number of portfolios under limit");
        next_block();
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            portfolios_final,
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&asset_id, amount * 2);
        bob.assert_balance_increased(&asset_id, amount * 2);
        alice.assert_default_portfolio_bal_decreased(amount * 2, &asset_id);
        bob.assert_default_portfolio_bal_increased(amount, &asset_id);
        bob.assert_portfolio_bal(bob_num, amount, &asset_id);
        assert_locked_assets(&asset_id, &alice, 0);
    });
}

#[test]
fn reject_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let amount = 100u128;

        let reject_instruction = |user: &User, instruction_id| {
            Settlement::reject_instruction(
                user.origin(),
                instruction_id,
                PortfolioId::default_portfolio(user.did).into(),
            )
        };

        let assert_user_affirmations = |instruction_id, alice_status, bob_status| {
            assert_eq!(
                UserAffirmations::<TestStorage>::get(
                    AssetHolder::from(PortfolioId::default_portfolio(alice.did)),
                    instruction_id
                ),
                alice_status
            );
            assert_eq!(
                UserAffirmations::<TestStorage>::get(
                    AssetHolder::from(PortfolioId::default_portfolio(bob.did)),
                    instruction_id
                ),
                bob_status
            );
        };

        let instruction_id = create_instruction(&alice, &bob, venue_counter, asset_id, amount);
        assert_user_affirmations(
            instruction_id,
            AffirmationStatus::Affirmed,
            AffirmationStatus::Affirmed,
        );
        // Try rejecting the instruction from a non-party account.
        assert_noop!(
            reject_instruction(&charlie, instruction_id),
            Error::CallerIsNotAParty
        );
        assert_ok!(reject_instruction(&alice, instruction_id,));
        next_block();
        // Instruction should've been deleted
        assert_user_affirmations(
            instruction_id,
            AffirmationStatus::Unknown,
            AffirmationStatus::Unknown,
        );

        // Test that the receiver can also reject the instruction
        let instruction_id2 = create_instruction(&alice, &bob, venue_counter, asset_id, amount);

        assert_ok!(reject_instruction(&bob, instruction_id2,));
        next_block();
        // Instruction should've been deleted
        assert_user_affirmations(
            instruction_id2,
            AffirmationStatus::Unknown,
            AffirmationStatus::Unknown,
        );
    });
}

#[test]
fn dirty_storage_with_tx() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount1 = 100u128;
        let amount2 = 50u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![
                Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount: amount1,
                },
                Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount: amount2,
                }
            ],
            None,
        ));

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Advances the block no. to execute the instruction.
        let total_amount = amount1 + amount2;
        assert_eq!(
            InstructionAffirmsPending::<TestStorage>::get(instruction_id),
            0
        );
        next_block();
        assert_eq!(
            InstructionLegs::<TestStorage>::iter_prefix(instruction_id).count(),
            0
        );

        // Ensure proper balance transfers
        alice.assert_balance_decreased(&asset_id, total_amount);
        bob.assert_balance_increased(&asset_id, total_amount);
    });
}

#[test]
fn reject_failed_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);
        let amount = 100u128;

        let instruction_id = create_instruction(&alice, &bob, venue_counter, asset_id, amount);

        // Resume compliance to cause transfer failure.
        assert_ok!(ComplianceManager::resume_asset_compliance(
            alice.origin(),
            asset_id
        ));
        assert_ok!(ComplianceManager::reset_asset_compliance(
            alice.origin(),
            asset_id
        ));

        assert_ok!(ComplianceManager::add_compliance_requirement(
            alice.origin(),
            asset_id,
            Default::default(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(bob.did)
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction])
                }]
            }],
        ));

        // Go to next block to have the scheduled execution run and ensure it has failed.
        next_block();
        assert_instruction_status(instruction_id, InstructionStatus::<BlockNumber>::Failed);

        // Reject instruction so that it is pruned on next execution.
        assert_ok!(Settlement::reject_instruction(
            bob.origin(),
            instruction_id,
            PortfolioId::default_portfolio(bob.did).into(),
        ));

        // Go to next block to have the scheduled execution run and ensure it has pruned the instruction.
        next_block();
        assert_instruction_status(
            instruction_id,
            InstructionStatus::Rejected(System::block_number() - 1),
        );
    });
}

#[test]
fn modify_venue_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let venue_counter = VenueCounter::<TestStorage>::get();

        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([
                Sr25519Keyring::Alice.to_account_id(),
                Sr25519Keyring::Bob.to_account_id()
            ]),
            VenueType::Exchange
        ));

        // Charlie fails to add dave to signer list
        assert_noop!(
            Settlement::update_venue_signers(
                charlie.origin(),
                venue_counter,
                BTreeSet::from([Sr25519Keyring::Dave.to_account_id()]),
                true
            ),
            Error::Unauthorized
        );

        // Alice adds charlie to signer list
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_counter,
            BTreeSet::from([Sr25519Keyring::Charlie.to_account_id()]),
            true
        ));

        // Alice fails to remove dave from signer list
        assert_noop!(
            Settlement::update_venue_signers(
                alice.origin(),
                venue_counter,
                BTreeSet::from([Sr25519Keyring::Dave.to_account_id()]),
                false
            ),
            Error::SignerDoesNotExist
        );

        // Alice fails to add charlie to the signer list
        assert_noop!(
            Settlement::update_venue_signers(
                alice.origin(),
                venue_counter,
                BTreeSet::from([Sr25519Keyring::Charlie.to_account_id()]),
                true
            ),
            Error::SignerAlreadyExists
        );

        // Alice removes charlie from signer list
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_counter,
            BTreeSet::from([Sr25519Keyring::Charlie.to_account_id()]),
            false
        ));

        // this checks if the signer is already in the signer list
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, alice.acc()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, Sr25519Keyring::Bob.to_account_id()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(
                venue_counter,
                Sr25519Keyring::Charlie.to_account_id()
            ),
            false
        );

        // Alice adds charlie, dave and eve
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_counter,
            BTreeSet::from([
                Sr25519Keyring::Charlie.to_account_id(),
                Sr25519Keyring::Dave.to_account_id(),
                Sr25519Keyring::Eve.to_account_id(),
            ]),
            true
        ));

        // Alice removes charlie, dave and eve
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_counter,
            BTreeSet::from([
                Sr25519Keyring::Charlie.to_account_id(),
                Sr25519Keyring::Dave.to_account_id(),
                Sr25519Keyring::Eve.to_account_id(),
            ]),
            false
        ));

        // Alice fails to adds charlie, dave, eve and bob
        assert_noop!(
            Settlement::update_venue_signers(
                alice.origin(),
                venue_counter,
                BTreeSet::from([
                    Sr25519Keyring::Charlie.to_account_id(),
                    Sr25519Keyring::Dave.to_account_id(),
                    Sr25519Keyring::Eve.to_account_id(),
                    Sr25519Keyring::Bob.to_account_id()
                ]),
                true
            ),
            Error::SignerAlreadyExists
        );

        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, alice.acc()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, Sr25519Keyring::Bob.to_account_id()),
            true
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(
                venue_counter,
                Sr25519Keyring::Charlie.to_account_id()
            ),
            false
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, Sr25519Keyring::Dave.to_account_id()),
            false
        );
        assert_eq!(
            VenueSigners::<TestStorage>::get(venue_counter, Sr25519Keyring::Eve.to_account_id()),
            false
        );
    });
}

#[test]
fn assert_number_of_venue_signers() {
    ExtBuilder::default().build().execute_with(|| {
        let max_signers =
            <TestStorage as pallet_settlement::Config>::MaxNumberOfVenueSigners::get();
        let venue_id = VenueId(0);
        let alice = User::new(Sr25519Keyring::Alice);
        let initial_signers: BTreeSet<AccountId32> = (0..max_signers as u8)
            .map(|i| AccountId32::from([i; 32]))
            .collect();
        let over_limit_signers: BTreeSet<AccountId32> = (0..max_signers as u8 + 1)
            .map(|i| AccountId32::from([i; 32]))
            .collect();
        // Verifies that an error will be thrown when the limit is exceeded
        assert_noop!(
            Settlement::create_venue(
                alice.origin(),
                VenueDetails::default(),
                over_limit_signers,
                VenueType::Exchange
            ),
            Error::NumberOfVenueSignersExceeded
        );
        // Successfully creates a venue with max_signers
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            initial_signers.clone(),
            VenueType::Exchange
        ));
        assert_eq!(
            NumberOfVenueSigners::<TestStorage>::get(venue_id),
            max_signers
        );
        // Verifies that an error will be thrown when the limit is exceeded
        assert_noop!(
            Settlement::update_venue_signers(
                alice.origin(),
                venue_id,
                BTreeSet::from([AccountId32::from([51; 32])]),
                true
            ),
            Error::NumberOfVenueSignersExceeded
        );
        // Verifies that the count is being updated when removing signers
        let remove_signers: BTreeSet<AccountId32> =
            initial_signers.iter().take(3).cloned().collect();
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_id,
            remove_signers,
            false
        ));
        assert_eq!(
            NumberOfVenueSigners::<TestStorage>::get(venue_id),
            max_signers - 3
        );
        // Verifies that the count is being updated when adding new signers
        let add_signers: BTreeSet<AccountId32> = initial_signers.iter().take(2).cloned().collect();
        assert_ok!(Settlement::update_venue_signers(
            alice.origin(),
            venue_id,
            add_signers,
            true
        ));
        assert_eq!(
            NumberOfVenueSigners::<TestStorage>::get(venue_id),
            max_signers - 1
        );
    })
}

#[test]
fn reject_instruction_with_zero_amount() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let amount = 0u128;

        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![Leg::Fungible {
                    sender: PortfolioId::default_portfolio(alice.did).into(),
                    receiver: PortfolioId::default_portfolio(bob.did).into(),
                    asset_id,
                    amount,
                }],
                None,
            ),
            Error::ZeroAmount
        );
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
    });
}

#[test]
fn basic_settlement_with_memo() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount,
            }],
            Some(Memo::default()),
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // check that the memo was stored correctly
        assert_eq!(
            InstructionMemos::<TestStorage>::get(instruction_id).unwrap(),
            Memo::default()
        );

        set_current_block_number(5);
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        // Advances the block no. to execute the instruction.
        next_block();
        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
    });
}

fn create_instruction(
    alice: &User,
    bob: &User,
    venue_counter: Option<VenueId>,
    asset_id: AssetId,
    amount: u128,
) -> InstructionId {
    let instruction_id = InstructionCounter::<TestStorage>::get();
    set_current_block_number(10);
    assert_ok!(Settlement::add_and_affirm_instruction(
        alice.origin(),
        venue_counter,
        SettlementType::SettleOnAffirmation,
        None,
        None,
        vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            asset_id,
            amount
        }],
        default_asset_holder_set(alice.did),
        None,
    ));
    instruction_id
}

#[test]
fn settle_manual_instruction() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let block_number = System::block_number() + 1;
        let amount = 10u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            asset_id,
            amount,
        }];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleManual(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));

        // Ensure instruction is pending
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);

        // Affirm instruction for alice
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        // Ensure it gave the correct error message after it failed because the execution block number hasn't reached yet
        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                alice.origin(),
                instruction_id,
                None,
                1,
                0,
                0,
                None
            ),
            Error::InstructionSettleBlockNotReached
        ));
        next_block();
        // Ensure bob can't execute instruction with portfolio set to none since he is not the venue creator
        assert_noop!(
            Settlement::execute_manual_instruction(
                bob.origin(),
                instruction_id,
                None,
                1,
                0,
                0,
                None
            ),
            DispatchErrorWithPostInfo {
                post_info: Some(Settlement::execute_manual_instruction_minimum_weight()).into(),
                error: Error::CallerIsNotAParty.into()
            }
        );
        // Ensure correct error message when wrong number of legs is given
        assert_noop!(
            Settlement::execute_manual_instruction(
                alice.origin(),
                instruction_id,
                None,
                0,
                0,
                0,
                None
            ),
            DispatchErrorWithPostInfo {
                post_info: Some(Settlement::execute_manual_instruction_minimum_weight()).into(),
                error: Error::NumberOfFungibleTransfersUnderestimated.into()
            }
        );
        // Ensure it succeeds as the execute block was reached
        assert_ok!(Settlement::execute_manual_instruction(
            alice.origin(),
            instruction_id,
            None,
            1,
            0,
            0,
            None
        ));
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_locked_assets(&asset_id, &alice, 0);

        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);
    });
}

#[test]
fn settle_manual_instruction_with_portfolio() {
    test_with_did_registrar(|_eve| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let mut alice = UserWithBalance::new(alice, &[asset_id]);
        let mut bob = UserWithBalance::new(bob, &[asset_id]);
        let charlie = UserWithBalance::new(charlie, &[asset_id]);

        let alice_portfolio = PortfolioId::default_portfolio(alice.did);
        let charlie_portfolio = PortfolioId::default_portfolio(charlie.did);
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let block_number = System::block_number() + 1;
        let amount = 10u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![Leg::Fungible {
            sender: alice_portfolio.clone().into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            asset_id,
            amount,
        }];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleManual(block_number),
            None,
            None,
            legs.clone(),
            None,
        ));

        // Ensure instruction is pending
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);

        // Affirm instruction for alice
        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did);

        // Ensure it gave the correct error message after it failed because the execution block number hasn't reached yet
        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                alice.origin(),
                instruction_id,
                Some(alice_portfolio.clone().into()),
                1,
                0,
                0,
                None
            ),
            Error::InstructionSettleBlockNotReached
        ));
        next_block();
        // Ensure correct error is shown when non party member tries to execute function
        assert_noop!(
            Settlement::execute_manual_instruction(
                charlie.origin(),
                instruction_id,
                Some(charlie_portfolio.into()),
                1,
                0,
                0,
                None,
            ),
            DispatchErrorWithPostInfo {
                post_info: Some(Settlement::execute_manual_instruction_minimum_weight()).into(),
                error: Error::CallerIsNotAParty.into()
            }
        );
        // Ensure correct error message when wrong number of legs is given
        assert_noop!(
            Settlement::execute_manual_instruction(
                alice.origin(),
                instruction_id,
                Some(alice_portfolio.clone().into()),
                0,
                0,
                0,
                None
            ),
            DispatchErrorWithPostInfo {
                post_info: Some(Settlement::execute_manual_instruction_minimum_weight()).into(),
                error: Error::NumberOfFungibleTransfersUnderestimated.into()
            }
        );
        // Ensure it succeeds as the execute block was reached
        assert_ok!(Settlement::execute_manual_instruction(
            alice.origin(),
            instruction_id,
            Some(alice_portfolio.into()),
            1,
            0,
            0,
            None
        ));
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_locked_assets(&asset_id, &alice, 0);

        alice.assert_balance_decreased(&asset_id, amount);
        bob.assert_balance_increased(&asset_id, amount);

        let mut system_events = System::events();
        assert_eq!(
            system_events.pop().unwrap().event,
            super::storage::EventTest::Settlement(Event::SettlementManuallyExecuted(
                alice.did,
                instruction_id
            ))
        );
        assert_eq!(
            system_events.pop().unwrap().event,
            super::storage::EventTest::Settlement(Event::InstructionExecuted(
                alice.did,
                instruction_id
            ))
        );
    });
}

/// An instruction with non-fungible assets, must reject duplicated NFTIds.
#[test]
fn add_nft_instruction_with_duplicated_nfts() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1), NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                Some(Memo::default()),
            ),
            NFTError::DuplicatedNFTId
        );
    });
}

/// An instruction with non-fungible assets, must reject legs with more than MaxNumberOfNFTsPerLeg.
#[test]
fn add_nft_instruction_exceeding_nfts() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(
            asset_id,
            vec![
                NFTId(1),
                NFTId(2),
                NFTId(3),
                NFTId(4),
                NFTId(5),
                NFTId(6),
                NFTId(7),
                NFTId(8),
                NFTId(9),
                NFTId(10),
                NFTId(11),
            ],
        );
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                Some(Memo::default()),
            ),
            NFTError::MaxNumberOfNFTsPerLegExceeded
        );
    });
}

/// Successfully adds an instruction with non-fungible assets.
#[test]
fn add_nft_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            Some(Memo::default()),
        ));
    });
}

/// Successfully adds and affirms an instruction with non-fungible assets.
#[test]
fn add_and_affirm_nft_instruction() {
    test_with_did_registrar(|_eve| {
        // First we need to create a collection, mint one NFT, and create a venue
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);
        // Opt-in so Bob must explicitly affirm
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );
        ComplianceManager::pause_asset_compliance(alice.origin(), asset_id).unwrap();
        let venue_id = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        // Adds and affirms the instruction
        let instruction_id = InstructionCounter::<TestStorage>::get();
        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            Some(Memo::default()),
        ));

        // Before bob accepts the transaction balances must not be changed and the NFT must be locked.
        assert_eq!(NumberOfNFTs::<TestStorage>::get(asset_id, alice.did), 1);
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                &PortfolioId::default_portfolio(alice.did),
                asset_id,
                NFTId(1)
            )),
            true
        );
        assert_eq!(
            PortfolioLockedNFT::<TestStorage>::get(
                PortfolioId::default_portfolio(alice.did),
                (asset_id, NFTId(1))
            ),
            true
        );

        // Bob affirms the instruction. Balances must be updated and NFT unlocked.
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            default_asset_holder_set(bob.did),
        ));
        next_block();
        assert_eq!(NumberOfNFTs::<TestStorage>::get(asset_id, alice.did), 0);
        assert_eq!(NumberOfNFTs::<TestStorage>::get(asset_id, bob.did), 1);
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                PortfolioId::default_portfolio(alice.did),
                asset_id,
                NFTId(1)
            )),
            false
        );
        assert_eq!(
            PortfolioNFT::<TestStorage>::get((
                PortfolioId::default_portfolio(bob.did),
                asset_id,
                NFTId(1)
            )),
            true
        );
        assert_eq!(
            PortfolioLockedNFT::<TestStorage>::get(
                PortfolioId::default_portfolio(alice.did),
                (asset_id, NFTId(1))
            ),
            false
        );
        assert_eq!(
            PortfolioLockedNFT::<TestStorage>::get(
                PortfolioId::default_portfolio(bob.did),
                (asset_id, NFTId(1))
            ),
            false
        );
    });
}

/// Only instructions with NFTS owned by the caller can be affirmed.
#[test]
fn add_and_affirm_nft_not_owned() {
    test_with_did_registrar(|_eve| {
        // First we need to create a collection, mint one NFT, and create a venue
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );
        let venue_id = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        // Adds and affirms the instruction
        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(2)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_noop!(
            Settlement::add_and_affirm_instruction(
                alice.origin(),
                Some(venue_id),
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                default_asset_holder_set(alice.did),
                Some(Memo::default()),
            ),
            NFTError::NFTNotFound
        );
    });
}

/// An NFT can only be included in one of the legs.
#[test]
fn add_same_nft_different_legs() {
    test_with_did_registrar(|_eve| {
        // First we need to create a collection, mint two NFTs, and create a venue
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata.clone(),
            AssetHolderKind::DefaultPortfolio,
        );
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );
        let venue_id = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        // Adds and affirms the instruction
        let legs: Vec<Leg> = vec![
            Leg::NonFungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
            },
            Leg::NonFungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
            },
        ];
        assert_noop!(
            Settlement::add_and_affirm_instruction(
                alice.origin(),
                Some(venue_id),
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                default_asset_holder_set(alice.did),
                Some(Memo::default()),
            ),
            NFTError::NFTIsLocked
        );
    });
}

/// Receipts can only be used for offchain assets.
#[test]
fn add_and_affirm_with_receipts_nfts() {
    test_with_did_registrar(|_eve| {
        // First we need to create a collection, mint one NFT, and create a venue
        let id = InstructionId(0);
        let ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob: User = User::new(Sr25519Keyring::Bob);
        let collection_keys: NFTCollectionKeys =
            vec![AssetMetadataKey::Local(AssetMetadataLocalKey(1))].into();
        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        let nfts_metadata: Vec<NFTMetadataAttribute> = vec![NFTMetadataAttribute {
            key: AssetMetadataKey::Local(AssetMetadataLocalKey(1)),
            value: AssetMetadataValue(b"test".to_vec()),
        }];
        mint_nft(
            alice.clone(),
            asset_id,
            nfts_metadata,
            AssetHolderKind::DefaultPortfolio,
        );
        let venue_id = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        // Adds the instruction and fails to use a receipt
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts: NFTs::new_unverified(asset_id, vec![NFTId(1)]),
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(venue_id),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            Some(Memo::default()),
        ));
        let expires_at = 100u64;
        let msg1 = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(0), alice.did, bob.did, ticker, 1),
        );
        assert_noop!(
            Settlement::affirm_with_receipts(
                alice.origin(),
                InstructionId(0),
                vec![ReceiptDetails::new(
                    0,
                    id,
                    LegId(0),
                    Sr25519Keyring::Alice.to_account_id(),
                    msg1.sign(&Sr25519Keyring::Alice)
                        .expect("Failed to sign message")
                        .into(),
                    expires_at,
                    None
                )],
                Default::default(),
            ),
            Error::ReceiptForInvalidLegType
        );
    });
}

/// An instruction must reject legs that are not of type off-chain if the ticker is not on chain.
#[test]
fn add_instruction_unexpected_offchain_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let venue_counter = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        let nfts = NFTs::new_unverified([0; 16].into(), vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                Some(venue_counter),
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                Some(Memo::default()),
            ),
            Error::UnexpectedOFFChainAsset
        );

        let legs: Vec<Leg> = vec![Leg::Fungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            asset_id: [0; 16].into(),
            amount: 1,
        }];
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                Some(venue_counter),
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                Some(Memo::default()),
            ),
            Error::UnexpectedOFFChainAsset
        );
    });
}

#[test]
fn add_and_execute_offchain_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let charlie = User::new(Sr25519Keyring::Charlie);
        let alice = User::new(Sr25519Keyring::Alice);
        let dave = User::new(Sr25519Keyring::Dave);
        let bob = User::new(Sr25519Keyring::Bob);
        let ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let (_, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let amount = 1;
        let id = InstructionId(0);

        let legs: Vec<Leg> = vec![Leg::OffChain {
            sender_identity: charlie.did,
            receiver_identity: bob.did,
            ticker,
            amount,
        }];
        let expires_at = 100u64;
        let receipt = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(0), charlie.did, bob.did, ticker, amount),
        );
        let receipts_details = vec![ReceiptDetails::new(
            0,
            id,
            LegId(0),
            Sr25519Keyring::Alice.to_account_id(),
            receipt
                .sign(&Sr25519Keyring::Alice)
                .expect("Failed to sign receipt")
                .into(),
            expires_at,
            None,
        )];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_id,
            SettlementType::SettleManual(System::block_number() + 1),
            None,
            None,
            legs,
            Some(Memo::default()),
        ),);
        assert_ok!(Settlement::affirm_with_receipts(
            alice.origin(),
            id,
            receipts_details,
            Default::default(),
        ),);
        next_block();

        assert_noop!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                0,
                0,
                1,
                None
            ),
            DispatchErrorWithPostInfo {
                post_info: Some(Settlement::execute_manual_instruction_minimum_weight()).into(),
                error: Error::CallerIsNotAParty.into()
            }
        );
        assert_ok!(Settlement::execute_manual_instruction(
            charlie.origin(),
            InstructionId(0),
            None,
            0,
            0,
            1,
            None
        ),);
    });
}

/// Off-chain assets can only be affirmed with receipts.
#[test]
fn affirm_offchain_asset_without_receipt() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let venue = VenueCounter::<TestStorage>::get();
        Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other,
        )
        .unwrap();

        let legs: Vec<Leg> = vec![Leg::OffChain {
            sender_identity: alice.did,
            receiver_identity: bob.did,
            ticker: Ticker::from_slice_truncated(b"TICKER".as_ref()),
            amount: 1,
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(venue),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs,
            Some(Memo::default()),
        ),);
        assert_noop!(
            Settlement::affirm_instruction(
                alice.origin(),
                InstructionId(0),
                default_asset_holder_set(alice.did),
            ),
            Error::UnexpectedAffirmationStatus
        );
    });
}

#[test]
fn add_instruction_with_offchain_assets() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let asset_id2 = AssetId::new([0; 16]);

        let instruction_memo = Some(Memo::default());
        Portfolio::create_portfolio(bob.origin(), b"BobUserPortfolio".into()).unwrap();

        // Both users have pre-affirmed the ticker
        Asset::pre_approve_asset(alice.origin(), asset_id2).unwrap();
        Asset::pre_approve_asset(bob.origin(), asset_id2).unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::OffChain {
                sender_identity: alice.did,
                receiver_identity: bob.did,
                ticker: Ticker::from_slice_truncated(b"TICKER2".as_ref()),
                amount: ONE_UNIT,
            },
            Leg::OffChain {
                sender_identity: alice.did,
                receiver_identity: bob.did,
                ticker: Ticker::from_slice_truncated(b"TICKER".as_ref()),
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));
        // Only the sender still has to approve the transfer
        let portfolios_pending_approval = BTreeSet::from([alice_default_portfolio]);
        let portfolios_pre_approved = BTreeSet::new();
        let offchain_legs = BTreeSet::from([LegId(1), LegId(2)]);
        let instruction_id = InstructionId(0);
        assert_add_instruction_storage(
            &instruction_id,
            &portfolios_pending_approval,
            &portfolios_pre_approved,
            &offchain_legs,
            instruction_memo,
            &legs,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
    });
}

/// The number of pending affirmations can't include receivers that have pre-affirmed the ticker.
#[test]
fn add_instruction_with_pre_affirmed_tickers() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let bob_user_porfolio = PortfolioId::user_portfolio(bob.did, PortfolioNumber(1));
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        Portfolio::create_portfolio(bob.origin(), b"BobUserPortfolio".into()).unwrap();

        // Both users have pre-affirmed the ticker
        Asset::pre_approve_asset(alice.origin(), asset_id).unwrap();
        Asset::pre_approve_asset(bob.origin(), asset_id).unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_user_porfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));
        // Only the sender still has to approve the transfer
        let portfolios_pending_approval = BTreeSet::from([alice_default_portfolio]);
        let portfolios_pre_approved = BTreeSet::from([bob_user_porfolio, bob_default_portfolio]);
        let instruction_id = InstructionId(0);
        assert_add_instruction_storage(
            &instruction_id,
            &portfolios_pending_approval,
            &portfolios_pre_approved,
            &BTreeSet::new(),
            instruction_memo,
            &legs,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
    });
}

/// The number of pending affirmations must include receivers that have pre-affirmed the ticker, but
/// have assigned custodians that have not pre-affirmed the portfolio.
#[test]
fn add_instruction_with_pre_affirmed_tickers_with_assigned_custodian() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob_user_porfolio = PortfolioId::user_portfolio(bob.did, PortfolioNumber(1));
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        Portfolio::create_portfolio(bob.origin(), b"BobUserPortfolio".into()).unwrap();

        // Both users have pre-affirmed the ticker
        Asset::pre_approve_asset(alice.origin(), asset_id).unwrap();
        Asset::pre_approve_asset(bob.origin(), asset_id).unwrap();

        // Bob assigns a custodian to its user portfolio
        let authorization_id = Identity::add_auth(
            bob.did,
            Signatory::from(charlie.did),
            AuthorizationData::PortfolioCustody(bob_user_porfolio.clone()),
            None,
        )
        .unwrap();
        Portfolio::accept_portfolio_custody(charlie.origin(), authorization_id).unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_user_porfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));
        // The sender must affirm. Bob's default portfolio is pre-approved (Bob pre-approved
        // the asset). Bob's user portfolio is also auto-approved because the custodian
        // (Charlie) has not opted in to mandatory receiver affirmation.
        let portfolios_pending_approval = BTreeSet::from([alice_default_portfolio]);
        let portfolios_pre_approved = BTreeSet::from([bob_default_portfolio, bob_user_porfolio]);
        let instruction_id = InstructionId(0);
        assert_add_instruction_storage(
            &instruction_id,
            &portfolios_pending_approval,
            &portfolios_pre_approved,
            &BTreeSet::new(),
            instruction_memo,
            &legs,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
    });
}

/// The number of pending affirmations can't include receivers that have pre-affirmed transfers to a portfolio.
#[test]
fn add_instruction_with_pre_affirmed_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let alice_user_porfolio = PortfolioId::user_portfolio(alice.did, PortfolioNumber(1));
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let bob_user_porfolio = PortfolioId::user_portfolio(bob.did, PortfolioNumber(1));
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        Portfolio::create_portfolio(bob.origin(), b"BobUserPortfolio".into()).unwrap();
        Portfolio::create_portfolio(alice.origin(), b"AliceUserPortfolio".into()).unwrap();

        // Bob opts in to mandatory receiver affirmation so pre-approval is exercised.
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));

        // Both users have pre-affirmed their user portfolios
        Portfolio::pre_approve_portfolio(bob.origin(), asset_id, bob_user_porfolio.clone())
            .unwrap();
        Portfolio::pre_approve_portfolio(alice.origin(), asset_id, alice_user_porfolio.clone())
            .unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_user_porfolio.clone().into(),
                receiver: bob_user_porfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));
        // The sender has to approve both portfolios and the receiver only the default one
        let portfolios_pending_approval = BTreeSet::from([
            alice_default_portfolio,
            alice_user_porfolio,
            bob_default_portfolio,
        ]);
        let portfolios_pre_approved = BTreeSet::from([bob_user_porfolio]);
        let instruction_id = InstructionId(0);
        assert_add_instruction_storage(
            &instruction_id,
            &portfolios_pending_approval,
            &portfolios_pre_approved,
            &BTreeSet::new(),
            instruction_memo,
            &legs,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
    });
}

/// In case a single not pre-affirmed asset is transferred to a portfolio, the number of pending
/// affirmations must include that portfolio.
#[test]
fn add_instruction_with_single_pre_affirmed() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        let asset_id2 = create_and_issue_sample_asset(&alice);

        // Bob opts in to mandatory receiver affirmation so pre-approval is exercised.
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));

        // Bob has pre-affirmed asset_id but not asset_id2
        Asset::pre_approve_asset(bob.origin(), asset_id).unwrap();
        Asset::pre_approve_asset(alice.origin(), asset_id).unwrap();
        Asset::pre_approve_asset(alice.origin(), asset_id2).unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id: asset_id2,
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));
        // Both the sender and receiver have to affirm their portfolio
        let portfolios_pending_approval =
            BTreeSet::from([alice_default_portfolio, bob_default_portfolio]);
        let instruction_id = InstructionId(0);
        assert_add_instruction_storage(
            &instruction_id,
            &portfolios_pending_approval,
            &BTreeSet::new(),
            &BTreeSet::new(),
            instruction_memo,
            &legs,
            &BTreeSet::new(),
            &BTreeSet::new(),
        );
    });
}

/// Successfully executes an instruction after one failed attempt.
#[test]
fn manually_execute_failed_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        let asset_id2 = create_and_issue_sample_asset(&alice);

        // Creates and affirms an instruction and force a failed execution
        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: 1,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id: asset_id2,
                amount: 1,
            },
        ];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnBlock(System::block_number() + 1),
            None,
            None,
            legs.clone(),
            default_asset_holder_set(alice.did),
            instruction_memo.clone(),
        ));
        assert_ok!(Asset::freeze(alice.origin(), asset_id));
        next_block();
        assert_instruction_status(InstructionId(0), InstructionStatus::Failed);
        assert_eq!(
            BalanceOf::<TestStorage>::get(asset_id, alice.did),
            ISSUE_AMOUNT
        );
        assert_eq!(
            BalanceOf::<TestStorage>::get(asset_id2, alice.did),
            ISSUE_AMOUNT
        );
        // Executes the instruction once again, now successfully.
        assert_ok!(Asset::unfreeze(alice.origin(), asset_id));
        assert_ok!(Settlement::execute_manual_instruction(
            alice.origin(),
            InstructionId(0),
            None,
            2,
            0,
            0,
            None
        ));
        assert_eq!(BalanceOf::<TestStorage>::get(asset_id, bob.did), 1);
        assert_eq!(BalanceOf::<TestStorage>::get(asset_id2, bob.did), 1);
        assert_eq!(
            BalanceOf::<TestStorage>::get(asset_id, alice.did),
            ISSUE_AMOUNT - 1
        );
        assert_eq!(
            BalanceOf::<TestStorage>::get(asset_id2, alice.did),
            ISSUE_AMOUNT - 1
        );
        assert_instruction_status(
            InstructionId(0),
            InstructionStatus::Success(System::block_number()),
        );
    });
}

#[test]
fn affirm_with_receipts_cost() {
    ExtBuilder::default().build().execute_with(|| {
        let charlie = User::new(Sr25519Keyring::Charlie);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let ticker = Ticker::from_slice_truncated(b"TICKER2".as_ref());
        let (_, venue_id) = create_and_issue_sample_asset_with_venue(&alice);
        let amount = 1;
        let id = InstructionId(0);

        let legs: Vec<Leg> = vec![Leg::OffChain {
            sender_identity: charlie.did,
            receiver_identity: bob.did,
            ticker,
            amount,
        }];
        let expires_at = 100u64;
        let receipt = ChainScopedMessage::<TestStorage, _>::new_unchecked(
            0,
            SETTLEMENT_RECEIPT_LABEL,
            expires_at,
            Receipt::new(id, LegId(0), charlie.did, bob.did, ticker, amount),
        );
        let receipts_details = vec![ReceiptDetails::new(
            0,
            id,
            LegId(0),
            Sr25519Keyring::Alice.to_account_id(),
            receipt
                .sign(&Sr25519Keyring::Alice)
                .expect("Failed to sign receipt")
                .into(),
            expires_at,
            None,
        )];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_id,
            SettlementType::SettleManual(System::block_number() + 1),
            None,
            None,
            legs,
            Some(Memo::default()),
        ),);

        let affirmation_count =
            AffirmationCount::new(AssetCount::default(), AssetCount::default(), 0);
        assert_noop!(
            Settlement::affirm_with_receipts_with_count(
                alice.origin(),
                id,
                receipts_details,
                Default::default(),
                Some(affirmation_count)
            ),
            Error::NumberOfOffChainTransfersUnderestimated
        );
    });
}

#[test]
fn affirm_instruction_cost() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let alice_user_porfolio = PortfolioId::user_portfolio(alice.did, PortfolioNumber(1));
        let bob = User::new(Sr25519Keyring::Bob);
        // Opt-in so Bob must explicitly affirm
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            bob.origin(),
            AffirmationRequirement::Required
        ));
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let bob_user_porfolio = PortfolioId::user_portfolio(bob.did, PortfolioNumber(1));
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());
        Portfolio::create_portfolio(bob.origin(), b"BobUserPortfolio".into()).unwrap();
        Portfolio::create_portfolio(alice.origin(), b"AliceUserPortfolio".into()).unwrap();

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_user_porfolio.clone().into(),
                receiver: bob_user_porfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ONE_UNIT,
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));

        let affirmation_count =
            AffirmationCount::new(AssetCount::new(0, 0, 0), AssetCount::default(), 0);
        assert_noop!(
            Settlement::affirm_instruction_with_count(
                alice.origin(),
                InstructionId(0),
                vec_to_btreeset(vec![alice_user_porfolio, alice_default_portfolio]),
                Some(affirmation_count)
            ),
            Error::NumberOfFungibleTransfersUnderestimated
        );
        let affirmation_count =
            AffirmationCount::new(AssetCount::default(), AssetCount::new(1, 0, 0), 0);
        assert_noop!(
            Settlement::affirm_instruction_with_count(
                bob.origin(),
                InstructionId(0),
                vec_to_btreeset(vec![bob_user_porfolio, bob_default_portfolio]),
                Some(affirmation_count)
            ),
            Error::NumberOfFungibleTransfersUnderestimated
        );
    });
}

#[test]
fn reject_instruction_cost() {
    ExtBuilder::default().build().execute_with(|| {
        // Setup base parameters
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let bob = User::new(Sr25519Keyring::Bob);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let (asset_id, venue) = create_and_issue_sample_asset_with_venue(&alice);
        let instruction_memo = Some(Memo::default());

        let asset_id2 = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            NFTCollectionKeys::default(),
        );
        mint_nft(
            alice.clone(),
            asset_id2,
            Default::default(),
            AssetHolderKind::DefaultPortfolio,
        );

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: 1,
            },
            Leg::NonFungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                nfts: NFTs::new_unverified(asset_id2, vec![NFTId(1)]),
            },
        ];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            instruction_memo.clone(),
        ));

        assert_noop!(
            Settlement::reject_instruction_with_count(
                bob.origin(),
                InstructionId(0),
                bob_default_portfolio.clone().into(),
                Some(AssetCount::new(1, 0, 0))
            ),
            Error::NumberOfTransferredNFTsUnderestimated
        );
        assert_ok!(Settlement::reject_instruction_with_count(
            bob.origin(),
            InstructionId(0),
            bob_default_portfolio.into(),
            Some(AssetCount::new(1, 1, 0))
        ),);
    });
}

#[test]
fn add_instruction_with_mediators() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let asset_mediator = BTreeSet::from([dave.did]);
        Asset::add_mandatory_mediators(
            alice.origin(),
            asset_id,
            asset_mediator.try_into().unwrap(),
        )
        .unwrap();

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        let instruction_mediators = BTreeSet::from([charlie.did]);
        assert_ok!(Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            instruction_mediators.try_into().unwrap()
        ));

        let portfolios_pending_approval = BTreeSet::from([alice_default_portfolio]);
        let mediators_pending_approval = BTreeSet::from([dave.did, charlie.did]);
        assert_add_instruction_storage(
            &InstructionId(0),
            &portfolios_pending_approval,
            &BTreeSet::new(),
            &BTreeSet::new(),
            None,
            &legs,
            &mediators_pending_approval,
            &BTreeSet::new(),
        );
    });
}

#[test]
fn affirm_as_mediator_invalid_mediator() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_ok!(Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([charlie.did]).try_into().unwrap()
        ));

        assert_noop!(
            Settlement::affirm_instruction_as_mediator(dave.origin(), InstructionId(0), None),
            Error::CallerIsNotAMediator
        );
    });
}

#[test]
fn affirm_as_mediator() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_ok!(Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([charlie.did]).try_into().unwrap()
        ));

        assert_ok!(Settlement::affirm_instruction_as_mediator(
            charlie.origin(),
            InstructionId(0),
            None
        ),);

        let portfolios_pending_approval = BTreeSet::from([alice_default_portfolio]);
        let mediators_affirmed = BTreeSet::from([charlie.did]);
        assert_add_instruction_storage(
            &InstructionId(0),
            &portfolios_pending_approval,
            &BTreeSet::new(),
            &BTreeSet::new(),
            None,
            &legs,
            &BTreeSet::new(),
            &mediators_affirmed,
        );
    });
}

#[test]
fn expired_affirmation_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let legs: Vec<Leg> = vec![Leg::Fungible {
            sender: alice_default_portfolio.into(),
            receiver: bob_default_portfolio.into(),
            asset_id,
            amount: 1,
        }];
        assert_ok!(Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([charlie.did]).try_into().unwrap()
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            InstructionId(0),
            default_asset_holder_set(alice.did),
        ),);
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            charlie.origin(),
            InstructionId(0),
            Some(Timestamp::get() + 1)
        ),);

        Timestamp::set_timestamp(Timestamp::get() + 2);

        next_block();
        assert_instruction_status(InstructionId(0), InstructionStatus::Failed);

        assert_ok!(Settlement::affirm_instruction_as_mediator(
            charlie.origin(),
            InstructionId(0),
            Some(Timestamp::get() + 3)
        ),);
        assert_ok!(Settlement::execute_manual_instruction(
            alice.origin(),
            InstructionId(0),
            None,
            1,
            0,
            0,
            None
        ));
        assert_instruction_status(
            InstructionId(0),
            InstructionStatus::Success(System::block_number()),
        );
    });
}

#[test]
fn reject_instruction_as_mediator() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);

        let (asset_id, venue_counter) = create_and_issue_sample_asset_with_venue(&alice);

        let nfts = NFTs::new_unverified(asset_id, vec![NFTId(1)]);
        let legs: Vec<Leg> = vec![Leg::NonFungible {
            sender: PortfolioId::default_portfolio(alice.did).into(),
            receiver: PortfolioId::default_portfolio(bob.did).into(),
            nfts,
        }];
        assert_ok!(Settlement::add_instruction_with_mediators(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone(),
            None,
            BTreeSet::from([charlie.did]).try_into().unwrap()
        ));

        assert_noop!(
            Settlement::reject_instruction_as_mediator(dave.origin(), InstructionId(0), None),
            Error::CallerIsNotAParty
        );
        assert_ok!(Settlement::reject_instruction_as_mediator(
            charlie.origin(),
            InstructionId(0),
            None
        ),);
        assert_instruction_status(
            InstructionId(0),
            InstructionStatus::Rejected(System::block_number()),
        );
    });
}

#[test]
fn missing_venue_for_offchain_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let (asset_id, _) = create_and_issue_sample_asset_with_venue(&alice);

        let legs: Vec<Leg> = vec![
            Leg::Fungible {
                sender: PortfolioId::default_portfolio(alice.did).into(),
                receiver: PortfolioId::default_portfolio(bob.did).into(),
                asset_id,
                amount: 1_000_000,
            },
            Leg::OffChain {
                sender_identity: alice.did,
                receiver_identity: bob.did,
                ticker: Ticker::from_slice_truncated(b"MYASSET"),
                amount: 1_000_000,
            },
        ];
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                None,
            ),
            Error::OffChainAssetsMustHaveAVenue
        );
    });
}

/// Asserts the storage has been updated after adding an instruction.
/// While each portfolio in `portfolios_pending_approval` must have a pending `AffirmationStatus`, each portfolio in `portfolios_pre_approved`
/// must have an affirmed status. The number of pending affirmations must be equal to the number of portfolios in `portfolios_pending_approval` + the number of offchain legs,
/// all legs must have been included in `InstructionLegs` and `InstructionMemos` must be equal to `instruction_memo`.
fn assert_add_instruction_storage(
    instruction_id: &InstructionId,
    portfolios_pending_approval: &BTreeSet<PortfolioId>,
    portfolios_pre_approved: &BTreeSet<PortfolioId>,
    offchain_legs: &BTreeSet<LegId>,
    instruction_memo: Option<Memo>,
    legs: &[Leg],
    mediators_pending_approval: &BTreeSet<IdentityId>,
    mediators_affirmed: &BTreeSet<IdentityId>,
) {
    portfolios_pending_approval.iter().for_each(|portfolio_id| {
        assert_eq!(
            UserAffirmations::<TestStorage>::get(
                AssetHolder::from(portfolio_id.clone()),
                instruction_id
            ),
            AffirmationStatus::Pending
        )
    });
    portfolios_pre_approved.iter().for_each(|portfolio_id| {
        let asset_holder = AssetHolder::from(portfolio_id.clone());
        assert_eq!(
            UserAffirmations::<TestStorage>::get(&asset_holder, instruction_id),
            AffirmationStatus::Affirmed
        );
        assert_eq!(
            AffirmsReceived::<TestStorage>::get(instruction_id, &asset_holder),
            AffirmationStatus::Affirmed
        )
    });
    offchain_legs.iter().for_each(|leg_id| {
        assert_eq!(
            OffChainAffirmations::<TestStorage>::get(instruction_id, leg_id),
            AffirmationStatus::Pending
        );
    });
    assert_eq!(
        InstructionAffirmsPending::<TestStorage>::get(instruction_id),
        portfolios_pending_approval.len() as u64
            + offchain_legs.len() as u64
            + mediators_pending_approval.len() as u64
    );

    assert_eq!(
        InstructionMemos::<TestStorage>::get(instruction_id),
        instruction_memo
    );

    (0..legs.len()).for_each(|i| {
        assert_eq!(
            InstructionLegs::<TestStorage>::get(instruction_id, LegId(i as u64)),
            Some(legs[i].clone())
        )
    });

    mediators_pending_approval.iter().for_each(|identity_id| {
        assert_eq!(
            InstructionMediatorsAffirmations::<TestStorage>::get(instruction_id, identity_id),
            MediatorAffirmationStatus::Pending
        )
    });
    mediators_affirmed.iter().for_each(|identity_id| {
        match InstructionMediatorsAffirmations::<TestStorage>::get(instruction_id, identity_id) {
            MediatorAffirmationStatus::Pending | MediatorAffirmationStatus::Unknown => {
                panic!("unexpected mediator affirmation status")
            }
            MediatorAffirmationStatus::Affirmed { .. } => {}
        }
    });
}

#[track_caller]
fn assert_instruction_details(
    instruction_id: InstructionId,
    details: Instruction<Moment, BlockNumber>,
) {
    assert_eq!(
        InstructionDetails::<TestStorage>::get(instruction_id),
        details
    );
}

#[track_caller]
fn assert_instruction_status(
    instruction_id: InstructionId,
    status: InstructionStatus<BlockNumber>,
) {
    assert_eq!(
        InstructionStatuses::<TestStorage>::get(instruction_id),
        status
    );
}

#[track_caller]
fn assert_balance(asset_id: &AssetId, user: &User, balance: Balance) {
    assert_eq!(BalanceOf::<TestStorage>::get(asset_id, user.did), balance);
}

#[track_caller]
fn assert_user_affirms(instruction_id: InstructionId, user: &User, status: AffirmationStatus) {
    assert_eq!(
        UserAffirmations::<TestStorage>::get(
            AssetHolder::from(PortfolioId::default_portfolio(user.did)),
            instruction_id
        ),
        status
    );

    let affirms_received_status = match status {
        AffirmationStatus::Pending => AffirmationStatus::Unknown,
        AffirmationStatus::Affirmed => AffirmationStatus::Affirmed,
        _ => return,
    };

    assert_eq!(
        AffirmsReceived::<TestStorage>::get(
            instruction_id,
            AssetHolder::from(PortfolioId::default_portfolio(user.did))
        ),
        affirms_received_status
    );
}

#[track_caller]
fn assert_leg_status(instruction_id: InstructionId, leg: LegId, status: LegStatus<AccountId>) {
    assert_eq!(
        InstructionLegStatus::<TestStorage>::get(instruction_id, leg),
        status
    );
}

#[track_caller]
fn assert_affirms_pending(instruction_id: InstructionId, pending: u64) {
    assert_eq!(
        InstructionAffirmsPending::<TestStorage>::get(instruction_id),
        pending
    );
}

#[track_caller]
fn assert_locked_assets(asset_id: &AssetId, user: &User, num_of_assets: Balance) {
    assert_eq!(
        PortfolioLockedAssets::<TestStorage>::get(
            PortfolioId::default_portfolio(user.did),
            asset_id
        ),
        num_of_assets
    );
}

#[test]
fn set_mandatory_receiver_affirmation() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = AssetId::new([0; 16]);
        let alice_holder: AssetHolder = PortfolioId::default_portfolio(alice.did).into();

        // Default: no mandatory receiver affirmation.
        assert!(!Settlement::identity_requires_affirmation(&alice.did));
        // Receiver affirmation is skipped by default.
        assert!(Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());

        // Opt-in to mandatory receiver affirmation.
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            alice.origin(),
            AffirmationRequirement::Required
        ));
        assert!(Settlement::identity_requires_affirmation(&alice.did));
        // Now receiver affirmation is required.
        assert!(!Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());

        // Pre-approve a specific asset — overrides mandatory receiver affirmation.
        assert_ok!(Asset::pre_approve_asset(alice.origin(), asset_id));
        assert!(Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());

        // Remove pre-approval — mandatory receiver affirmation applies again.
        assert_ok!(Asset::remove_asset_pre_approval(alice.origin(), asset_id));
        assert!(!Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());

        // Global asset exemption overrides mandatory receiver affirmation.
        assert_ok!(Asset::exempt_asset_affirmation(root(), asset_id));
        assert!(Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());
        assert_ok!(Asset::remove_asset_affirmation_exemption(root(), asset_id));

        // Opt out of mandatory receiver affirmation.
        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            alice.origin(),
            AffirmationRequirement::Automatic
        ));
        assert!(!Settlement::identity_requires_affirmation(&alice.did));
        // Affirmation is skipped again.
        assert!(Asset::skip_asset_holder_affirmation(&alice_holder, &asset_id).unwrap());
    });
}
