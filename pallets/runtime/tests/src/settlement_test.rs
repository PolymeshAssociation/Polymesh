use super::{
    storage::{
        default_portfolio_vec, make_account_without_cdd, provide_scope_claim_to_multiple_parties,
        register_keyring_account, user_portfolio_vec, TestStorage,
    },
    ExtBuilder,
};
use codec::{Decode, Encode};
use confidential_asset::{
    EncryptedAssetIdWrapper, InitializedAssetTxWrapper, MercatAccountId, PubAccountTxWrapper,
};
use core::convert::{TryFrom, TryInto};
use cryptography::{
    asset_proofs::{CommitmentWitness, ElgamalSecretKey},
    mercat::{
        account::{convert_asset_ids, AccountCreator},
        asset::AssetIssuer,
        transaction::{CtxMediator, CtxReceiver, CtxSender},
        Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptedAmount,
        EncryptionKeys, FinalizedTransferTx, InitializedTransferTx, PubAccount, PubAccountTx,
        SecAccount, TransferTransactionMediator, TransferTransactionReceiver,
        TransferTransactionSender,
    },
    AssetId,
};
use curve25519_dalek::scalar::Scalar;
use frame_support::dispatch::GetDispatchInfo;
use frame_support::{assert_noop, assert_ok};
use pallet_asset as asset;
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_confidential_asset as confidential_asset;
use pallet_identity as identity;
use pallet_portfolio::MovePortfolioItem;
use pallet_settlement::{
    self as settlement, weight_for, AuthorizationStatus, Call as SettlementCall, ConfidentialLeg,
    Instruction, InstructionStatus, Leg, LegStatus, MercatTxData, NonConfidentialLeg, Receipt,
    ReceiptDetails, SettlementType, VenueDetails, VenueType,
};
use polymesh_primitives::{
    AssetOwnershipRelation, AssetType, AuthorizationData, Base64Vec, Claim, Condition,
    ConditionType, FundingRoundName, IdentityId, PortfolioId, PortfolioName, SecurityToken,
    Signatory, Ticker,
};
use rand::{prelude::*, thread_rng};
use sp_core::sr25519::Public;
use sp_runtime::traits::Zero;
use sp_runtime::AnySignature;
use std::collections::HashMap;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type DidRecords = identity::DidRecords<TestStorage>;
type Settlement = settlement::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type Error = settlement::Error<TestStorage>;
type ConfidentialAsset = confidential_asset::Module<TestStorage>;

macro_rules! assert_add_claim {
    ($signer:expr, $target:expr, $claim:expr) => {
        assert_ok!(Identity::add_claim($signer, $target, $claim, None,));
    };
}

fn init(token_name: &[u8], ticker: Ticker, keyring: Public) -> u64 {
    create_token(token_name, ticker, keyring);
    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        Origin::signed(keyring),
        VenueDetails::default(),
        vec![keyring],
        VenueType::Other
    ));
    venue_counter
}

fn create_token(token_name: &[u8], ticker: Ticker, keyring: Public) {
    assert_ok!(Asset::create_asset(
        Origin::signed(keyring),
        token_name.into(),
        ticker,
        100_000,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
    assert_ok!(ComplianceManager::add_compliance_requirement(
        Origin::signed(keyring),
        ticker,
        vec![],
        vec![]
    ));
}

fn next_block() {
    let block_number = System::block_number() + 1;
    System::set_block_number(block_number);
    Settlement::on_initialize(block_number);
}

#[test]
fn venue_registration() {
    ExtBuilder::default()
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let venue_counter = Settlement::venue_counter();
            assert_ok!(Settlement::create_venue(
                alice_signed.clone(),
                VenueDetails::default(),
                vec![AccountKeyring::Alice.public(), AccountKeyring::Bob.public()],
                VenueType::Exchange
            ));
            let venue_info = Settlement::venue_info(venue_counter);
            assert_eq!(Settlement::venue_counter(), venue_counter + 1);
            assert_eq!(Settlement::user_venues(alice_did), [venue_counter]);
            assert_eq!(venue_info.creator, alice_did);
            assert_eq!(venue_info.instructions.len(), 0);
            assert_eq!(venue_info.details, VenueDetails::default());
            assert_eq!(venue_info.venue_type, VenueType::Exchange);
            assert_eq!(
                Settlement::venue_signers(venue_counter, AccountKeyring::Alice.public()),
                true
            );
            assert_eq!(
                Settlement::venue_signers(venue_counter, AccountKeyring::Bob.public()),
                true
            );
            assert_eq!(
                Settlement::venue_signers(venue_counter, AccountKeyring::Charlie.public()),
                false
            );

            // Creating a second venue
            assert_ok!(Settlement::create_venue(
                alice_signed.clone(),
                VenueDetails::default(),
                vec![AccountKeyring::Alice.public(), AccountKeyring::Bob.public()],
                VenueType::Exchange
            ));
            assert_eq!(
                Settlement::user_venues(alice_did),
                [venue_counter, venue_counter + 1]
            );

            // Editing venue details
            assert_ok!(Settlement::update_venue(
                alice_signed,
                venue_counter,
                Some([0x01].into()),
                None
            ));
            let venue_info = Settlement::venue_info(venue_counter);
            assert_eq!(venue_info.creator, alice_did);
            assert_eq!(venue_info.instructions.len(), 0);
            assert_eq!(venue_info.details, [0x01].into());
            assert_eq!(venue_info.venue_type, VenueType::Exchange);
        });
}

#[test]
fn basic_settlement() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100u128;
            let eve = AccountKeyring::Eve.public();

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount
                })]
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount
            );
        });
}

#[test]
fn create_and_authorize_instruction() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100u128;
            let eve = AccountKeyring::Eve.public();

            // Provide scope claim to both the parties of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            assert_ok!(Settlement::add_and_authorize_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount
                })],
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);

            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount
            );
        });
}

#[test]
fn overdraft_failure() {
    ExtBuilder::default()
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let _bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100_000_000u128;
            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount
                })]
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_noop!(
                Settlement::authorize_instruction(
                    alice_signed.clone(),
                    instruction_counter,
                    default_portfolio_vec(alice_did)
                ),
                Error::FailedToLockTokens
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
        });
}

#[test]
fn token_swap() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let token_name2 = b"ACME2";
            let ticker2 = Ticker::try_from(&token_name2[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let eve = AccountKeyring::Eve.public();
            init(token_name2, ticker2, AccountKeyring::Bob.public());

            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let alice_init_balance2 = Asset::balance_of(&ticker2, alice_did);
            let bob_init_balance2 = Asset::balance_of(&ticker2, bob_did);

            let amount = 100u128;
            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount,
                }),
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(bob_did),
                    to: PortfolioId::default_portfolio(alice_did),
                    asset: ticker2,
                    amount: amount,
                }),
            ];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                legs.clone()
            ));

            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );

            for i in 0..legs.len() {
                assert_eq!(
                    Settlement::instruction_legs(
                        instruction_counter,
                        u64::try_from(i).unwrap_or_default()
                    ),
                    legs[i]
                );
            }

            let instruction_details = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_counter,
                status: InstructionStatus::Pending,
                settlement_type: SettlementType::SettleOnAuthorization,
                created_at: Some(Timestamp::get()),
                valid_from: None,
            };
            assert_eq!(
                Settlement::instruction_details(instruction_counter),
                instruction_details
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                2
            );
            assert_eq!(
                Settlement::venue_info(venue_counter).instructions,
                vec![instruction_counter]
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            // Provide scope claim to parties involved in a instruction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker2, eve);

            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::unauthorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                2
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Instruction should've settled
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount
            );
            assert_eq!(
                Asset::balance_of(&ticker2, alice_did),
                alice_init_balance2 + amount
            );
            assert_eq!(
                Asset::balance_of(&ticker2, bob_did),
                bob_init_balance2 - amount
            );
        });
}

#[test]
fn claiming_receipt() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let token_name2 = b"ACME2";
            let ticker2 = Ticker::try_from(&token_name2[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let eve = AccountKeyring::Eve.public();
            init(token_name2, ticker2, AccountKeyring::Bob.public());

            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let alice_init_balance2 = Asset::balance_of(&ticker2, alice_did);
            let bob_init_balance2 = Asset::balance_of(&ticker2, bob_did);

            // Provide scope claims to multiple parties of a transactions.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker2, eve);

            let amount = 100u128;
            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount,
                }),
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(bob_did),
                    to: PortfolioId::default_portfolio(alice_did),
                    asset: ticker2,
                    amount: amount,
                }),
            ];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                legs.clone()
            ));

            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );

            for i in 0..legs.len() {
                assert_eq!(
                    Settlement::instruction_legs(
                        instruction_counter,
                        u64::try_from(i).unwrap_or_default()
                    ),
                    legs[i]
                );
            }

            let instruction_details = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_counter,
                status: InstructionStatus::Pending,
                settlement_type: SettlementType::SettleOnAuthorization,
                created_at: Some(Timestamp::get()),
                valid_from: None,
            };
            assert_eq!(
                Settlement::instruction_details(instruction_counter),
                instruction_details
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                2
            );
            assert_eq!(
                Settlement::venue_info(venue_counter).instructions,
                vec![instruction_counter]
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            let msg = Receipt {
                receipt_uid: 0,
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker,
                amount: amount,
            };

            assert_noop!(
                Settlement::claim_receipt(
                    alice_signed.clone(),
                    instruction_counter,
                    ReceiptDetails {
                        receipt_uid: 0,
                        leg_id: 0,
                        signer: AccountKeyring::Alice.public(),
                        signature: OffChainSignature::from(
                            AccountKeyring::Alice.sign(&msg.encode())
                        )
                    }
                ),
                Error::LegNotPending
            );

            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            let msg2 = Receipt {
                receipt_uid: 0,
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(alice_did),
                asset: ticker,
                amount: amount,
            };

            assert_noop!(
                Settlement::claim_receipt(
                    alice_signed.clone(),
                    instruction_counter,
                    ReceiptDetails {
                        receipt_uid: 0,
                        leg_id: 0,
                        signer: AccountKeyring::Alice.public(),
                        signature: OffChainSignature::from(
                            AccountKeyring::Alice.sign(&msg2.encode())
                        )
                    }
                ),
                Error::InvalidSignature
            );

            // Claiming, unclaiming and claiming receipt
            assert_ok!(Settlement::claim_receipt(
                alice_signed.clone(),
                instruction_counter,
                ReceiptDetails {
                    receipt_uid: 0,
                    leg_id: 0,
                    signer: AccountKeyring::Alice.public(),
                    signature: OffChainSignature::from(AccountKeyring::Alice.sign(&msg.encode()))
                }
            ));

            assert_eq!(
                Settlement::receipts_used(AccountKeyring::Alice.public(), 0),
                true
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.public(), 0)
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::unclaim_receipt(
                alice_signed.clone(),
                instruction_counter,
                0
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::claim_receipt(
                alice_signed.clone(),
                instruction_counter,
                ReceiptDetails {
                    receipt_uid: 0,
                    leg_id: 0,
                    signer: AccountKeyring::Alice.public(),
                    signature: OffChainSignature::from(AccountKeyring::Alice.sign(&msg.encode()))
                }
            ));

            assert_eq!(
                Settlement::receipts_used(AccountKeyring::Alice.public(), 0),
                true
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.public(), 0)
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Instruction should've settled
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Asset::balance_of(&ticker2, alice_did),
                alice_init_balance2 + amount
            );
            assert_eq!(
                Asset::balance_of(&ticker2, bob_did),
                bob_init_balance2 - amount
            );
        });
}

#[test]
fn settle_on_block() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let token_name2 = b"ACME2";
            let ticker2 = Ticker::try_from(&token_name2[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            init(token_name2, ticker2, AccountKeyring::Bob.public());
            let block_number = System::block_number() + 1;
            let eve = AccountKeyring::Eve.public();
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let alice_init_balance2 = Asset::balance_of(&ticker2, alice_did);
            let bob_init_balance2 = Asset::balance_of(&ticker2, bob_did);

            let amount = 100u128;
            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount,
                }),
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(bob_did),
                    to: PortfolioId::default_portfolio(alice_did),
                    asset: ticker2,
                    amount: amount,
                }),
            ];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                legs.clone()
            ));

            assert_eq!(
                Settlement::scheduled_instructions(block_number),
                vec![instruction_counter]
            );

            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );

            for i in 0..legs.len() {
                assert_eq!(
                    Settlement::instruction_legs(
                        instruction_counter,
                        u64::try_from(i).unwrap_or_default()
                    ),
                    legs[i]
                );
            }

            let instruction_details = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_counter,
                status: InstructionStatus::Pending,
                settlement_type: SettlementType::SettleOnBlock(block_number),
                created_at: Some(Timestamp::get()),
                valid_from: None,
            };
            assert_eq!(
                Settlement::instruction_details(instruction_counter),
                instruction_details
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                2
            );
            assert_eq!(
                Settlement::venue_info(venue_counter).instructions,
                vec![instruction_counter]
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            // Before authorization need to provide the scope claim for both the parties of a transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker2, eve);

            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                0
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(bob_did), &ticker2),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            next_block();

            // Instruction should've settled
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount
            );
            assert_eq!(
                Asset::balance_of(&ticker2, alice_did),
                alice_init_balance2 + amount
            );
            assert_eq!(
                Asset::balance_of(&ticker2, bob_did),
                bob_init_balance2 - amount
            );
        });
}

#[test]
fn failed_execution() {
    ExtBuilder::default()
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let token_name2 = b"ACME2";
            let ticker2 = Ticker::try_from(&token_name2[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            init(token_name2, ticker2, AccountKeyring::Bob.public());
            assert_ok!(ComplianceManager::reset_asset_compliance(
                Origin::signed(AccountKeyring::Bob.public()),
                ticker2,
            ));
            let block_number = System::block_number() + 1;

            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let alice_init_balance2 = Asset::balance_of(&ticker2, alice_did);
            let bob_init_balance2 = Asset::balance_of(&ticker2, bob_did);

            let amount = 100u128;
            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount,
                }),
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(bob_did),
                    to: PortfolioId::default_portfolio(alice_did),
                    asset: ticker2,
                    amount: amount,
                }),
            ];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                legs.clone()
            ));

            assert_eq!(
                Settlement::scheduled_instructions(block_number),
                vec![instruction_counter]
            );

            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );

            for i in 0..legs.len() {
                assert_eq!(
                    Settlement::instruction_legs(
                        instruction_counter,
                        u64::try_from(i).unwrap_or_default()
                    ),
                    legs[i]
                );
            }

            let instruction_details = Instruction {
                instruction_id: instruction_counter,
                venue_id: venue_counter,
                status: InstructionStatus::Pending,
                settlement_type: SettlementType::SettleOnBlock(block_number),
                created_at: Some(Timestamp::get()),
                valid_from: None,
            };
            assert_eq!(
                Settlement::instruction_details(instruction_counter),
                instruction_details
            );
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                2
            );
            assert_eq!(
                Settlement::venue_info(venue_counter).instructions,
                vec![instruction_counter]
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::PendingTokenLock
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));
            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                0
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::ExecutionPending
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(bob_did), &ticker2),
                amount
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            next_block();

            // Instruction should've settled
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(bob_did), &ticker2),
                0
            );
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);
        });
}

#[test]
fn venue_filtering() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let block_number = System::block_number() + 1;
            let instruction_counter = Settlement::instruction_counter();
            let eve = AccountKeyring::Eve.public();

            // provide scope claim.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            let legs = vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker,
                amount: 10,
            })];
            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                legs.clone()
            ));
            assert_ok!(Settlement::set_venue_filtering(
                alice_signed.clone(),
                ticker,
                true
            ));
            assert_noop!(
                Settlement::add_instruction(
                    alice_signed.clone(),
                    venue_counter,
                    SettlementType::SettleOnBlock(block_number),
                    None,
                    legs.clone()
                ),
                Error::UnauthorizedVenue
            );
            assert_ok!(Settlement::allow_venues(
                alice_signed.clone(),
                ticker,
                vec![venue_counter]
            ));
            assert_ok!(Settlement::add_and_authorize_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number + 1),
                None,
                legs.clone(),
                default_portfolio_vec(alice_did)
            ));
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter + 1,
                default_portfolio_vec(bob_did)
            ));
            next_block();
            assert_eq!(Asset::balance_of(&ticker, bob_did), 10);
            assert_ok!(Settlement::disallow_venues(
                alice_signed.clone(),
                ticker,
                vec![venue_counter]
            ));
            next_block();
            // Second instruction fails to settle due to venue being not whitelisted
            assert_eq!(Asset::balance_of(&ticker, bob_did), 10);
        });
}

#[test]
fn basic_fuzzing() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let charlie_signed = Origin::signed(AccountKeyring::Charlie.public());
            let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
            let dave_signed = Origin::signed(AccountKeyring::Dave.public());
            let dave_did = register_keyring_account(AccountKeyring::Dave).unwrap();
            let venue_counter = Settlement::venue_counter();
            let eve = AccountKeyring::Eve.public();
            assert_ok!(Settlement::create_venue(
                Origin::signed(AccountKeyring::Alice.public()),
                VenueDetails::default(),
                vec![AccountKeyring::Alice.public()],
                VenueType::Other
            ));
            let mut tickers = Vec::with_capacity(40);
            let mut balances = HashMap::with_capacity(320);
            let dids = vec![alice_did, bob_did, charlie_did, dave_did];
            let signers = vec![
                alice_signed.clone(),
                bob_signed.clone(),
                charlie_signed.clone(),
                dave_signed.clone(),
            ];

            for i in 0..10 {
                let mut token_name = [123u8 + u8::try_from(i * 4 + 0).unwrap()];
                tickers.push(Ticker::try_from(&token_name[..]).unwrap());
                create_token(
                    &token_name[..],
                    tickers[i * 4 + 0],
                    AccountKeyring::Alice.public(),
                );

                token_name = [123u8 + u8::try_from(i * 4 + 1).unwrap()];
                tickers.push(Ticker::try_from(&token_name[..]).unwrap());
                create_token(
                    &token_name[..],
                    tickers[i * 4 + 1],
                    AccountKeyring::Bob.public(),
                );

                token_name = [123u8 + u8::try_from(i * 4 + 2).unwrap()];
                tickers.push(Ticker::try_from(&token_name[..]).unwrap());
                create_token(
                    &token_name[..],
                    tickers[i * 4 + 2],
                    AccountKeyring::Charlie.public(),
                );

                token_name = [123u8 + u8::try_from(i * 4 + 3).unwrap()];
                tickers.push(Ticker::try_from(&token_name[..]).unwrap());
                create_token(
                    &token_name[..],
                    tickers[i * 4 + 3],
                    AccountKeyring::Dave.public(),
                );
            }

            let block_number = System::block_number() + 1;
            let instruction_counter = Settlement::instruction_counter();

            // initialize balances
            for i in 0..10 {
                for j in 0..4 {
                    balances.insert((tickers[i * 4 + j], dids[j], "init").encode(), 100_000);
                    balances.insert((tickers[i * 4 + j], dids[j], "final").encode(), 100_000);
                    for k in 0..4 {
                        if j == k {
                            continue;
                        }
                        balances.insert((tickers[i * 4 + j], dids[k], "init").encode(), 0);
                        balances.insert((tickers[i * 4 + j], dids[k], "final").encode(), 0);
                    }
                }
            }

            let mut legs = Vec::with_capacity(100);
            let mut receipts = Vec::with_capacity(100);
            let mut receipt_legs = HashMap::with_capacity(100);
            for i in 0..10 {
                for j in 0..4 {
                    let mut final_i = 100_000;
                    balances.insert((tickers[i * 4 + j], dids[j], "init").encode(), 100_000);
                    for k in 0..4 {
                        if j == k {
                            continue;
                        }
                        balances.insert((tickers[i * 4 + j], dids[k], "init").encode(), 0);
                        if random() {
                            // This leg should happen
                            if random() {
                                // Receipt to be claimed
                                balances.insert((tickers[i * 4 + j], dids[k], "final").encode(), 0);
                                receipts.push(Receipt {
                                    receipt_uid: u64::try_from(k * 1000 + i * 4 + j).unwrap(),
                                    from: PortfolioId::default_portfolio(dids[j]),
                                    to: PortfolioId::default_portfolio(dids[k]),
                                    asset: tickers[i * 4 + j],
                                    amount: 1u128,
                                });
                                receipt_legs.insert(receipts.last().unwrap().encode(), legs.len());
                            } else {
                                balances.insert((tickers[i * 4 + j], dids[k], "final").encode(), 1);
                                final_i -= 1;
                            }
                            // Provide scope claim for all the dids
                            provide_scope_claim_to_multiple_parties(
                                &[dids[j], dids[k]],
                                tickers[i * 4 + j],
                                eve,
                            );
                            legs.push(Leg::NonConfidentialLeg(NonConfidentialLeg {
                                from: PortfolioId::default_portfolio(dids[j]),
                                to: PortfolioId::default_portfolio(dids[k]),
                                asset: tickers[i * 4 + j],
                                amount: 1,
                            }));
                            if legs.len() >= 100 {
                                break;
                            }
                        }
                    }
                    balances.insert((tickers[i * 4 + j], dids[j], "final").encode(), final_i);
                    if legs.len() >= 100 {
                        break;
                    }
                }
                if legs.len() >= 100 {
                    break;
                }
            }

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnBlock(block_number),
                None,
                legs
            ));

            // Authorize instructions and do a few authorize/unauthorize in between
            for (i, signer) in signers.clone().iter().enumerate() {
                for _ in 0..2 {
                    if random() {
                        assert_ok!(Settlement::authorize_instruction(
                            signer.clone(),
                            instruction_counter,
                            default_portfolio_vec(dids[i])
                        ));
                        assert_ok!(Settlement::unauthorize_instruction(
                            signer.clone(),
                            instruction_counter,
                            default_portfolio_vec(dids[i])
                        ));
                    }
                }
                assert_ok!(Settlement::authorize_instruction(
                    signer.clone(),
                    instruction_counter,
                    default_portfolio_vec(dids[i])
                ));
            }

            // Claim receipts and do a few claim/unclaims in between
            for receipt in receipts {
                let leg_num =
                    u64::try_from(*receipt_legs.get(&(receipt.encode())).unwrap()).unwrap();
                let signer = &signers[dids
                    .iter()
                    .position(|&from| PortfolioId::default_portfolio(from) == receipt.from)
                    .unwrap()];
                for _ in 0..2 {
                    if random() {
                        assert_ok!(Settlement::claim_receipt(
                            signer.clone(),
                            instruction_counter,
                            ReceiptDetails {
                                receipt_uid: receipt.receipt_uid,
                                leg_id: leg_num,
                                signer: AccountKeyring::Alice.public(),
                                signature: OffChainSignature::from(
                                    AccountKeyring::Alice.sign(&receipt.encode())
                                )
                            }
                        ));
                        assert_ok!(Settlement::unclaim_receipt(
                            signer.clone(),
                            instruction_counter,
                            leg_num
                        ));
                    }
                }
                assert_ok!(Settlement::claim_receipt(
                    signer.clone(),
                    instruction_counter,
                    ReceiptDetails {
                        receipt_uid: receipt.receipt_uid,
                        leg_id: leg_num,
                        signer: AccountKeyring::Alice.public(),
                        signature: OffChainSignature::from(
                            AccountKeyring::Alice.sign(&receipt.encode())
                        )
                    }
                ));
            }

            let fail: bool = random();
            if fail {
                let mut rng = thread_rng();
                let i = rng.gen_range(0, 4);
                assert_ok!(Settlement::unauthorize_instruction(
                    signers[i].clone(),
                    instruction_counter,
                    default_portfolio_vec(dids[i])
                ));
            }

            next_block();

            for i in 0..40 {
                for j in 0..4 {
                    assert_eq!(
                        Portfolio::locked_assets(
                            PortfolioId::default_portfolio(dids[j]),
                            &tickers[i]
                        ),
                        0
                    );
                    if fail {
                        assert_eq!(
                            Asset::balance_of(&tickers[i], dids[j]),
                            u128::try_from(
                                *balances
                                    .get(&(tickers[i], dids[j], "init").encode())
                                    .unwrap()
                            )
                            .unwrap()
                        );
                    } else {
                        assert_eq!(
                            Asset::balance_of(&tickers[i], dids[j]),
                            u128::try_from(
                                *balances
                                    .get(&(tickers[i], dids[j], "final").encode())
                                    .unwrap()
                            )
                            .unwrap()
                        );
                    }
                }
            }
        });
}

#[test]
fn claim_multiple_receipts_during_authorization() {
    ExtBuilder::default()
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let token_name2 = b"ACME2";
            let ticker2 = Ticker::try_from(&token_name2[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            init(token_name2, ticker2, AccountKeyring::Bob.public());

            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let alice_init_balance2 = Asset::balance_of(&ticker2, alice_did);
            let bob_init_balance2 = Asset::balance_of(&ticker2, bob_did);

            let amount = 100u128;
            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: amount,
                }),
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker2,
                    amount: amount,
                }),
            ];

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                legs.clone()
            ));

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            let msg1 = Receipt {
                receipt_uid: 0,
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker,
                amount: amount,
            };
            let msg2 = Receipt {
                receipt_uid: 0,
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker2,
                amount: amount,
            };
            let msg3 = Receipt {
                receipt_uid: 1,
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker2,
                amount: amount,
            };

            assert_noop!(
                Settlement::authorize_with_receipts(
                    alice_signed.clone(),
                    instruction_counter,
                    vec![
                        ReceiptDetails {
                            receipt_uid: 0,
                            leg_id: 0,
                            signer: AccountKeyring::Alice.public(),
                            signature: OffChainSignature::from(
                                AccountKeyring::Alice.sign(&msg1.encode())
                            )
                        },
                        ReceiptDetails {
                            receipt_uid: 0,
                            leg_id: 0,
                            signer: AccountKeyring::Alice.public(),
                            signature: OffChainSignature::from(
                                AccountKeyring::Alice.sign(&msg2.encode())
                            )
                        },
                    ],
                    default_portfolio_vec(alice_did)
                ),
                Error::ReceiptAlreadyClaimed
            );

            assert_ok!(Settlement::authorize_with_receipts(
                alice_signed.clone(),
                instruction_counter,
                vec![
                    ReceiptDetails {
                        receipt_uid: 0,
                        leg_id: 0,
                        signer: AccountKeyring::Alice.public(),
                        signature: OffChainSignature::from(
                            AccountKeyring::Alice.sign(&msg1.encode())
                        )
                    },
                    ReceiptDetails {
                        receipt_uid: 1,
                        leg_id: 1,
                        signer: AccountKeyring::Alice.public(),
                        signature: OffChainSignature::from(
                            AccountKeyring::Alice.sign(&msg3.encode())
                        )
                    },
                ],
                default_portfolio_vec(alice_did)
            ));

            assert_eq!(
                Settlement::instruction_auths_pending(instruction_counter),
                1
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Pending
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(alice_did)
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::auths_received(
                    instruction_counter,
                    PortfolioId::default_portfolio(bob_did)
                ),
                AuthorizationStatus::Unknown
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 0),
                LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.public(), 0)
            );
            assert_eq!(
                Settlement::instruction_leg_status(instruction_counter, 1),
                LegStatus::ExecutionToBeSkipped(AccountKeyring::Alice.public(), 1)
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);

            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Instruction should've settled
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(alice_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Settlement::user_auths(
                    PortfolioId::default_portfolio(bob_did),
                    instruction_counter
                ),
                AuthorizationStatus::Authorized
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(Asset::balance_of(&ticker2, alice_did), alice_init_balance2);
            assert_eq!(Asset::balance_of(&ticker2, bob_did), bob_init_balance2);
        });
}

#[test]
fn overload_settle_on_block() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let eve = AccountKeyring::Eve.public();
            let block_number = System::block_number() + 1;

            let legs = vec![
                Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    asset: ticker,
                    amount: 1u128,
                });
                500
            ];

            // Provide scope claim to multiple parties of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            for _ in 0..2 {
                assert_ok!(Settlement::add_instruction(
                    alice_signed.clone(),
                    venue_counter,
                    SettlementType::SettleOnBlock(block_number),
                    None,
                    legs.clone()
                ));
                assert_ok!(Settlement::add_instruction(
                    alice_signed.clone(),
                    venue_counter,
                    SettlementType::SettleOnBlock(block_number + 1),
                    None,
                    legs.clone()
                ));
            }

            for i in &[0u64, 1, 3] {
                assert_ok!(Settlement::authorize_instruction(
                    alice_signed.clone(),
                    instruction_counter + i,
                    default_portfolio_vec(alice_did)
                ));
                assert_ok!(Settlement::authorize_instruction(
                    bob_signed.clone(),
                    instruction_counter + i,
                    default_portfolio_vec(bob_did)
                ));
            }

            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);

            assert_eq!(
                Settlement::scheduled_instructions(block_number),
                vec![instruction_counter, instruction_counter + 2]
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 1),
                vec![instruction_counter + 1, instruction_counter + 3]
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 2).len(),
                0
            );

            next_block();
            // First Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - 500
            );
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance + 500);
            assert_eq!(Settlement::scheduled_instructions(block_number).len(), 0);
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 1),
                vec![
                    instruction_counter + 1,
                    instruction_counter + 3,
                    instruction_counter + 2
                ]
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 2).len(),
                0
            );

            next_block();
            // Second instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - 1000
            );
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance + 1000);
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 1).len(),
                0
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 2),
                vec![instruction_counter + 3, instruction_counter + 2]
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 3).len(),
                0
            );

            next_block();
            // Fourth instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - 1500
            );
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance + 1500);
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 2).len(),
                0
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 3),
                vec![instruction_counter + 2]
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 4).len(),
                0
            );

            assert_noop!(
                Settlement::authorize_instruction(
                    alice_signed.clone(),
                    instruction_counter + 2,
                    default_portfolio_vec(alice_did)
                ),
                Error::InstructionSettleBlockPassed
            );
            next_block();
            // Third instruction should've settled (Failed due to missing auth)
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - 1500
            );
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance + 1500);
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 3).len(),
                0
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 4).len(),
                0
            );
            assert_eq!(
                Settlement::scheduled_instructions(block_number + 5).len(),
                0
            );
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
        .cdd_providers(vec![AccountKeyring::Dave.public()])
        .set_max_legs_allowed(5) // set maximum no. of legs allowed for an instruction.
        .set_max_tms_allowed(4) // set maximum no. of tms an asset can have.
        .build()
        .execute_with(|| {
            let alice = AccountKeyring::Alice.public();
            let (alice_signed, alice_did) = make_account_without_cdd(alice).unwrap();

            let bob = AccountKeyring::Bob.public();
            let (bob_signed, bob_did) = make_account_without_cdd(bob).unwrap();

            let eve = AccountKeyring::Eve.public();
            let (eve_signed, eve_did) = make_account_without_cdd(eve).unwrap();

            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();

            let venue_counter = init(token_name, ticker, alice);
            let instruction_counter = Settlement::instruction_counter();

            let dave = AccountKeyring::Dave.public();

            // Get token Id.
            let ticker_id = Identity::get_token_did(&ticker).unwrap();

            // Remove existing rules
            assert_ok!(ComplianceManager::remove_compliance_requirement(
                alice_signed.clone(),
                ticker,
                1
            ));
            // Add claim rules for settlement
            assert_ok!(ComplianceManager::add_compliance_requirement(
                alice_signed.clone(),
                ticker,
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(ticker_id.into())),
                        &[eve_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAbsent(Claim::BuyLockup(ticker_id.into())),
                        &[eve_did]
                    )
                ],
                vec![
                    Condition::from_dids(
                        ConditionType::IsPresent(Claim::Accredited(ticker_id.into())),
                        &[eve_did]
                    ),
                    Condition::from_dids(
                        ConditionType::IsAnyOf(vec![
                            Claim::BuyLockup(ticker_id.into()),
                            Claim::KnowYourCustomer(ticker_id.into())
                        ]),
                        &[eve_did]
                    )
                ]
            ));

            // Providing claim to sender and receiver
            // For Alice
            assert_add_claim!(
                eve_signed.clone(),
                alice_did,
                Claim::Accredited(ticker_id.into())
            );
            // For Bob
            assert_add_claim!(
                eve_signed.clone(),
                bob_did,
                Claim::Accredited(ticker_id.into())
            );
            assert_add_claim!(
                eve_signed.clone(),
                bob_did,
                Claim::KnowYourCustomer(ticker_id.into())
            );

            // Provide scope claim as well to pass through the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, dave);

            // Create instruction
            let legs = vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                from: PortfolioId::default_portfolio(alice_did),
                to: PortfolioId::default_portfolio(bob_did),
                asset: ticker,
                amount: 100u128,
            })];

            let weight_to_add_instruction = SettlementCall::<TestStorage>::add_instruction(
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                legs.clone(),
            )
            .get_dispatch_info()
            .weight;

            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                legs.clone()
            ));

            assert_eq!(
                weight_to_add_instruction,
                weight_for::weight_for_instruction_creation::<TestStorage>(legs.len())
            );

            // Authorize instruction by Alice first and check for weight.
            let weight_for_authorize_instruction_1 =
                SettlementCall::<TestStorage>::authorize_instruction(
                    instruction_counter,
                    default_portfolio_vec(alice_did),
                )
                .get_dispatch_info()
                .weight;
            let result_authorize_instruction_1 = Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did),
            );
            assert_ok!(result_authorize_instruction_1);
            assert_eq!(
                weight_for::weight_for_authorize_instruction::<TestStorage>(),
                weight_for_authorize_instruction_1
                    - weight_for::weight_for_transfer::<TestStorage>()
            );

            let weight_for_authorize_instruction_2 =
                SettlementCall::<TestStorage>::authorize_instruction(
                    instruction_counter,
                    default_portfolio_vec(bob_did),
                )
                .get_dispatch_info()
                .weight;
            let result_authorize_instruction_2 = Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did),
            );
            assert_ok!(result_authorize_instruction_2);
            assert_eq!(Asset::balance_of(ticker, bob_did), 100);
            let acutal_weight = result_authorize_instruction_2
                .unwrap()
                .actual_weight
                .unwrap();
            let (transfer_result, _weight_for_is_valid_transfer) = Asset::_is_valid_transfer(
                &ticker,
                alice,
                PortfolioId::default_portfolio(alice_did),
                PortfolioId::default_portfolio(bob_did),
                100,
            )
            .unwrap();
            assert_eq!(transfer_result, 81);
            assert!(weight_for_authorize_instruction_2 > acutal_weight);
        });
}

#[test]
fn cross_portfolio_settlement() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let name = PortfolioName::from([42u8].to_vec());
            let num = Portfolio::next_portfolio_number(&bob_did);
            assert_ok!(Portfolio::create_portfolio(
                bob_signed.clone(),
                name.clone()
            ));
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100u128;
            let eve = AccountKeyring::Eve.public();

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            // Instruction referencing a user defined portfolio is created
            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![Leg::NonConfidentialLeg(NonConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::user_portfolio(bob_did, num),
                    asset: ticker,
                    amount: amount
                })]
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(Portfolio::user_portfolio_balance(bob_did, num, &ticker), 0,);
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            // Approved by Alice
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(Portfolio::user_portfolio_balance(bob_did, num, &ticker), 0,);
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            // Bob fails to approve the instruction with a
            // different portfolio than the one specified in the instruction
            assert_noop!(
                Settlement::authorize_instruction(
                    bob_signed.clone(),
                    instruction_counter,
                    default_portfolio_vec(bob_did),
                ),
                Error::NoPendingAuth
            );

            // Bob approves the instruction with the correct portfolio
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                user_portfolio_vec(bob_did, num)
            ));

            // Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, num, &ticker),
                amount,
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
        });
}

#[test]
fn multiple_portfolio_settlement() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let name = PortfolioName::from([42u8].to_vec());
            let alice_num = Portfolio::next_portfolio_number(&alice_did);
            let bob_num = Portfolio::next_portfolio_number(&bob_did);
            assert_ok!(Portfolio::create_portfolio(
                bob_signed.clone(),
                name.clone()
            ));
            assert_ok!(Portfolio::create_portfolio(
                alice_signed.clone(),
                name.clone()
            ));
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100u128;
            let eve = AccountKeyring::Eve.public();

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            // An instruction is created with multiple legs referencing multiple portfolios
            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![
                    Leg::NonConfidentialLeg(NonConfidentialLeg {
                        from: PortfolioId::user_portfolio(alice_did, alice_num),
                        to: PortfolioId::default_portfolio(bob_did),
                        asset: ticker,
                        amount: amount
                    }),
                    Leg::NonConfidentialLeg(NonConfidentialLeg {
                        from: PortfolioId::default_portfolio(alice_did),
                        to: PortfolioId::user_portfolio(bob_did, bob_num),
                        asset: ticker,
                        amount: amount
                    })
                ]
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                0,
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            // Alice approves the instruction from her default portfolio
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                0
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );

            // Alice fails to approve the instruction from her user specified portfolio due to lack of funds
            assert_noop!(
                Settlement::authorize_instruction(
                    alice_signed.clone(),
                    instruction_counter,
                    user_portfolio_vec(alice_did, alice_num)
                ),
                Error::FailedToLockTokens
            );

            // Alice moves her funds to the correct portfolio
            assert_ok!(Portfolio::move_portfolio_funds(
                alice_signed.clone(),
                PortfolioId::default_portfolio(alice_did),
                PortfolioId::user_portfolio(alice_did, alice_num),
                vec![MovePortfolioItem { ticker, amount }]
            ));

            // Alice is now able to approve the instruction with the user portfolio
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                user_portfolio_vec(alice_did, alice_num)
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(alice_did, alice_num, &ticker),
                amount,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                0
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );
            assert_eq!(
                Portfolio::locked_assets(
                    PortfolioId::user_portfolio(alice_did, alice_num),
                    &ticker
                ),
                amount
            );

            // Bob approves the instruction with both of his portfolios in a single transaction
            let portfolios_vec = vec![
                PortfolioId::default_portfolio(bob_did),
                PortfolioId::user_portfolio(bob_did, bob_num),
            ];
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                portfolios_vec
            ));

            // Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount * 2
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount * 2
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount * 2,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance + amount,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                amount,
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
        });
}

#[test]
fn multiple_custodian_settlement() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

            // Create portfolios
            let name = PortfolioName::from([42u8].to_vec());
            let alice_num = Portfolio::next_portfolio_number(&alice_did);
            let bob_num = Portfolio::next_portfolio_number(&bob_did);
            assert_ok!(Portfolio::create_portfolio(
                bob_signed.clone(),
                name.clone()
            ));
            assert_ok!(Portfolio::create_portfolio(
                alice_signed.clone(),
                name.clone()
            ));

            // Give custody of Bob's user portfolio to Alice
            let auth_id = Identity::add_auth(
                bob_did,
                Signatory::from(alice_did),
                AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(bob_did, bob_num)),
                None,
            );
            assert_ok!(Identity::accept_authorization(
                alice_signed.clone(),
                auth_id
            ));

            // Create a token
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            let venue_counter = init(token_name, ticker, AccountKeyring::Alice.public());
            let instruction_counter = Settlement::instruction_counter();
            let alice_init_balance = Asset::balance_of(&ticker, alice_did);
            let bob_init_balance = Asset::balance_of(&ticker, bob_did);
            let amount = 100u128;
            let eve = AccountKeyring::Eve.public();

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, eve);

            assert_ok!(Portfolio::move_portfolio_funds(
                alice_signed.clone(),
                PortfolioId::default_portfolio(alice_did),
                PortfolioId::user_portfolio(alice_did, alice_num),
                vec![MovePortfolioItem { ticker, amount }]
            ));

            // An instruction is created with multiple legs referencing multiple portfolios
            assert_ok!(Settlement::add_instruction(
                alice_signed.clone(),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![
                    Leg::NonConfidentialLeg(NonConfidentialLeg {
                        from: PortfolioId::user_portfolio(alice_did, alice_num),
                        to: PortfolioId::default_portfolio(bob_did),
                        asset: ticker,
                        amount: amount
                    }),
                    Leg::NonConfidentialLeg(NonConfidentialLeg {
                        from: PortfolioId::default_portfolio(alice_did),
                        to: PortfolioId::user_portfolio(bob_did, bob_num),
                        asset: ticker,
                        amount: amount
                    })
                ]
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                0,
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            // Alice approves the instruction from both of her portfolios
            let portfolios_vec = vec![
                PortfolioId::default_portfolio(alice_did),
                PortfolioId::user_portfolio(alice_did, alice_num),
            ];
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                portfolios_vec.clone()
            ));
            assert_eq!(Asset::balance_of(&ticker, alice_did), alice_init_balance);
            assert_eq!(Asset::balance_of(&ticker, bob_did), bob_init_balance);
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                0
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                amount
            );
            assert_eq!(
                Portfolio::locked_assets(
                    PortfolioId::user_portfolio(alice_did, alice_num),
                    &ticker
                ),
                amount
            );

            // Alice transfers custody of her portfolios but it won't affect any already approved instruction
            let auth_id2 = Identity::add_auth(
                alice_did,
                Signatory::from(bob_did),
                AuthorizationData::PortfolioCustody(PortfolioId::user_portfolio(
                    alice_did, alice_num,
                )),
                None,
            );
            assert_ok!(Identity::accept_authorization(bob_signed.clone(), auth_id2));

            // Bob fails to approve the instruction with both of his portfolios since he doesn't have custody for the second one
            let portfolios_bob = vec![
                PortfolioId::default_portfolio(bob_did),
                PortfolioId::user_portfolio(bob_did, bob_num),
            ];
            assert_noop!(
                Settlement::authorize_instruction(
                    bob_signed.clone(),
                    instruction_counter,
                    portfolios_bob
                ),
                PortfolioError::UnauthorizedCustodian
            );

            // Bob can approve instruction from the portfolio he has custody of
            assert_ok!(Settlement::authorize_instruction(
                bob_signed.clone(),
                instruction_counter,
                default_portfolio_vec(bob_did)
            ));

            // Alice fails to unauthorize the instruction from both her portfolios since she doesn't have the custody
            assert_noop!(
                Settlement::unauthorize_instruction(
                    alice_signed.clone(),
                    instruction_counter,
                    portfolios_vec
                ),
                PortfolioError::UnauthorizedCustodian
            );

            // Alice can unauthorize instruction from the portfolio she has custody of
            assert_ok!(Settlement::unauthorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                default_portfolio_vec(alice_did)
            ));
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );

            // Alice can authorize instruction from remaining portfolios since she has the custody
            let portfolios_final = vec![
                PortfolioId::default_portfolio(alice_did),
                PortfolioId::user_portfolio(bob_did, bob_num),
            ];
            assert_ok!(Settlement::authorize_instruction(
                alice_signed.clone(),
                instruction_counter,
                portfolios_final
            ));

            // Instruction should've settled
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                alice_init_balance - amount * 2
            );
            assert_eq!(
                Asset::balance_of(&ticker, bob_did),
                bob_init_balance + amount * 2
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(alice_did, &ticker),
                alice_init_balance - amount * 2,
            );
            assert_eq!(
                Portfolio::default_portfolio_balance(bob_did, &ticker),
                bob_init_balance + amount,
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(bob_did, bob_num, &ticker),
                amount,
            );
            assert_eq!(
                Portfolio::locked_assets(PortfolioId::default_portfolio(alice_did), &ticker),
                0
            );
        });
}

// ----------------------------------------- Confidential transfer tests -----------------------------------

/// Creates a mercat account and returns its secret part (to be stored in the wallet) and
/// the account creation proofs (to be submitted to the chain).
pub fn gen_account(
    tx_id: u32,
    mut rng: &mut StdRng,
    token_name: &[u8],
    valid_asset_ids: Vec<AssetId>,
) -> (SecAccount, PubAccountTx) {
    // These are the encryptions keys used by MERCAT and are different from the signing keys
    // that Polymesh uses.
    let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let elg_pub = elg_secret.get_public_key();
    let enc_keys = EncryptionKeys {
        pblc: elg_pub.into(),
        scrt: elg_secret.into(),
    };

    let asset_id = AssetId {
        id: *Ticker::try_from(token_name).unwrap().as_bytes(),
    };

    let mut seed = [0u8; 32];
    rng.fill(&mut seed);
    let mut new_rng = StdRng::from_seed(seed);
    let asset_id_witness = CommitmentWitness::from((asset_id.clone().into(), &mut new_rng));
    let secret_account = SecAccount {
        enc_keys,
        asset_id_witness,
    };
    let valid_asset_ids = convert_asset_ids(valid_asset_ids);
    let mercat_account_tx = AccountCreator {}
        .create(tx_id, &secret_account, &valid_asset_ids, &mut rng)
        .unwrap();

    (secret_account, mercat_account_tx)
}

/// Creates a mercat account for the `owner` and submits the proofs to the chain and validates them.
/// It then return the secret part of the account, the account id, the public portion of the account and the initial
/// encrypted balance of zero.
pub fn init_account(
    tx_id: u32,
    mut rng: &mut StdRng,
    token_name: &[u8],
    owner: Public,
    did: IdentityId,
) -> (SecAccount, MercatAccountId, PubAccount, EncryptedAmount) {
    let valid_asset_ids = ConfidentialAsset::confidential_tickers();
    let (secret_account, mercat_account_tx) =
        gen_account(tx_id, &mut rng, token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        Origin::signed(owner),
        PubAccountTxWrapper::from(mercat_account_tx.clone())
    ));

    let account_id = MercatAccountId(
        EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id)
            .0
            .clone(),
    );
    (
        secret_account,
        account_id.clone(),
        ConfidentialAsset::mercat_accounts(did, account_id.clone())
            .to_mercat::<TestStorage>()
            .unwrap(),
        ConfidentialAsset::mercat_account_balance(did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
    )
}

/// Performs mercat account creation, validation, and minting of the account with `total_supply` tokens.
/// It returns the next transaction id, the secret portion of the account, the account id, the public portion of the account,
/// and the encrypted balance of `total_supply`.
pub fn create_account_and_mint_token(
    owner: Public,
    owner_did: IdentityId,
    total_supply: u128,
    token_name: Vec<u8>,
    tx_id: u32,
    mut rng: &mut StdRng,
) -> (
    u32,
    SecAccount,
    MercatAccountId,
    PubAccount,
    EncryptedAmount,
) {
    let funding_round_name: FundingRoundName = b"round1".into();

    let token = SecurityToken {
        name: token_name.clone().into(),
        owner_did,
        total_supply,
        divisible: false,
        asset_type: AssetType::default(),
        primary_issuance_agent: Some(owner_did),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

    assert_ok!(ConfidentialAsset::create_confidential_asset(
        Origin::signed(owner),
        token.name.clone(),
        ticker,
        true,
        token.asset_type.clone(),
        vec![],
        Some(funding_round_name.clone()),
    ));

    // In the initial call, the total_supply must be zero.
    assert_eq!(Asset::token_details(ticker).total_supply, Zero::zero());

    // ---------------- prepare for minting the asset

    let valid_asset_ids = ConfidentialAsset::confidential_tickers();

    let (secret_account, mercat_account_tx) =
        gen_account(tx_id, &mut rng, &token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        Origin::signed(owner),
        PubAccountTxWrapper::from(mercat_account_tx.clone())
    ));

    // ------------- Computations that will happen in owner's Wallet ----------
    let amount: u32 = token.total_supply.try_into().unwrap(); // mercat amounts are 32 bit integers.
    let issuer_account = Account {
        scrt: secret_account.clone(),
        pblc: mercat_account_tx.pub_account.clone(),
    };

    let tx_id = tx_id + 1;
    let initialized_asset_tx = AssetIssuer {}
        .initialize_asset_transaction(tx_id, &issuer_account, &[], amount, &mut rng)
        .unwrap();

    // Wallet submits the transaction to the chain for verification.
    assert_ok!(ConfidentialAsset::mint_confidential_asset(
        Origin::signed(owner),
        ticker,
        amount.into(), // convert to u128
        InitializedAssetTxWrapper::from(initialized_asset_tx),
    ));

    // ------------------------- Ensuring that the asset details are set correctly

    // A correct entry is added.
    assert_eq!(
        Asset::asset_ownership_relation(token.owner_did, ticker),
        AssetOwnershipRelation::AssetOwned
    );
    assert_eq!(Asset::funding_round(ticker), funding_round_name.clone());

    // Ticker is added to the list of confidential tokens.
    assert_eq!(
        ConfidentialAsset::confidential_tickers().last(),
        Some(&AssetId {
            id: *ticker.as_bytes()
        })
    );

    // -------------------------- Ensure the encrypted balance matches the minted amount.
    let account_id = MercatAccountId(
        EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id)
            .0
            .clone(),
    );
    let stored_balance = ConfidentialAsset::mercat_account_balance(owner_did, account_id.clone())
        .to_mercat::<TestStorage>()
        .unwrap();
    let stored_balance = secret_account
        .enc_keys
        .scrt
        .decrypt(&stored_balance)
        .unwrap();

    assert_eq!(stored_balance, amount);

    (
        tx_id,
        secret_account,
        account_id.clone(),
        ConfidentialAsset::mercat_accounts(owner_did, account_id.clone())
            .to_mercat::<TestStorage>()
            .unwrap(),
        ConfidentialAsset::mercat_account_balance(owner_did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
    )
}

#[test]
fn basic_confidential_settlement() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .set_max_legs_allowed(500)
        .build()
        .execute_with(|| {
            // The rest of rngs are built from it. Its initial value can be set using proptest.
            let mut rng = StdRng::from_seed([10u8; 32]);

            // Setting:
            //   - Alice is the token issuer.
            //   - Alice is also the sender of the token.
            //   - Bob is the receiver of the token.
            //   - Charlie is the mediator.
            //   - Eve is the CDD provider.
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();

            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

            let charlie = AccountKeyring::Charlie.public();
            let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

            // ------------ Setup mercat
            let token_name = b"ACME";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();

            // Create an account for Alice and mint 10,000,000 tokens to ACME.
            // let total_supply = 1_1000_000;
            let total_supply = 5;
            let (
                tx_id,
                alice_secret_account,
                alice_account_id,
                alice_public_account,
                alice_encrypted_init_balance,
            ) = create_account_and_mint_token(
                AccountKeyring::Alice.public(), // owner of ACME.
                alice_did,
                total_supply,
                token_name.to_vec(),
                0, // transaction id: not important in this test.
                &mut rng,
            );

            // Create accounts for Bob, and Charlie.
            let tx_id = tx_id + 1;
            let (
                bob_secret_account,
                bob_account_id,
                bob_public_account,
                bob_encrypted_init_balance,
            ) = init_account(
                tx_id,
                &mut rng,
                token_name,
                AccountKeyring::Bob.public(),
                bob_did,
            );

            let tx_id = tx_id + 1;
            let (charlie_secret_account, _, charlie_public_account, _) =
                init_account(tx_id, &mut rng, token_name, charlie, charlie_did);

            // Mediator creates a venue
            let venue_counter = Settlement::venue_counter();
            assert_ok!(Settlement::create_venue(
                Origin::signed(charlie),
                VenueDetails::default(),
                vec![charlie],
                VenueType::Other
            ));

            // Mediator creates an instruction
            let instruction_counter = Settlement::instruction_counter();

            //// Provide scope claim to sender and receiver of the transaction.
            //provide_scope_claim_to_multiple_parties(&[alice_did, bob_did], ticker, alice); // TODO: CRYP-172 I think we decided not to do this as it would leak the ticker name

            assert_ok!(Settlement::add_instruction(
                Origin::signed(charlie),
                venue_counter,
                SettlementType::SettleOnAuthorization,
                None,
                vec![Leg::ConfidentialLeg(ConfidentialLeg {
                    from: PortfolioId::default_portfolio(alice_did),
                    to: PortfolioId::default_portfolio(bob_did),
                    mediator: PortfolioId::default_portfolio(charlie_did),
                    from_account_id: alice_account_id.clone(),
                    to_account_id: bob_account_id.clone(),
                })]
            ));

            // -------------------------- Perform the transfer
            let amount = 100u32; // This plain format is only used on functions that emulate the work of the wallet.

            println!("-------------> Checking if alice has enough funds.");
            // Ensure that Alice has minted enough tokens.
            assert!(
                alice_secret_account
                    .enc_keys
                    .scrt
                    .decrypt(&alice_encrypted_init_balance)
                    .unwrap()
                    > amount
            );

            // ----- Sender authorizes.
            // Sender computes the proofs in the wallet.
            println!("-------------> Alice is going to authorize.");
            let tx_id = tx_id + 1;
            let sender_data = CtxSender {}
                .create_transaction(
                    tx_id,
                    &Account {
                        pblc: alice_public_account.clone(),
                        scrt: alice_secret_account.clone(),
                    },
                    &alice_encrypted_init_balance,
                    &bob_public_account,
                    &charlie_public_account.owner_enc_pub_key,
                    &[],
                    amount,
                    &mut rng,
                )
                .unwrap();
            let alice_encrypted_transfer_amount = sender_data.memo.enc_amount_using_sndr;
            let bob_encrypted_transfer_amount = sender_data.memo.enc_amount_using_rcvr;
            let initialized_tx = MercatTxData::InitializedTransfer(
                Base64Vec::new(sender_data.encode()).into_bytes(),
            );
            // Sender authorizes the instruction and passes in the proofs.
            assert_ok!(Settlement::authorize_confidential_instruction(
                Origin::signed(AccountKeyring::Alice.public()),
                instruction_counter,
                initialized_tx,
                default_portfolio_vec(alice_did),
            ));

            // ------ Receiver authorizes.
            // Receiver reads the sender's proof from the chain.
            println!("-------------> Bob is going to authorize.");
            let mut tx_data = Settlement::mercat_tx_data(instruction_counter);
            assert_eq!(tx_data.len(), 1);

            let tx_data = tx_data.remove(0);

            let decoded_initialized_tx = match tx_data {
                MercatTxData::InitializedTransfer(init) => {
                    let mut data: &[u8] = &init.decode().unwrap();
                    InitializedTransferTx::decode(&mut data).unwrap()
                }
                _ => {
                    println!("{:?}", tx_data);
                    panic!("Unexpected data type");
                }
            };

            // Receiver computes the proofs in the wallet.
            let finalized_tx = MercatTxData::FinalizedTransfer(Base64Vec::new(
                CtxReceiver {}
                    .finalize_transaction(
                        tx_id,
                        decoded_initialized_tx,
                        Account {
                            pblc: bob_public_account.clone(),
                            scrt: bob_secret_account.clone(),
                        },
                        amount,
                        &mut rng,
                    )
                    .unwrap()
                    .encode(),
            ));

            // Receiver submits the proof to the chain.
            assert_ok!(Settlement::authorize_confidential_instruction(
                Origin::signed(AccountKeyring::Bob.public()),
                instruction_counter,
                finalized_tx,
                default_portfolio_vec(bob_did),
            ));

            // ------ Mediator authorizes.
            // Mediator reads the receiver's proofs from the chain (it contains the sender's proofs as well).
            println!("-------------> Charlie is going to authorize.");
            let mut tx_data = Settlement::mercat_tx_data(instruction_counter);
            assert_eq!(tx_data.len(), 2);

            let tx_data = tx_data.remove(1);
            let decoded_finalized_tx = match tx_data {
                MercatTxData::FinalizedTransfer(finalized) => {
                    let mut data: &[u8] = &finalized.decode().unwrap();
                    FinalizedTransferTx::decode(&mut data).unwrap()
                }
                _ => {
                    panic!("Unexpected data type");
                }
            };

            // Mediator verifies the proofs in the wallet.
            let justified_tx = MercatTxData::JustifiedTransfer(Base64Vec::new(
                CtxMediator {}
                    .justify_transaction(
                        decoded_finalized_tx,
                        &charlie_secret_account.enc_keys,
                        &alice_public_account,
                        &alice_encrypted_init_balance,
                        &bob_public_account,
                        &[],
                        AssetId {
                            id: *ticker.as_bytes(),
                        },
                        &mut rng,
                    )
                    .unwrap()
                    .encode(),
            ));

            println!("-------------> This should trigger the execution");
            assert_ok!(Settlement::authorize_confidential_instruction(
                Origin::signed(charlie),
                instruction_counter,
                justified_tx,
                default_portfolio_vec(charlie_did),
            ));

            // Instruction should've settled.
            // Verify by decrypting the new balance of both Alice and Bob.
            let new_alice_balance =
                ConfidentialAsset::mercat_account_balance(alice_did, alice_account_id)
                    .to_mercat::<TestStorage>()
                    .unwrap();
            let expected_alice_balance =
                alice_encrypted_init_balance - alice_encrypted_transfer_amount;
            assert_eq!(new_alice_balance, expected_alice_balance);

            // let new_alice_balance = alice_secret_account
            //     .enc_keys
            //     .scrt
            //     .decrypt(&new_alice_balance)
            //     .unwrap();
            // assert_eq!(new_alice_balance as u128, total_supply - amount as u128);

            let new_bob_balance =
                ConfidentialAsset::mercat_account_balance(bob_did, bob_account_id)
                    .to_mercat::<TestStorage>()
                    .unwrap();

            let expected_bob_balance = bob_encrypted_init_balance + bob_encrypted_transfer_amount;
            assert_eq!(new_bob_balance, expected_bob_balance);
            // let new_bob_balance = bob_secret_account
            //     .enc_keys
            //     .scrt
            //     .decrypt(&new_bob_balance)
            //     .unwrap();
            // assert_eq!(new_bob_balance, amount);
        });
}
