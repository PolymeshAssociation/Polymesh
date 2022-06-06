use super::{
    asset_test::{allow_all_transfers, max_len_bytes},
    next_block,
    storage::{
        default_portfolio_vec, make_account_without_cdd, provide_scope_claim_to_multiple_parties,
        user_portfolio_vec, TestStorage, User,
    },
    ExtBuilder,
};
use codec::Encode;
use frame_support::{assert_noop, assert_ok, IterableStorageDoubleMap};
use pallet_asset as asset;
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use pallet_portfolio::MovePortfolioItem;
use pallet_scheduler as scheduler;
use pallet_settlement::{
    AffirmationStatus, Instruction, InstructionId, InstructionStatus, Leg, LegId, LegStatus,
    Receipt, ReceiptDetails, ReceiptMetadata, SettlementType, VenueDetails, VenueId,
    VenueInstructions, VenueType,
};
use polymesh_common_utilities::constants::ERC1400_TRANSFER_SUCCESS;
use polymesh_primitives::{
    asset::AssetType, checked_inc::CheckedInc, AccountId, AuthorizationData, Balance, Claim,
    Condition, ConditionType, IdentityId, PortfolioId, PortfolioName, PortfolioNumber, Signatory,
    Ticker,
};
use rand::{prelude::*, thread_rng};
use sp_runtime::AnySignature;
use std::collections::HashMap;
use std::convert::TryFrom;
use std::ops::Deref;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Config>::Origin;
type Moment = <TestStorage as pallet_timestamp::Config>::Moment;
type BlockNumber = <TestStorage as frame_system::Config>::BlockNumber;
type Settlement = pallet_settlement::Module<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Error = pallet_settlement::Error<TestStorage>;
type Scheduler = scheduler::Pallet<TestStorage>;

const TICKER: Ticker = Ticker::new_unchecked([b'A', b'C', b'M', b'E', 0, 0, 0, 0, 0, 0, 0, 0]);
const TICKER2: Ticker = Ticker::new_unchecked([b'A', b'C', b'M', b'E', b'2', 0, 0, 0, 0, 0, 0, 0]);

macro_rules! assert_add_claim {
    ($signer:expr, $target:expr, $claim:expr) => {
        assert_ok!(Identity::add_claim($signer, $target, $claim, None,));
    };
}

macro_rules! assert_affirm_instruction {
    ($signer:expr, $instruction_id:expr, $did:expr, $count:expr) => {
        assert_ok!(Settlement::affirm_instruction(
            $signer,
            $instruction_id,
            default_portfolio_vec($did),
            $count
        ));
    };
}

macro_rules! assert_affirm_instruction_with_one_leg {
    ($signer:expr, $instruction_id:expr, $did:expr) => {
        assert_affirm_instruction!($signer, $instruction_id, $did, 1);
    };
}

macro_rules! assert_affirm_instruction_with_zero_leg {
    ($signer:expr, $instruction_id:expr, $did:expr) => {
        assert_affirm_instruction!($signer, $instruction_id, $did, 0);
    };
}

struct UserWithBalance {
    user: User,
    init_balances: Vec<(Ticker, Balance)>,
}

impl UserWithBalance {
    fn new(acc: AccountKeyring, tickers: &[Ticker]) -> Self {
        let user = User::new(acc);
        Self {
            init_balances: tickers
                .iter()
                .map(|ticker| (*ticker, Asset::balance_of(ticker, user.did)))
                .collect(),
            user,
        }
    }

    fn refresh_init_balances(&mut self) {
        for (ticker, balance) in &mut self.init_balances {
            *balance = Asset::balance_of(ticker, self.user.did);
        }
    }

    #[track_caller]
    fn init_balance(&self, ticker: &Ticker) -> Balance {
        self.init_balances
            .iter()
            .find(|bs| bs.0 == *ticker)
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
    fn assert_balance_unchanged(&self, ticker: &Ticker) {
        assert_balance(ticker, &self.user, self.init_balance(ticker));
    }

    #[track_caller]
    fn assert_balance_increased(&self, ticker: &Ticker, amount: Balance) {
        assert_balance(ticker, &self.user, self.init_balance(ticker) + amount);
    }

    #[track_caller]
    fn assert_balance_decreased(&self, ticker: &Ticker, amount: Balance) {
        assert_balance(ticker, &self.user, self.init_balance(ticker) - amount);
    }

    #[track_caller]
    fn assert_portfolio_bal(&self, num: PortfolioNumber, balance: Balance) {
        assert_eq!(
            Portfolio::user_portfolio_balance(self.user.did, num, &TICKER),
            balance,
        );
    }

    #[track_caller]
    fn assert_default_portfolio_bal(&self, balance: Balance) {
        assert_eq!(
            Portfolio::default_portfolio_balance(self.user.did, &TICKER),
            balance,
        );
    }

    #[track_caller]
    fn assert_default_portfolio_bal_unchanged(&self) {
        self.assert_default_portfolio_bal(self.init_balance(&TICKER));
    }

    #[track_caller]
    fn assert_default_portfolio_bal_decreased(&self, amount: Balance) {
        self.assert_default_portfolio_bal(self.init_balance(&TICKER) - amount);
    }

    #[track_caller]
    fn assert_default_portfolio_bal_increased(&self, amount: Balance) {
        self.assert_default_portfolio_bal(self.init_balance(&TICKER) + amount);
    }
}

impl Deref for UserWithBalance {
    type Target = User;

    fn deref(&self) -> &Self::Target {
        &self.user
    }
}

fn create_token_and_venue(ticker: Ticker, user: User) -> VenueId {
    create_token(ticker, user);
    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        user.origin(),
        VenueDetails::default(),
        vec![user.acc()],
        VenueType::Other
    ));
    venue_counter
}

fn create_token(ticker: Ticker, user: User) {
    assert_ok!(Asset::create_asset(
        user.origin(),
        ticker.as_slice().into(),
        ticker,
        true,
        AssetType::default(),
        vec![],
        None,
        false,
    ));
    assert_ok!(Asset::issue(user.origin(), ticker, 100_000));
    allow_all_transfers(ticker, user);
}

pub fn set_current_block_number(block: u32) {
    System::set_block_number(block);
}

#[test]
fn venue_details_length_limited() {
    ExtBuilder::default().build().execute_with(|| {
        let actor = User::new(AccountKeyring::Alice);
        let id = Settlement::venue_counter();
        let create = |d| Settlement::create_venue(actor.origin(), d, vec![], VenueType::Exchange);
        let update = |d| Settlement::update_venue_details(actor.origin(), id, d);
        assert_too_long!(create(max_len_bytes(1)));
        assert_ok!(create(max_len_bytes(0)));
        assert_too_long!(update(max_len_bytes(1)));
        assert_ok!(update(max_len_bytes(0)));
    });
}

fn venue_instructions(id: VenueId) -> Vec<InstructionId> {
    VenueInstructions::iter_prefix(id).map(|(i, _)| i).collect()
}

#[test]
fn venue_registration() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let venue_counter = Settlement::venue_counter();
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            vec![
                AccountKeyring::Alice.to_account_id(),
                AccountKeyring::Bob.to_account_id()
            ],
            VenueType::Exchange
        ));
        let venue_info = Settlement::venue_info(venue_counter).unwrap();
        assert_eq!(
            Settlement::venue_counter(),
            venue_counter.checked_inc().unwrap()
        );
        assert_eq!(Settlement::user_venues(alice.did), [venue_counter]);
        assert_eq!(venue_info.creator, alice.did);
        assert_eq!(venue_instructions(venue_counter).len(), 0);
        assert_eq!(Settlement::details(venue_counter), VenueDetails::default());
        assert_eq!(venue_info.venue_type, VenueType::Exchange);
        assert_eq!(Settlement::venue_signers(venue_counter, alice.acc()), true);
        assert_eq!(
            Settlement::venue_signers(venue_counter, AccountKeyring::Bob.to_account_id()),
            true
        );
        assert_eq!(
            Settlement::venue_signers(venue_counter, AccountKeyring::Charlie.to_account_id()),
            false
        );

        // Creating a second venue
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            vec![alice.acc(), AccountKeyring::Bob.to_account_id()],
            VenueType::Exchange
        ));
        assert_eq!(
            Settlement::user_venues(alice.did),
            [venue_counter, venue_counter.checked_inc().unwrap()]
        );

        // Editing venue details
        assert_ok!(Settlement::update_venue_details(
            alice.origin(),
            venue_counter,
            [0x01].into(),
        ));
        let venue_info = Settlement::venue_info(venue_counter).unwrap();
        assert_eq!(venue_info.creator, alice.did);
        assert_eq!(venue_instructions(venue_counter).len(), 0);
        assert_eq!(Settlement::details(venue_counter), [0x01].into());
        assert_eq!(venue_info.venue_type, VenueType::Exchange);
    });
}

fn test_with_cdd_provider(test: impl FnOnce(AccountId)) {
    let cdd = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![cdd.clone()])
        .build()
        .execute_with(|| test(cdd));
}

#[test]
fn basic_settlement() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to sender and receiver of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount: amount
            }]
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        set_current_block_number(5);
        // Instruction get scheduled to next block.
        assert_affirm_instruction_with_zero_leg!(bob.origin(), instruction_id, bob.did);

        // Advances the block no. to execute the instruction.
        next_block();
        alice.assert_balance_decreased(&TICKER, amount);
        bob.assert_balance_increased(&TICKER, amount);
    });
}

#[test]
fn create_and_affirm_instruction() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to both the parties of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        let add_and_affirm_tx = |affirm_from_portfolio| {
            Settlement::add_and_affirm_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                vec![Leg {
                    from: PortfolioId::default_portfolio(alice.did),
                    to: PortfolioId::default_portfolio(bob.did),
                    asset: TICKER,
                    amount,
                }],
                affirm_from_portfolio,
            )
        };

        // If affirmation fails, the instruction should be rolled back.
        // i.e. this tx should be a no-op.
        assert_noop!(
            add_and_affirm_tx(user_portfolio_vec(alice.did, 1u64.into())),
            Error::UnexpectedAffirmationStatus
        );

        assert_ok!(add_and_affirm_tx(default_portfolio_vec(alice.did)));

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        set_current_block_number(5);

        assert_affirm_instruction_with_zero_leg!(bob.origin(), instruction_id, bob.did);

        // Advances the block no.
        next_block();
        alice.assert_balance_decreased(&TICKER, amount);
        bob.assert_balance_increased(&TICKER, amount);
    });
}

#[test]
fn overdraft_failure() {
    ExtBuilder::default().build().execute_with(|| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100_000_000u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount: amount
            }]
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        assert_noop!(
            Settlement::affirm_instruction(
                alice.origin(),
                instruction_id,
                default_portfolio_vec(alice.did),
                1
            ),
            Error::FailedToLockTokens
        );
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
    });
}

#[test]
fn token_swap() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER, TICKER2]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER, TICKER2]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        create_token(TICKER2, bob.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount,
            },
            Leg {
                from: PortfolioId::default_portfolio(bob.did),
                to: PortfolioId::default_portfolio(alice.did),
                asset: TICKER2,
                amount,
            },
        ];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone()
        ));

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                Settlement::instruction_legs(
                    instruction_id,
                    u64::try_from(i).map(LegId).unwrap_or_default()
                ),
                legs[i]
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            status: InstructionStatus::Pending,
            settlement_type: SettlementType::SettleOnAffirmation,
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_instruction_details(instruction_id, instruction_details);

        assert_affirms_pending(instruction_id, 2);
        assert_eq!(venue_instructions(venue_counter), vec![instruction_id]);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Provide scope claim to parties involved in a instruction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve.clone());
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER2, eve);

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);
        assert_affirms_pending(instruction_id, 1);

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        assert_locked_assets(&TICKER, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_ok!(Settlement::withdraw_affirmation(
            alice.origin(),
            instruction_id,
            default_portfolio_vec(alice.did),
            1
        ));

        assert_affirms_pending(instruction_id, 2);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        assert_leg_status(instruction_id, LegId(0), LegStatus::PendingTokenLock);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        assert_locked_assets(&TICKER, &alice, 0);
        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        assert_locked_assets(&TICKER, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        set_current_block_number(500);

        assert_affirm_instruction_with_one_leg!(bob.origin(), instruction_id, bob.did);

        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&TICKER, &alice, 0);
        alice.assert_balance_decreased(&TICKER, amount);
        alice.assert_balance_increased(&TICKER2, amount);
        bob.assert_balance_increased(&TICKER, amount);
        bob.assert_balance_decreased(&TICKER2, amount);
    });
}

#[test]
fn claiming_receipt() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER, TICKER2]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER, TICKER2]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        create_token(TICKER2, bob.user);
        let instruction_id = Settlement::instruction_counter();
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claims to multiple parties of a transactions.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve.clone());
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER2, eve);

        let amount = 100u128;
        let legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount,
            },
            Leg {
                from: PortfolioId::default_portfolio(bob.did),
                to: PortfolioId::default_portfolio(alice.did),
                asset: TICKER2,
                amount,
            },
        ];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone()
        ));

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                Settlement::instruction_legs(
                    instruction_id,
                    u64::try_from(i).map(LegId).unwrap_or_default()
                ),
                legs[i]
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            status: InstructionStatus::Pending,
            settlement_type: SettlementType::SettleOnAffirmation,
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_eq!(
            Settlement::instruction_details(instruction_id),
            instruction_details
        );

        assert_affirms_pending(instruction_id, 2);
        assert_eq!(venue_instructions(venue_counter), vec![instruction_id]);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        let msg = Receipt {
            receipt_uid: 0,
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: TICKER,
            amount,
        };
        let signature = AccountKeyring::Alice.sign(&msg.encode());

        let claim_receipt = |signature, metadata| {
            Settlement::claim_receipt(
                alice.origin(),
                instruction_id,
                ReceiptDetails {
                    receipt_uid: 0,
                    leg_id: LegId(0),
                    signer: AccountKeyring::Alice.to_account_id(),
                    signature,
                    metadata,
                },
            )
        };

        assert_noop!(
            claim_receipt(signature.clone().into(), ReceiptMetadata::default()),
            Error::LegNotPending
        );
        set_current_block_number(4);

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&TICKER, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        let msg2 = Receipt {
            receipt_uid: 0,
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(alice.did),
            asset: TICKER,
            amount,
        };
        let signature2 = AccountKeyring::Alice.sign(&msg2.encode());

        assert_noop!(
            claim_receipt(signature2.into(), ReceiptMetadata::default()),
            Error::InvalidSignature
        );

        let metadata = ReceiptMetadata::from(vec![42u8]);

        // Can not claim invalidated receipt
        let change_receipt_validity = |validity| {
            assert_ok!(Settlement::change_receipt_validity(
                alice.origin(),
                0,
                validity
            ));
        };
        change_receipt_validity(false);
        assert_noop!(
            claim_receipt(signature.clone().into(), metadata.clone()),
            Error::ReceiptAlreadyClaimed
        );
        change_receipt_validity(true);

        // Claiming, unclaiming and claiming receipt
        assert_ok!(claim_receipt(signature.into(), metadata.clone()));

        assert_eq!(
            Settlement::receipts_used(AccountKeyring::Alice.to_account_id(), 0),
            true
        );
        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(
            instruction_id,
            LegId(0),
            LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.to_account_id(), 0),
        );
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&TICKER, &alice, 0);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_ok!(Settlement::unclaim_receipt(
            alice.origin(),
            instruction_id,
            LegId(0)
        ));

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&TICKER, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_ok!(Settlement::claim_receipt(
            alice.origin(),
            instruction_id,
            ReceiptDetails {
                receipt_uid: 0,
                leg_id: LegId(0),
                signer: AccountKeyring::Alice.to_account_id(),
                signature: AccountKeyring::Alice.sign(&msg.encode()).into(),
                metadata: ReceiptMetadata::default()
            }
        ));

        assert_eq!(
            Settlement::receipts_used(AccountKeyring::Alice.to_account_id(), 0),
            true
        );
        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(
            instruction_id,
            LegId(0),
            LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.to_account_id(), 0),
        );
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&TICKER, &alice, 0);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        set_current_block_number(10);

        assert_affirm_instruction_with_one_leg!(bob.origin(), instruction_id, bob.did);

        // Advances block.
        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&TICKER, &alice, 0);
        alice.assert_balance_unchanged(&TICKER);
        bob.assert_balance_unchanged(&TICKER);
        alice.assert_balance_increased(&TICKER2, amount);
        bob.assert_balance_decreased(&TICKER2, amount);
    });
}

#[test]
fn settle_on_block() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER, TICKER2]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER, TICKER2]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        create_token(TICKER2, bob.user);
        let instruction_id = Settlement::instruction_counter();
        let block_number = System::block_number() + 1;
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount,
            },
            Leg {
                from: PortfolioId::default_portfolio(bob.did),
                to: PortfolioId::default_portfolio(alice.did),
                asset: TICKER2,
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
            legs.clone()
        ));
        assert_eq!(1, scheduler::Agenda::<TestStorage>::get(block_number).len());

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                Settlement::instruction_legs(
                    instruction_id,
                    u64::try_from(i).map(LegId).unwrap_or_default()
                ),
                legs[i]
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            status: InstructionStatus::Pending,
            settlement_type: SettlementType::SettleOnBlock(block_number),
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_eq!(
            Settlement::instruction_details(instruction_id),
            instruction_details
        );

        assert_affirms_pending(instruction_id, 2);
        assert_eq!(venue_instructions(venue_counter), vec![instruction_id]);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Before authorization need to provide the scope claim for both the parties of a transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve.clone());
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER2, eve);

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);
        assert_locked_assets(&TICKER, &alice, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction_with_one_leg!(bob.origin(), instruction_id, bob.did);

        assert_affirms_pending(instruction_id, 0);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Affirmed);
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::ExecutionPending);
        assert_locked_assets(&TICKER, &alice, amount);
        assert_locked_assets(&TICKER2, &bob, amount);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Instruction should've settled
        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&TICKER, &alice, 0);
        assert_locked_assets(&TICKER, &bob, 0);

        alice.assert_balance_decreased(&TICKER, amount);
        bob.assert_balance_increased(&TICKER, amount);
        alice.assert_balance_increased(&TICKER2, amount);
        bob.assert_balance_decreased(&TICKER2, amount);
    });
}

#[test]
fn failed_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER, TICKER2]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER, TICKER2]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        create_token(TICKER2, bob.user);
        let instruction_id = Settlement::instruction_counter();
        assert_ok!(ComplianceManager::reset_asset_compliance(
            Origin::signed(AccountKeyring::Bob.to_account_id()),
            TICKER2,
        ));
        let block_number = System::block_number() + 1;
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount,
            },
            Leg {
                from: PortfolioId::default_portfolio(bob.did),
                to: PortfolioId::default_portfolio(alice.did),
                asset: TICKER2,
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
            legs.clone()
        ));
        assert_eq!(1, scheduler::Agenda::<TestStorage>::get(block_number).len());

        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Pending);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        for i in 0..legs.len() {
            assert_eq!(
                Settlement::instruction_legs(
                    instruction_id,
                    u64::try_from(i).map(LegId).unwrap_or_default()
                ),
                legs[i]
            );
        }

        let instruction_details = Instruction {
            instruction_id,
            venue_id: venue_counter,
            status: InstructionStatus::Pending,
            settlement_type: SettlementType::SettleOnBlock(block_number),
            created_at: Some(Timestamp::get()),
            trade_date: None,
            value_date: None,
        };
        assert_eq!(
            Settlement::instruction_details(instruction_id),
            instruction_details
        );
        assert_affirms_pending(instruction_id, 2);
        assert_eq!(venue_instructions(venue_counter), vec![instruction_id]);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        // Ensure affirms are in correct state.
        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);

        // Ensure legs are in a correct state.
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::PendingTokenLock);

        // Check that tokens are locked for settlement execution.
        assert_locked_assets(&TICKER, &alice, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_affirm_instruction_with_one_leg!(bob.origin(), instruction_id, bob.did);

        // Ensure all affirms were successful.
        assert_affirms_pending(instruction_id, 0);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Affirmed);

        // Ensure legs are in a pending state.
        assert_leg_status(instruction_id, LegId(0), LegStatus::ExecutionPending);
        assert_leg_status(instruction_id, LegId(1), LegStatus::ExecutionPending);

        // Check that tokens are locked for settlement execution.
        assert_locked_assets(&TICKER, &alice, amount);
        assert_locked_assets(&TICKER2, &bob, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        assert_instruction_status(instruction_id, InstructionStatus::Pending);

        // Instruction should execute on the next block and settlement should fail,
        // since the tokens are still locked for settlement execution.
        next_block();

        assert_instruction_status(instruction_id, InstructionStatus::Failed);

        // Check that tokens stay locked after settlement execution failure.
        assert_locked_assets(&TICKER, &alice, amount);
        assert_locked_assets(&TICKER2, &bob, amount);

        // Ensure balances have not changed.
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        // Reschedule instruction and ensure the state is identical to the original state.
        assert_ok!(Settlement::reschedule_instruction(
            alice.origin(),
            instruction_id
        ));
        assert_eq!(
            Settlement::instruction_details(instruction_id),
            instruction_details
        );
    });
}

#[test]
fn venue_filtering() {
    test_with_cdd_provider(|eve| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let venue_counter = create_token_and_venue(TICKER, alice);
        let block_number = System::block_number() + 1;
        let instruction_id = Settlement::instruction_counter();

        // provide scope claim.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        let legs = vec![Leg {
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: TICKER,
            amount: 10,
        }];
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone()
        ));
        assert_ok!(Settlement::set_venue_filtering(
            alice.origin(),
            TICKER,
            true
        ));
        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                None,
                legs.clone()
            ),
            Error::UnauthorizedVenue
        );
        assert_ok!(Settlement::allow_venues(
            alice.origin(),
            TICKER,
            vec![venue_counter]
        ));
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnBlock(block_number + 1),
            None,
            None,
            legs.clone(),
            default_portfolio_vec(alice.did)
        ));

        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);
        assert_affirm_instruction_with_zero_leg!(bob.origin(), instruction_id, bob.did);
        assert_affirm_instruction_with_zero_leg!(
            bob.origin(),
            instruction_id.checked_inc().unwrap(),
            bob.did
        );

        next_block();
        assert_eq!(Asset::balance_of(&TICKER, bob.did), 10);
        assert_ok!(Settlement::disallow_venues(
            alice.origin(),
            TICKER,
            vec![venue_counter]
        ));
        next_block();
        // Second instruction fails to settle due to venue being not whitelisted
        assert_balance(&TICKER, &bob, 10)
    });
}

#[test]
fn basic_fuzzing() {
    test_with_cdd_provider(|_eve| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        let dave = User::new(AccountKeyring::Dave);
        let eve = User::existing(AccountKeyring::Eve);
        let venue_counter = Settlement::venue_counter();
        assert_ok!(Settlement::create_venue(
            Origin::signed(AccountKeyring::Alice.to_account_id()),
            VenueDetails::default(),
            vec![AccountKeyring::Alice.to_account_id()],
            VenueType::Other
        ));
        let mut tickers = Vec::with_capacity(40);
        let mut balances = HashMap::with_capacity(320);
        let users = vec![alice, bob, charlie, dave];

        for ticker_id in 0..10 {
            let mut create = |x: usize, user: User| {
                let tn = [b'!' + u8::try_from(ticker_id * 4 + x).unwrap()];
                tickers.push(Ticker::try_from(&tn[..]).unwrap());
                create_token(tickers[ticker_id * 4 + x], user);
            };
            create(0, alice);
            create(1, bob);
            create(2, charlie);
            create(3, dave);
        }

        let block_number = System::block_number() + 1;
        let instruction_id = Settlement::instruction_counter();

        // initialize balances
        for ticker_id in 0..10 {
            for user_id in 0..4 {
                balances.insert(
                    (tickers[ticker_id * 4 + user_id], users[user_id].did, "init").encode(),
                    100_000,
                );
                balances.insert(
                    (
                        tickers[ticker_id * 4 + user_id],
                        users[user_id].did,
                        "final",
                    )
                        .encode(),
                    100_000,
                );
                for k in 0..4 {
                    if user_id == k {
                        continue;
                    }
                    balances.insert(
                        (tickers[ticker_id * 4 + user_id], users[k].did, "init").encode(),
                        0,
                    );
                    balances.insert(
                        (tickers[ticker_id * 4 + user_id], users[k].did, "final").encode(),
                        0,
                    );
                }
            }
        }

        let mut legs = Vec::with_capacity(100);
        let mut receipts = Vec::with_capacity(100);
        let mut receipt_legs = HashMap::with_capacity(100);
        let mut legs_count: HashMap<IdentityId, u32> = HashMap::with_capacity(100);
        let mut locked_assets = HashMap::with_capacity(100);
        for ticker_id in 0..10 {
            for user_id in 0..4 {
                let mut final_i = 100_000;
                balances.insert(
                    (tickers[ticker_id * 4 + user_id], users[user_id].did, "init").encode(),
                    100_000,
                );
                for k in 0..4 {
                    if user_id == k {
                        continue;
                    }
                    balances.insert(
                        (tickers[ticker_id * 4 + user_id], users[k].did, "init").encode(),
                        0,
                    );
                    if random() {
                        // This leg should happen
                        if random() {
                            // Receipt to be claimed
                            balances.insert(
                                (tickers[ticker_id * 4 + user_id], users[k].did, "final").encode(),
                                0,
                            );
                            receipts.push(Receipt {
                                receipt_uid: u64::try_from(k * 1000 + ticker_id * 4 + user_id)
                                    .unwrap(),
                                from: PortfolioId::default_portfolio(users[user_id].did),
                                to: PortfolioId::default_portfolio(users[k].did),
                                asset: tickers[ticker_id * 4 + user_id],
                                amount: 1u128,
                            });
                            receipt_legs.insert(receipts.last().unwrap().encode(), legs.len());
                        } else {
                            balances.insert(
                                (tickers[ticker_id * 4 + user_id], users[k].did, "final").encode(),
                                1,
                            );
                            final_i -= 1;
                            *locked_assets
                                .entry((users[user_id].did, tickers[ticker_id * 4 + user_id]))
                                .or_insert(0) += 1;
                        }
                        // Provide scope claim for all the dids
                        provide_scope_claim_to_multiple_parties(
                            &[users[user_id].did, users[k].did],
                            tickers[ticker_id * 4 + user_id],
                            eve.acc(),
                        );
                        legs.push(Leg {
                            from: PortfolioId::default_portfolio(users[user_id].did),
                            to: PortfolioId::default_portfolio(users[k].did),
                            asset: tickers[ticker_id * 4 + user_id],
                            amount: 1,
                        });
                        *legs_count.entry(users[user_id].did).or_insert(0) += 1;
                        if legs.len() >= 100 {
                            break;
                        }
                    }
                }
                balances.insert(
                    (
                        tickers[ticker_id * 4 + user_id],
                        users[user_id].did,
                        "final",
                    )
                        .encode(),
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
            venue_counter,
            SettlementType::SettleOnBlock(block_number),
            None,
            None,
            legs.clone()
        ));

        // Authorize instructions and do a few authorize/deny in between
        for (_, user) in users.clone().iter().enumerate() {
            let leg_count = *legs_count.get(&user.did).unwrap_or(&0);
            for _ in 0..2 {
                if random() {
                    assert_affirm_instruction!(user.origin(), instruction_id, user.did, leg_count);
                    assert_ok!(Settlement::withdraw_affirmation(
                        user.origin(),
                        instruction_id,
                        default_portfolio_vec(user.did),
                        leg_count
                    ));
                }
            }
            assert_affirm_instruction!(user.origin(), instruction_id, user.did, leg_count);
        }

        // Claim receipts and do a few claim/unclaims in between
        for receipt in receipts {
            let leg_num = u64::try_from(*receipt_legs.get(&(receipt.encode())).unwrap())
                .map(LegId)
                .unwrap();
            let user = users
                .iter()
                .filter(|&from| PortfolioId::default_portfolio(from.did) == receipt.from)
                .next()
                .unwrap();
            for _ in 0..2 {
                if random() {
                    assert_ok!(Settlement::claim_receipt(
                        user.origin(),
                        instruction_id,
                        ReceiptDetails {
                            receipt_uid: receipt.receipt_uid,
                            leg_id: leg_num,
                            signer: AccountKeyring::Alice.to_account_id(),
                            signature: AccountKeyring::Alice.sign(&receipt.encode()).into(),
                            metadata: ReceiptMetadata::default()
                        }
                    ));
                    assert_ok!(Settlement::unclaim_receipt(
                        user.origin(),
                        instruction_id,
                        leg_num
                    ));
                }
            }
            assert_ok!(Settlement::claim_receipt(
                user.origin(),
                instruction_id,
                ReceiptDetails {
                    receipt_uid: receipt.receipt_uid,
                    leg_id: leg_num,
                    signer: AccountKeyring::Alice.to_account_id(),
                    signature: AccountKeyring::Alice.sign(&receipt.encode()).into(),
                    metadata: ReceiptMetadata::default()
                }
            ));
        }

        fn check_locked_assets(
            locked_assets: &HashMap<(IdentityId, Ticker), i32>,
            tickers: &Vec<Ticker>,
            users: &Vec<User>,
        ) {
            for ((did, ticker), balance) in locked_assets {
                assert_eq!(
                    Portfolio::locked_assets(PortfolioId::default_portfolio(*did), ticker),
                    *balance as u128
                );
            }
            for ticker in tickers {
                for user in users {
                    assert_eq!(
                        Portfolio::locked_assets(PortfolioId::default_portfolio(user.did), &ticker),
                        locked_assets
                            .get(&(user.did, *ticker))
                            .cloned()
                            .unwrap_or(0) as u128
                    );
                }
            }
        }

        check_locked_assets(&locked_assets, &tickers, &users);

        let fail: bool = random();
        let mut rng = thread_rng();
        let failed_user = rng.gen_range(0, 4);
        if fail {
            assert_ok!(Settlement::withdraw_affirmation(
                users[failed_user].origin(),
                instruction_id,
                default_portfolio_vec(users[failed_user].did),
                *legs_count.get(&users[failed_user].did).unwrap_or(&0)
            ));
            locked_assets.retain(|(did, _), _| *did != users[failed_user].did);
        }

        next_block();

        if fail {
            assert_eq!(
                Settlement::instruction_details(instruction_id).status,
                InstructionStatus::Failed
            );
            check_locked_assets(&locked_assets, &tickers, &users);
        }

        for ticker in &tickers {
            for user in &users {
                if fail {
                    assert_eq!(
                        Asset::balance_of(&ticker, user.did),
                        u128::try_from(
                            *balances.get(&(ticker, user.did, "init").encode()).unwrap()
                        )
                        .unwrap()
                    );
                    assert_eq!(
                        Portfolio::locked_assets(PortfolioId::default_portfolio(user.did), &ticker),
                        locked_assets
                            .get(&(user.did, *ticker))
                            .cloned()
                            .unwrap_or(0) as u128
                    );
                } else {
                    assert_eq!(
                        Asset::balance_of(&ticker, user.did),
                        u128::try_from(
                            *balances.get(&(ticker, user.did, "final").encode()).unwrap()
                        )
                        .unwrap()
                    );
                    assert_eq!(
                        Portfolio::locked_assets(PortfolioId::default_portfolio(user.did), &ticker),
                        0
                    );
                }
            }
        }

        if fail {
            assert_ok!(Settlement::reject_instruction(
                users[0].origin(),
                instruction_id,
                PortfolioId::default_portfolio(users[0].did),
                legs.len() as u32,
            ));
            assert_eq!(
                Settlement::instruction_details(instruction_id).status,
                InstructionStatus::Unknown
            );
        }

        for ticker in &tickers {
            for user in &users {
                assert_eq!(
                    Portfolio::locked_assets(PortfolioId::default_portfolio(user.did), ticker),
                    0
                );
            }
        }
    });
}

#[test]
fn claim_multiple_receipts_during_authorization() {
    ExtBuilder::default().build().execute_with(|| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        let legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount,
            },
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER2,
                amount,
            },
        ];

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs.clone()
        ));

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        let msg1 = Receipt {
            receipt_uid: 0,
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: TICKER,
            amount,
        };
        let msg2 = Receipt {
            receipt_uid: 0,
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: TICKER2,
            amount,
        };
        let msg3 = Receipt {
            receipt_uid: 1,
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: TICKER2,
            amount,
        };

        assert_noop!(
            Settlement::affirm_with_receipts(
                alice.origin(),
                instruction_id,
                vec![
                    ReceiptDetails {
                        receipt_uid: 0,
                        leg_id: LegId(0),
                        signer: AccountKeyring::Alice.to_account_id(),
                        signature: AccountKeyring::Alice.sign(&msg1.encode()).into(),
                        metadata: ReceiptMetadata::default()
                    },
                    ReceiptDetails {
                        receipt_uid: 0,
                        leg_id: LegId(0),
                        signer: AccountKeyring::Alice.to_account_id(),
                        signature: AccountKeyring::Alice.sign(&msg2.encode()).into(),
                        metadata: ReceiptMetadata::default()
                    },
                ],
                default_portfolio_vec(alice.did),
                10
            ),
            Error::ReceiptAlreadyClaimed
        );

        assert_ok!(Settlement::affirm_with_receipts(
            alice.origin(),
            instruction_id,
            vec![
                ReceiptDetails {
                    receipt_uid: 0,
                    leg_id: LegId(0),
                    signer: AccountKeyring::Alice.to_account_id(),
                    signature: AccountKeyring::Alice.sign(&msg1.encode()).into(),
                    metadata: ReceiptMetadata::default()
                },
                ReceiptDetails {
                    receipt_uid: 1,
                    leg_id: LegId(1),
                    signer: AccountKeyring::Alice.to_account_id(),
                    signature: AccountKeyring::Alice.sign(&msg3.encode()).into(),
                    metadata: ReceiptMetadata::default()
                },
            ],
            default_portfolio_vec(alice.did),
            10
        ));

        assert_affirms_pending(instruction_id, 1);
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Affirmed);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Pending);
        assert_leg_status(
            instruction_id,
            LegId(0),
            LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.to_account_id(), 0),
        );
        assert_leg_status(
            instruction_id,
            LegId(1),
            LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.to_account_id(), 1),
        );
        assert_locked_assets(&TICKER, &alice, 0);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();

        set_current_block_number(1);

        assert_affirm_instruction_with_zero_leg!(bob.origin(), instruction_id, bob.did);

        // Advances block
        next_block();
        assert_user_affirms(instruction_id, &alice, AffirmationStatus::Unknown);
        assert_user_affirms(instruction_id, &bob, AffirmationStatus::Unknown);
        assert_locked_assets(&TICKER, &alice, 0);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
    });
}

#[test]
fn overload_instruction() {
    test_with_cdd_provider(|eve| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let venue_counter = create_token_and_venue(TICKER, alice);
        let leg_limit =
            <TestStorage as pallet_settlement::Config>::MaxLegsInInstruction::get() as usize;

        let mut legs = vec![
            Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                asset: TICKER,
                amount: 1u128,
            };
            leg_limit + 1
        ];

        // Provide scope claim to multiple parties of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        assert_noop!(
            Settlement::add_instruction(
                alice.origin(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs.clone()
            ),
            Error::InstructionHasTooManyLegs
        );
        legs.truncate(leg_limit);
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            legs
        ));
    });
}

#[test]
fn encode_receipt() {
    ExtBuilder::default().build().execute_with(|| {
        let token_name = [0x01u8];
        let ticker = Ticker::try_from(&token_name[..]).unwrap();
        let msg1 = Receipt {
            receipt_uid: 0,
            from: PortfolioId::default_portfolio(
                IdentityId::try_from(
                    "did:poly:0600000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap(),
            ),
            to: PortfolioId::default_portfolio(
                IdentityId::try_from(
                    "did:poly:0600000000000000000000000000000000000000000000000000000000000000",
                )
                .unwrap()
                .into(),
            ),
            asset: ticker,
            amount: 100u128,
        };
        println!("{:?}", AccountKeyring::Alice.sign(&msg1.encode()));
    });
}

#[test]
fn test_weights_for_settlement_transaction() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            let alice = AccountKeyring::Alice.to_account_id();
            let (alice_signed, alice_did) = make_account_without_cdd(alice.clone()).unwrap();

            let bob = AccountKeyring::Bob.to_account_id();
            let (bob_signed, bob_did) = make_account_without_cdd(bob).unwrap();

            let dave = AccountKeyring::Dave.to_account_id();
            let (dave_signed, dave_did) = make_account_without_cdd(dave).unwrap();

            let venue_counter =
                create_token_and_venue(TICKER, User::existing(AccountKeyring::Alice));
            let instruction_id = Settlement::instruction_counter();

            let eve = AccountKeyring::Eve.to_account_id();

            // Get token Id.
            let ticker_id = Identity::get_token_did(&TICKER).unwrap();

            // Remove existing rules
            assert_ok!(ComplianceManager::remove_compliance_requirement(
                alice_signed.clone(),
                TICKER,
                1
            ));
            // Add claim rules for settlement
            assert_ok!(ComplianceManager::add_compliance_requirement(
                alice_signed.clone(),
                TICKER,
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(ticker_id.into())),
                        &[dave_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAbsent(Claim::BuyLockup(ticker_id.into())),
                        &[dave_did]
                    )
                ],
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(ticker_id.into())),
                        &[dave_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAnyOf(vec![
                            Claim::BuyLockup(ticker_id.into()),
                            Claim::KnowYourCustomer(ticker_id.into())
                        ]),
                        &[dave_did]
                    )
                ]
            ));

            // Providing claim to sender and receiver
            // For Alice
            assert_add_claim!(
                dave_signed.clone(),
                alice_did,
                Claim::Accredited(ticker_id.into())
            );
            // For Bob
            assert_add_claim!(
                dave_signed.clone(),
                bob_did,
                Claim::Accredited(ticker_id.into())
            );
            assert_add_claim!(
                dave_signed.clone(),
                bob_did,
                Claim::KnowYourCustomer(ticker_id.into())
            );

            // Provide scope claim as well to pass through the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], TICKER, eve);

            // Create instruction
            let legs = vec![Leg {
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: TICKER,
                amount: 100u128,
            }];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs.clone()
            ));

            assert_affirm_instruction_with_one_leg!(
                alice_signed.clone(),
                instruction_id,
                alice_did
            );
            set_current_block_number(100);
            assert_affirm_instruction_with_zero_leg!(bob_signed.clone(), instruction_id, bob_did);

            assert_ok!(
                Asset::_is_valid_transfer(
                    &TICKER,
                    PortfolioId::default_portfolio(alice_did),
                    PortfolioId::default_portfolio(bob_did),
                    100,
                ),
                ERC1400_TRANSFER_SUCCESS
            );
        });
}

#[test]
fn cross_portfolio_settlement() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let name = PortfolioName::from([42u8].to_vec());
        let num = Portfolio::next_portfolio_number(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to sender and receiver of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        // Instruction referencing a user defined portfolio is created
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::user_portfolio(bob.did, num),
                asset: TICKER,
                amount: amount
            }]
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged();
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(num, 0);

        assert_locked_assets(&TICKER, &alice, 0);
        set_current_block_number(10);

        // Approved by Alice
        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        assert_locked_assets(&TICKER, &alice, amount);
        // Bob fails to approve the instruction with a
        // different portfolio than the one specified in the instruction
        next_block();
        assert_noop!(
            Settlement::affirm_instruction(
                bob.origin(),
                instruction_id,
                default_portfolio_vec(bob.did),
                0
            ),
            Error::UnexpectedAffirmationStatus
        );

        next_block();
        // Bob approves the instruction with the correct portfolio
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            user_portfolio_vec(bob.did, num),
            0
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&TICKER, amount);
        bob.assert_balance_increased(&TICKER, amount);
        alice.assert_default_portfolio_bal_decreased(amount);
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(num, amount);
        assert_locked_assets(&TICKER, &alice, 0);
    });
}

#[test]
fn multiple_portfolio_settlement() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let name = PortfolioName::from([42u8].to_vec());
        let alice_num = Portfolio::next_portfolio_number(&alice.did);
        let bob_num = Portfolio::next_portfolio_number(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        assert_ok!(Portfolio::create_portfolio(alice.origin(), name.clone()));
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to sender and receiver of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        // An instruction is created with multiple legs referencing multiple portfolios
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![
                Leg {
                    from: PortfolioId::user_portfolio(alice.did, alice_num),
                    to: PortfolioId::default_portfolio(bob.did),
                    asset: TICKER,
                    amount: amount
                },
                Leg {
                    from: PortfolioId::default_portfolio(alice.did),
                    to: PortfolioId::user_portfolio(bob.did, bob_num),
                    asset: TICKER,
                    amount: amount
                }
            ]
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged();
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(bob_num, 0);
        assert_locked_assets(&TICKER, &alice, 0);

        // Alice approves the instruction from her default portfolio
        assert_affirm_instruction_with_one_leg!(alice.origin(), instruction_id, alice.did);

        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_unchanged();
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(bob_num, 0);
        assert_locked_assets(&TICKER, &alice, amount);

        // Alice tries to withdraw affirmation from multiple portfolios where only one has been affirmed.
        assert_noop!(
            Settlement::withdraw_affirmation(
                alice.origin(),
                instruction_id,
                vec![
                    PortfolioId::default_portfolio(alice.did),
                    PortfolioId::user_portfolio(alice.did, alice_num)
                ],
                2
            ),
            Error::UnexpectedAffirmationStatus
        );

        // Alice fails to approve the instruction from her user specified portfolio due to lack of funds
        assert_noop!(
            Settlement::affirm_instruction(
                alice.origin(),
                instruction_id,
                user_portfolio_vec(alice.did, alice_num),
                1
            ),
            Error::FailedToLockTokens
        );

        // Alice moves her funds to the correct portfolio
        assert_ok!(Portfolio::move_portfolio_funds(
            alice.origin(),
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(alice.did, alice_num),
            vec![MovePortfolioItem {
                ticker: TICKER,
                amount,
                memo: None
            }]
        ));
        set_current_block_number(15);
        // Alice is now able to approve the instruction with the user portfolio
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            user_portfolio_vec(alice.did, alice_num),
            1
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount);
        alice.assert_portfolio_bal(alice_num, amount);
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(bob_num, 0);
        assert_locked_assets(&TICKER, &alice, amount);
        assert_eq!(
            Portfolio::locked_assets(PortfolioId::user_portfolio(alice.did, alice_num), &TICKER),
            amount
        );

        // Bob approves the instruction with both of his portfolios in a single transaction
        let portfolios_vec = vec![
            PortfolioId::default_portfolio(bob.did),
            PortfolioId::user_portfolio(bob.did, bob_num),
        ];

        next_block();
        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            portfolios_vec,
            0
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&TICKER, amount * 2);
        bob.assert_balance_increased(&TICKER, amount * 2);
        alice.assert_default_portfolio_bal_decreased(amount * 2);
        bob.assert_default_portfolio_bal_increased(amount);
        bob.assert_portfolio_bal(bob_num, amount);
        assert_locked_assets(&TICKER, &alice, 0);
    });
}

#[test]
fn multiple_custodian_settlement() {
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);

        // Create portfolios
        let name = PortfolioName::from([42u8].to_vec());
        let alice_num = Portfolio::next_portfolio_number(&alice.did);
        let bob_num = Portfolio::next_portfolio_number(&bob.did);
        assert_ok!(Portfolio::create_portfolio(bob.origin(), name.clone()));
        assert_ok!(Portfolio::create_portfolio(alice.origin(), name.clone()));

        // Give custody of Bob's user portfolio to Alice
        let auth_id = Identity::add_auth(
            bob.did,
            Signatory::from(alice.did),
            AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(bob.did, bob_num)),
            None,
        );
        assert_ok!(Portfolio::accept_portfolio_custody(alice.origin(), auth_id));

        // Create a token
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount = 100u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to sender and receiver of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        assert_ok!(Portfolio::move_portfolio_funds(
            alice.origin(),
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(alice.did, alice_num),
            vec![MovePortfolioItem {
                ticker: TICKER,
                amount,
                memo: None
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
                Leg {
                    from: PortfolioId::user_portfolio(alice.did, alice_num),
                    to: PortfolioId::default_portfolio(bob.did),
                    asset: TICKER,
                    amount: amount
                },
                Leg {
                    from: PortfolioId::default_portfolio(alice.did),
                    to: PortfolioId::user_portfolio(bob.did, bob_num),
                    asset: TICKER,
                    amount: amount
                }
            ]
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount);
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(bob_num, 0);
        assert_locked_assets(&TICKER, &alice, 0);

        // Alice approves the instruction from both of her portfolios
        let portfolios_vec = vec![
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(alice.did, alice_num),
        ];
        set_current_block_number(10);
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            portfolios_vec.clone(),
            2
        ));
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        alice.assert_default_portfolio_bal_decreased(amount);
        bob.assert_default_portfolio_bal_unchanged();
        bob.assert_portfolio_bal(bob_num, 0);
        assert_locked_assets(&TICKER, &alice, amount);
        assert_eq!(
            Portfolio::locked_assets(PortfolioId::user_portfolio(alice.did, alice_num), &TICKER),
            amount
        );

        // Alice transfers custody of her portfolios but it won't affect any already approved instruction
        let auth_id2 = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(alice.did, alice_num)),
            None,
        );
        assert_ok!(Portfolio::accept_portfolio_custody(bob.origin(), auth_id2));

        // Bob fails to approve the instruction with both of his portfolios since he doesn't have custody for the second one
        let portfolios_bob = vec![
            PortfolioId::default_portfolio(bob.did),
            PortfolioId::user_portfolio(bob.did, bob_num),
        ];
        assert_noop!(
            Settlement::affirm_instruction(bob.origin(), instruction_id, portfolios_bob, 0),
            PortfolioError::UnauthorizedCustodian
        );

        next_block();
        // Bob can approve instruction from the portfolio he has custody of
        assert_affirm_instruction_with_zero_leg!(bob.origin(), instruction_id, bob.did);

        // Alice fails to deny the instruction from both her portfolios since she doesn't have the custody
        next_block();
        assert_noop!(
            Settlement::withdraw_affirmation(alice.origin(), instruction_id, portfolios_vec, 2),
            PortfolioError::UnauthorizedCustodian
        );

        // Alice can deny instruction from the portfolio she has custody of
        assert_ok!(Settlement::withdraw_affirmation(
            alice.origin(),
            instruction_id,
            default_portfolio_vec(alice.did),
            1
        ));
        assert_locked_assets(&TICKER, &alice, 0);

        // Alice can authorize instruction from remaining portfolios since she has the custody
        let portfolios_final = vec![
            PortfolioId::default_portfolio(alice.did),
            PortfolioId::user_portfolio(bob.did, bob_num),
        ];
        next_block();
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            instruction_id,
            portfolios_final,
            1
        ));

        // Instruction should've settled
        next_block();
        alice.assert_balance_decreased(&TICKER, amount * 2);
        bob.assert_balance_increased(&TICKER, amount * 2);
        alice.assert_default_portfolio_bal_decreased(amount * 2);
        bob.assert_default_portfolio_bal_increased(amount);
        bob.assert_portfolio_bal(bob_num, amount);
        assert_locked_assets(&TICKER, &alice, 0);
    });
}

#[test]
fn reject_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);

        let venue_counter = create_token_and_venue(TICKER, alice);
        let amount = 100u128;

        let reject_instruction = |user: &User, instruction_id| {
            Settlement::reject_instruction(
                user.origin(),
                instruction_id,
                PortfolioId::default_portfolio(user.did),
                1,
            )
        };

        let assert_user_affirmations = |instruction_id, alice_status, bob_status| {
            assert_eq!(
                Settlement::user_affirmations(
                    PortfolioId::default_portfolio(alice.did),
                    instruction_id
                ),
                alice_status
            );
            assert_eq!(
                Settlement::user_affirmations(
                    PortfolioId::default_portfolio(bob.did),
                    instruction_id
                ),
                bob_status
            );
        };

        let instruction_id = create_instruction(&alice, &bob, venue_counter, TICKER, amount);
        assert_user_affirmations(
            instruction_id,
            AffirmationStatus::Affirmed,
            AffirmationStatus::Pending,
        );
        next_block();
        // Try rejecting the instruction from a non-party account.
        assert_noop!(
            reject_instruction(&charlie, instruction_id),
            Error::UnauthorizedSigner
        );
        next_block();
        assert_ok!(reject_instruction(&alice, instruction_id,));
        next_block();
        // Instruction should've been deleted
        assert_user_affirmations(
            instruction_id,
            AffirmationStatus::Unknown,
            AffirmationStatus::Unknown,
        );

        // Test that the receiver can also reject the instruction
        let instruction_id2 = create_instruction(&alice, &bob, venue_counter, TICKER, amount);

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
    test_with_cdd_provider(|eve| {
        let mut alice = UserWithBalance::new(AccountKeyring::Alice, &[TICKER]);
        let mut bob = UserWithBalance::new(AccountKeyring::Bob, &[TICKER]);
        let venue_counter = create_token_and_venue(TICKER, alice.user);
        let instruction_id = Settlement::instruction_counter();
        let amount1 = 100u128;
        let amount2 = 50u128;
        alice.refresh_init_balances();
        bob.refresh_init_balances();

        // Provide scope claim to sender and receiver of the transaction.
        provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], TICKER, eve);

        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![
                Leg {
                    from: PortfolioId::default_portfolio(alice.did),
                    to: PortfolioId::default_portfolio(bob.did),
                    asset: TICKER,
                    amount: amount1
                },
                Leg {
                    from: PortfolioId::default_portfolio(bob.did),
                    to: PortfolioId::default_portfolio(alice.did),
                    asset: TICKER,
                    amount: 0
                },
                Leg {
                    from: PortfolioId::default_portfolio(alice.did),
                    to: PortfolioId::default_portfolio(bob.did),
                    asset: TICKER,
                    amount: amount2
                }
            ]
        ));

        assert_affirm_instruction!(alice.origin(), instruction_id, alice.did, 2);
        alice.assert_all_balances_unchanged();
        bob.assert_all_balances_unchanged();
        set_current_block_number(5);
        assert_affirm_instruction_with_one_leg!(bob.origin(), instruction_id, bob.did);

        // Advances the block no. to execute the instruction.
        let total_amount = amount1 + amount2;
        assert_eq!(Settlement::instruction_affirms_pending(instruction_id), 0);
        next_block();
        assert_eq!(
            pallet_settlement::InstructionLegs::iter_prefix(instruction_id).count(),
            0
        );

        // Ensure proper balance transfers
        alice.assert_balance_decreased(&TICKER, total_amount);
        bob.assert_balance_increased(&TICKER, total_amount);
    });
}

#[test]
fn reject_failed_instruction() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let venue_counter = create_token_and_venue(TICKER, alice);
        let amount = 100u128;

        let instruction_id = create_instruction(&alice, &bob, venue_counter, TICKER, amount);

        assert_ok!(Settlement::affirm_instruction(
            bob.origin(),
            instruction_id,
            default_portfolio_vec(bob.did),
            1
        ));

        // Go to next block to have the scheduled execution run and ensure it has failed.
        next_block();
        assert_instruction_status(instruction_id, InstructionStatus::Failed);

        // Reject instruction so that it is pruned on next execution.
        assert_ok!(Settlement::reject_instruction(
            bob.origin(),
            instruction_id,
            PortfolioId::default_portfolio(bob.did),
            1
        ));

        // Go to next block to have the scheduled execution run and ensure it has pruned the instruction.
        next_block();
        assert_instruction_status(instruction_id, InstructionStatus::Unknown);
    });
}

fn create_instruction(
    alice: &User,
    bob: &User,
    venue_counter: VenueId,
    ticker: Ticker,
    amount: u128,
) -> InstructionId {
    let instruction_id = Settlement::instruction_counter();
    set_current_block_number(10);
    assert_ok!(Settlement::add_and_affirm_instruction(
        alice.origin(),
        venue_counter,
        SettlementType::SettleOnAffirmation,
        None,
        None,
        vec![Leg {
            from: PortfolioId::default_portfolio(alice.did),
            to: PortfolioId::default_portfolio(bob.did),
            asset: ticker,
            amount
        }],
        default_portfolio_vec(alice.did)
    ));
    instruction_id
}

#[track_caller]
fn assert_instruction_details(
    instruction_id: InstructionId,
    details: Instruction<Moment, BlockNumber>,
) {
    assert_eq!(Settlement::instruction_details(instruction_id), details);
}

#[track_caller]
fn assert_instruction_status(instruction_id: InstructionId, status: InstructionStatus) {
    assert_eq!(
        Settlement::instruction_details(instruction_id).status,
        status
    );
}

#[track_caller]
fn assert_balance(ticker: &Ticker, user: &User, balance: Balance) {
    assert_eq!(Asset::balance_of(&ticker, user.did), balance);
}

#[track_caller]
fn assert_user_affirms(instruction_id: InstructionId, user: &User, status: AffirmationStatus) {
    assert_eq!(
        Settlement::user_affirmations(PortfolioId::default_portfolio(user.did), instruction_id),
        status
    );

    let affirms_received_status = match status {
        AffirmationStatus::Pending => AffirmationStatus::Unknown,
        AffirmationStatus::Affirmed => AffirmationStatus::Affirmed,
        _ => return,
    };

    assert_eq!(
        Settlement::affirms_received(instruction_id, PortfolioId::default_portfolio(user.did)),
        affirms_received_status
    );
}

#[track_caller]
fn assert_leg_status(instruction_id: InstructionId, leg: LegId, status: LegStatus<AccountId>) {
    assert_eq!(
        Settlement::instruction_leg_status(instruction_id, leg),
        status
    );
}

#[track_caller]
fn assert_affirms_pending(instruction_id: InstructionId, pending: u64) {
    assert_eq!(
        Settlement::instruction_affirms_pending(instruction_id),
        pending
    );
}

#[track_caller]
fn assert_locked_assets(ticker: &Ticker, user: &User, num_of_assets: Balance) {
    assert_eq!(
        Portfolio::locked_assets(PortfolioId::default_portfolio(user.did), ticker),
        num_of_assets
    );
}
