use super::{
    storage::{default_portfolio_vec, next_block, register_keyring_account, TestStorage},
    ExtBuilder,
};
use codec::{Decode, Encode};
use confidential_asset::{
    EncryptedAssetIdWrapper, InitializedAssetTxWrapper, MercatAccountId, PubAccountTxWrapper,
};
use core::convert::{TryFrom, TryInto};
use frame_support::assert_ok;
use mercat::{
    account::{convert_asset_ids, AccountCreator},
    asset::AssetIssuer,
    cryptography_core::{
        asset_proofs::{CommitmentWitness, ElgamalSecretKey},
        curve25519_dalek::scalar::Scalar,
        AssetId,
    },
    transaction::{CtxMediator, CtxReceiver, CtxSender},
    Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptedAmount, EncryptionKeys,
    FinalizedTransferTx, InitializedTransferTx, PubAccount, PubAccountTx, SecAccount,
    TransferTransactionMediator, TransferTransactionReceiver, TransferTransactionSender,
};
use pallet_asset as asset;
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_confidential_asset as confidential_asset;
use pallet_identity as identity;
use pallet_settlement::{
    self as settlement, ConfidentialLeg, Leg, LegKind, MercatTxData, SettlementType, VenueDetails,
    VenueType,
};
use polymesh_primitives::{
    asset::{AssetOwnershipRelation, AssetType, Base64Vec, FundingRoundName, SecurityToken},
    IdentityId, PortfolioId, Ticker,
};
use rand::prelude::*;
use sp_core::sr25519::Public;
use sp_runtime::traits::Zero;
use sp_runtime::AnySignature;
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

#[derive(Clone, Debug)]
struct AccountCredentials {
    key: AccountKeyring,
    did: IdentityId,
    account_id: MercatAccountId,
    public_account: PubAccount,
}

#[derive(Clone, Debug)]
struct MediatorCredentials {
    mediator_key: AccountKeyring,
    mediator_did: IdentityId,
    mediator_public_account: PubAccount,
    ticker: Ticker,
}

/// Creates a mercat account and returns its secret part (to be stored in the wallet) and
/// the account creation proofs (to be submitted to the chain).
pub fn gen_account(
    mut rng: &mut StdRng,
    token_name: &[u8],
    valid_asset_ids: Vec<AssetId>,
) -> (SecAccount, PubAccountTx) {
    // These are the encryptions keys used by MERCAT and are different from the signing keys
    // that Polymesh uses.
    let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let elg_pub = elg_secret.get_public_key();
    let enc_keys = EncryptionKeys {
        public: elg_pub.into(),
        secret: elg_secret.into(),
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
    let mercat_account_tx = AccountCreator
        .create(&secret_account, &valid_asset_ids, &mut rng)
        .unwrap();

    (secret_account, mercat_account_tx)
}

/// Creates a mercat account for the `owner` and submits the proofs to the chain and validates them.
/// It then return the secret part of the account, the account id, the public portion of the account and the initial
/// encrypted balance of zero.
pub fn init_account(
    mut rng: &mut StdRng,
    token_name: &[u8],
    owner: Public,
    did: IdentityId,
) -> (SecAccount, MercatAccountId, PubAccount, EncryptedAmount) {
    let valid_asset_ids = ConfidentialAsset::confidential_tickers();
    let (secret_account, mercat_account_tx) = gen_account(&mut rng, token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        Origin::signed(owner),
        PubAccountTxWrapper::from(mercat_account_tx.clone()),
    ));

    let account_id = MercatAccountId(
        EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id)
            .0
            .clone(),
    );
    (
        secret_account,
        account_id.clone(),
        ConfidentialAsset::mercat_accounts(did, &account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
        ConfidentialAsset::mercat_account_balance(did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
    )
}

/// Performs mercat account creation, validation, and minting of the account with `total_supply` tokens.
/// It returns the the secret portion of the account, the account id, the public portion of the account,
/// and the encrypted balance of `total_supply`.
pub fn create_account_and_mint_token(
    owner: Public,
    owner_did: IdentityId,
    total_supply: u128,
    token_name: Vec<u8>,
    mut rng: &mut StdRng,
) -> (SecAccount, MercatAccountId, PubAccount, EncryptedAmount) {
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

    // Prepare for minting the asset.

    let valid_asset_ids = ConfidentialAsset::confidential_tickers();

    let (secret_account, mercat_account_tx) = gen_account(&mut rng, &token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        Origin::signed(owner),
        PubAccountTxWrapper::from(mercat_account_tx.clone()),
    ));

    // Computations that will happen in owner's Wallet.
    let amount: u32 = token.total_supply.try_into().unwrap();
    let issuer_account = Account {
        secret: secret_account.clone(),
        public: mercat_account_tx.pub_account.clone(),
    };

    let initialized_asset_tx = AssetIssuer
        .initialize_asset_transaction(&issuer_account, &[], amount, &mut rng)
        .unwrap();

    // Wallet submits the transaction to the chain for verification.
    assert_ok!(ConfidentialAsset::mint_confidential_asset(
        Origin::signed(owner),
        ticker,
        amount.into(), // convert to u128.
        InitializedAssetTxWrapper::from(initialized_asset_tx),
    ));

    // Ensuring that the asset details are set correctly.

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

    let account_id = MercatAccountId(
        EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id)
            .0
            .clone(),
    );

    (
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

fn initialize_transaction(
    sender_secret_account: SecAccount,
    sender_creds: AccountCredentials,
    sender_pending_balance: EncryptedAmount,
    receiver_secret_account: SecAccount,
    receiver_creds: AccountCredentials,
    mediator_creds: MediatorCredentials,
    amount: u32,
) -> (u64, EncryptedAmount, EncryptedAmount) {
    // The rest of rngs are built from it.
    let mut rng = StdRng::from_seed([10u8; 32]);

    // Mediator creates a venue.
    let venue_counter = Settlement::venue_counter();
    assert_ok!(Settlement::create_venue(
        Origin::signed(mediator_creds.mediator_key.public()),
        VenueDetails::default(),
        vec![mediator_creds.mediator_key.public()],
        VenueType::Other
    ));

    // Mediator creates an instruction.
    let instruction_counter = Settlement::instruction_counter();

    assert_ok!(Settlement::add_instruction(
        Origin::signed(mediator_creds.mediator_key.public()),
        venue_counter,
        SettlementType::SettleOnAffirmation,
        None,
        None,
        vec![Leg {
            from: PortfolioId::default_portfolio(sender_creds.did),
            to: PortfolioId::default_portfolio(receiver_creds.did),
            kind: LegKind::Confidential(ConfidentialLeg {
                mediator: PortfolioId::default_portfolio(mediator_creds.mediator_did),
                from_account_id: sender_creds.account_id.clone(),
                to_account_id: receiver_creds.account_id.clone(),
            }),
        }]
    ));

    // Sender authorizes.
    // Sender computes the proofs in the wallet.
    let sender_data = CtxSender
        .create_transaction(
            &Account {
                public: sender_creds.public_account.clone(),
                secret: sender_secret_account.clone(),
            },
            &sender_pending_balance,
            &receiver_creds.public_account,
            &mediator_creds.mediator_public_account.owner_enc_pub_key,
            &[],
            amount,
            &mut rng,
        )
        .unwrap();
    let initialized_tx = MercatTxData::InitializedTransfer(Base64Vec::new(sender_data.encode()));
    // Sender authorizes the instruction and passes in the proofs.
    assert_ok!(Settlement::affirm_confidential_instruction(
        Origin::signed(sender_creds.key.public()),
        instruction_counter,
        initialized_tx,
        default_portfolio_vec(sender_creds.did),
        1
    ));

    // Receiver authorizes.
    // Receiver reads the sender's proof from the chain.
    let mut tx_data = Settlement::mercat_tx_data(instruction_counter);
    assert_eq!(tx_data.len(), 1);

    let tx_data = tx_data.remove(0);

    let decoded_initialized_tx = match tx_data {
        MercatTxData::InitializedTransfer(init) => {
            let mut data: &[u8] = &init.decode().unwrap();
            InitializedTransferTx::decode(&mut data).unwrap()
        }
        _ => {
            panic!("Unexpected data type");
        }
    };
    let sender_encrypted_transfer_amount = decoded_initialized_tx.memo.enc_amount_using_sender;
    let receiver_encrypted_transfer_amount = decoded_initialized_tx.memo.enc_amount_using_receiver;

    // Receiver computes the proofs in the wallet.
    let finalized_tx = MercatTxData::FinalizedTransfer(Base64Vec::new(
        CtxReceiver
            .finalize_transaction(
                decoded_initialized_tx,
                Account {
                    public: receiver_creds.public_account.clone(),
                    secret: receiver_secret_account.clone(),
                },
                amount,
                &mut rng,
            )
            .unwrap()
            .encode(),
    ));

    // Receiver submits the proof to the chain.
    assert_ok!(Settlement::affirm_confidential_instruction(
        Origin::signed(receiver_creds.key.public()),
        instruction_counter,
        finalized_tx,
        default_portfolio_vec(receiver_creds.did),
        1
    ));

    (
        instruction_counter,
        sender_encrypted_transfer_amount,
        receiver_encrypted_transfer_amount,
    )
}

fn finalize_transaction(
    instruction_counter: u64,
    sender_creds: AccountCredentials,
    receiver_creds: AccountCredentials,
    mediator_creds: MediatorCredentials,
    mediator_secret_account: SecAccount,
    expected_sender_balance: EncryptedAmount,
    expected_receiver_balance: EncryptedAmount,
    validation_failure_expected: bool,
) {
    // The rest of rngs are built from it.
    let mut rng = StdRng::from_seed([10u8; 32]);

    // Mediator authorizes.
    // Mediator reads the receiver's proofs from the chain (it contains the sender's proofs as well).
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
    // Mediator has access to the ticker name in plaintext.
    // Mediator gets the pending state for this instruction from chain.
    let sender_pending_balance = ConfidentialAsset::mercat_tx_pending_state((
        sender_creds.did,
        sender_creds.account_id.clone(),
        instruction_counter,
    ))
    .to_mercat::<TestStorage>()
    .unwrap();

    let result = CtxMediator.justify_transaction(
        decoded_finalized_tx.clone(),
        &mediator_secret_account.enc_keys,
        &sender_creds.public_account,
        &sender_pending_balance,
        &receiver_creds.public_account,
        &[],
        AssetId {
            id: *mediator_creds.ticker.as_bytes(),
        },
        &mut rng,
    );

    if validation_failure_expected {
        assert!(result.is_err());
        return;
    }

    let justified_tx = MercatTxData::JustifiedTransfer(Base64Vec::new(result.unwrap().encode()));

    // Affirms and process the transaction.
    assert_ok!(Settlement::affirm_confidential_instruction(
        Origin::signed(mediator_creds.mediator_key.public()),
        instruction_counter,
        justified_tx,
        default_portfolio_vec(mediator_creds.mediator_did),
        1
    ));

    // Execute affirmed and scheduled instructions.
    next_block();

    // Instruction should've settled.
    // Verify by decrypting the new balance of both Sender and Receiver.
    let new_sender_balance =
        ConfidentialAsset::mercat_account_balance(sender_creds.did, sender_creds.account_id)
            .to_mercat::<TestStorage>()
            .unwrap();

    assert_eq!(new_sender_balance, expected_sender_balance);

    let new_receiver_balance =
        ConfidentialAsset::mercat_account_balance(receiver_creds.did, receiver_creds.account_id)
            .to_mercat::<TestStorage>()
            .unwrap();

    assert_eq!(new_receiver_balance, expected_receiver_balance);
}

fn chain_set_up(
    total_supply: u128,
) -> (
    AccountCredentials,
    SecAccount,
    EncryptedAmount,
    MediatorCredentials,
    SecAccount,
) {
    // The rest of rngs are built from it.
    let mut rng = StdRng::from_seed([10u8; 32]);

    // Setting:
    //   - Alice is the token issuer.
    //   - Alice is also the sender of the token.
    //   - Bob is the receiver of the token.
    //   - Charlie is the mediator.
    let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();

    let charlie = AccountKeyring::Charlie.public();
    let charlie_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

    // Setup a mercat asset.
    let token_name = b"ACME";
    let ticker = Ticker::try_from(&token_name[..]).unwrap();

    // Create an account for Alice and mint `total_supply` tokens to ACME.
    let (
        alice_secret_account,
        alice_account_id,
        alice_public_account,
        alice_encrypted_init_balance,
    ) = create_account_and_mint_token(
        AccountKeyring::Alice.public(), // owner of ACME.
        alice_did,
        total_supply,
        token_name.to_vec(),
        &mut rng,
    );

    let alice_creds = AccountCredentials {
        key: AccountKeyring::Alice,
        did: alice_did,
        account_id: alice_account_id,
        public_account: alice_public_account,
    };

    // Create an account for Charlie.
    let (charlie_secret_account, _, charlie_public_account, _) =
        init_account(&mut rng, token_name, charlie, charlie_did);
    let charlie_creds = MediatorCredentials {
        mediator_key: AccountKeyring::Charlie,
        mediator_did: charlie_did,
        mediator_public_account: charlie_public_account,
        ticker,
    };

    (
        alice_creds,
        alice_secret_account,
        alice_encrypted_init_balance,
        charlie_creds,
        charlie_secret_account,
    )
}

fn create_investor_account(
    key: AccountKeyring,
) -> (SecAccount, AccountCredentials, EncryptedAmount) {
    let mut rng = StdRng::from_seed([10u8; 32]);
    let token_name = b"ACME";
    // Create accounts for the key holder.
    let did = register_keyring_account(key).unwrap();

    let (secret_account, account_id, public_account, init_balance) =
        init_account(&mut rng, token_name, key.public(), did);

    let creds = AccountCredentials {
        key,
        did,
        account_id,
        public_account,
    };

    (secret_account, creds, init_balance)
}

#[test]
fn settle_out_of_order() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| {
            // Setting:
            //   - Alice is the token issuer, and has 10 assets in her supply.
            //   - Bob has a normal account.
            //   - Charlie is the mediator.
            //   - Eve is the CDD provider.
            let (
                alice_creds,
                alice_secret_account,
                alice_init_balance,
                charlie_creds,
                charlie_secret_account,
            ) = chain_set_up(10u128);

            let (bob_secret_account, bob_creds, bob_init_balance) =
                create_investor_account(AccountKeyring::Bob);

            // tx_id:1000 => Alice sends 5 assets to Bob.
            // tx_id:1001 => Alice sends 3 assets to Bob.
            //            => Charlie (the mediator) approves tx_id:1001 first.
            //            => Charlie (the mediator) approves tx_id:1000 second.
            let (instruction_counter1000, alice_sent_amount_1000, bob_received_amount_1000) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    5,
                );

            let alice_init_balance2 = alice_init_balance - alice_sent_amount_1000;
            let (instruction_counter1001, alice_sent_amount_1001, bob_received_amount_1001) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance2.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    3,
                );

            // Approve and process tx:1001.
            finalize_transaction(
                instruction_counter1001,
                alice_creds.clone(),
                bob_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1001,
                bob_init_balance + bob_received_amount_1001,
                false,
            );

            // Approve and process tx:1000.
            finalize_transaction(
                instruction_counter1000,
                alice_creds.clone(),
                bob_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1001 - alice_sent_amount_1000,
                bob_init_balance + bob_received_amount_1001 + bob_received_amount_1000,
                false,
            );
        });
}

#[test]
fn double_spending_fails() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| {
            // Setting:
            //   - Alice is the token issuer, and has 10 assets in her supply.
            //   - Bob has a normal account.
            //   - Charlie is the mediator.
            //   - Eve is the CDD provider.
            let (
                alice_creds,
                alice_secret_account,
                alice_init_balance,
                charlie_creds,
                charlie_secret_account,
            ) = chain_set_up(10u128);

            let (bob_secret_account, bob_creds, bob_init_balance) =
                create_investor_account(AccountKeyring::Bob);

            let (dave_secret_account, dave_creds, dave_init_balance) =
                create_investor_account(AccountKeyring::Dave);

            // Alice has 10 assets.
            // tx_id:1000 => Alice sends 5 assets to Bob.
            // tx_id:1001 => Alice sends 10 assets to Dave.
            //            => Charlie (the mediator) catches tx_id:1001's double spend.
            //            => Charlie (the mediator) approves tx_id:1000.
            let (instruction_counter1000, alice_sent_amount_1000, bob_received_amount_1000) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    5,
                );

            let (instruction_counter1001, alice_sent_amount_1001, dave_received_amount_1001) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    // Alice is reusing her initial balance as the pending balance.
                    // This is an attempt to double spend.
                    // She should have used `alice_init_balance - alice_sent_amount_1000`.
                    alice_init_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    10,
                );

            // Mediator fails the tx:1001.
            finalize_transaction(
                instruction_counter1001,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1001,
                dave_init_balance + dave_received_amount_1001,
                true, // Validation failure expected.
            );

            // Approve and process tx:1000.
            finalize_transaction(
                instruction_counter1000,
                alice_creds.clone(),
                bob_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1000,
                bob_init_balance + bob_received_amount_1000,
                false,
            );
        });
}

#[test]
fn mercat_whitepaper_scenario1() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| {
            // Setting:
            //   - Alice is the token issuer, and has 10 assets in her supply.
            //   - Bob has a normal account.
            //   - Dave has a normal account.
            //   - Charlie is the mediator.
            //   - Eve is the CDD provider.
            let (
                alice_creds,
                alice_secret_account,
                alice_init_balance,
                charlie_creds,
                charlie_secret_account,
            ) = chain_set_up(90u128);

            let (bob_secret_account, bob_creds, _) = create_investor_account(AccountKeyring::Bob);

            let (dave_secret_account, dave_creds, dave_init_balance) =
                create_investor_account(AccountKeyring::Dave);

            // Alice, the token issuer, sends 10 tokens to Dave so he has something in his account.
            let (instruction_counter999, alice_sent_amount_999, dave_received_amount_999) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    10,
                );
            finalize_transaction(
                instruction_counter999,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_999,
                dave_init_balance + dave_received_amount_999,
                false,
            );
            // Reset Dave's pending state.
            assert_ok!(ConfidentialAsset::reset_ordering_state(
                Origin::signed(dave_creds.key.public()),
                dave_creds.account_id.clone()
            ));
            let dave_init_balance = dave_init_balance + dave_received_amount_999;
            let alice_init_balance = alice_init_balance - alice_sent_amount_999;

            // tx_id:1000 => Alice sends 10 assets to Bob.
            // tx_id:1001 => Alice receives 8 tokens from Dave.
            // tx_id:1002 => Alice sends 14 tokens to Dave.
            //            => Charlie (the mediator) fails tx_id:1000.
            //            => Charlie (the mediator) approves tx_id:1001.
            //            => Charlie (the mediator) approves tx_id:1002.
            let (instruction_counter1000, alice_sent_amount_1000, _bob_received_amount_1000) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    10,
                );
            let alice_pending_balance = alice_init_balance - alice_sent_amount_1000;

            let (instruction_counter1001, dave_sent_amount_1001, alice_received_amount_1001) =
                initialize_transaction(
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    dave_init_balance.clone(),
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    charlie_creds.clone(),
                    8,
                );

            let (instruction_counter1002, alice_sent_amount_1002, dave_received_amount_1002) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_pending_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    14,
                );

            // Approve and process tx:1001.
            finalize_transaction(
                instruction_counter1001,
                dave_creds.clone(),
                alice_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                dave_init_balance - dave_sent_amount_1001,
                alice_init_balance + alice_received_amount_1001,
                false,
            );

            // Alice has a change of heart and rejects the transaction to Bob!
            assert_ok!(Settlement::reject_instruction(
                Origin::signed(alice_creds.key.public()),
                instruction_counter1000,
                default_portfolio_vec(alice_creds.did),
                1
            ));

            // Approve and process tx:1002.
            finalize_transaction(
                instruction_counter1002,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance + alice_received_amount_1001 - alice_sent_amount_1002,
                dave_init_balance - dave_sent_amount_1001 + dave_received_amount_1002,
                false,
            );
        });
}

#[test]
fn mercat_whitepaper_scenario2() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .build()
        .execute_with(|| {
            // Setting:
            //   - Alice is the token issuer, and has 10 assets in her supply.
            //   - Bob has a normal account.
            //   - Dave has a normal account.
            //   - Charlie is the mediator.
            //   - Eve is the CDD provider.
            let (
                alice_creds,
                alice_secret_account,
                alice_init_balance,
                charlie_creds,
                charlie_secret_account,
            ) = chain_set_up(90u128);

            let (bob_secret_account, bob_creds, bob_init_balance) =
                create_investor_account(AccountKeyring::Bob);

            let (dave_secret_account, dave_creds, dave_init_balance) =
                create_investor_account(AccountKeyring::Dave);

            // Alice, the token issuer, sends 10 tokens to Dave so he has something in his account.
            let (instruction_counter999, alice_sent_amount_999, dave_received_amount_999) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    10,
                );
            finalize_transaction(
                instruction_counter999,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_999,
                dave_init_balance + dave_received_amount_999,
                false,
            );
            // Reset Dave's pending state.
            assert_ok!(ConfidentialAsset::reset_ordering_state(
                Origin::signed(dave_creds.key.public()),
                dave_creds.account_id.clone()
            ));
            let dave_init_balance = dave_init_balance + dave_received_amount_999;
            let alice_init_balance = alice_init_balance - alice_sent_amount_999;

            // tx_id:1000 => Alice sends 10 assets to Bob.
            // tx_id:1001 => Alice receives 8 tokens from Dave.
            // tx_id:1002 => Alice sends 14 tokens to Dave.
            //            => Charlie (the mediator) fails tx_id:1000.
            //            => Charlie (the mediator) approves tx_id:1001.
            //            => Charlie (the mediator) approves tx_id:1002.
            // tx_id:1003 => Alice sends 19 assets to Bob.
            // Alice resets her pending state.
            // tx_id:1004 => Alice sends 55 assets to Dave.
            let (instruction_counter1000, alice_sent_amount_1000, _bob_received_amount_1000) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_init_balance.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    10,
                );
            let alice_pending_balance = alice_init_balance - alice_sent_amount_1000;

            let (instruction_counter1001, dave_sent_amount_1001, alice_received_amount_1001) =
                initialize_transaction(
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    dave_init_balance.clone(),
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    charlie_creds.clone(),
                    8,
                );

            let (instruction_counter1002, alice_sent_amount_1002, dave_received_amount_1002) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_pending_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    14,
                );
            let alice_pending_balance = alice_pending_balance - alice_sent_amount_1002;

            // Approve and process tx:1001.
            finalize_transaction(
                instruction_counter1001,
                dave_creds.clone(),
                alice_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                dave_init_balance - dave_sent_amount_1001,
                alice_init_balance + alice_received_amount_1001,
                false,
            );

            // Alice has a change of heart and rejects the transaction to Bob!
            assert_ok!(Settlement::reject_instruction(
                Origin::signed(alice_creds.key.public()),
                instruction_counter1000,
                default_portfolio_vec(alice_creds.did),
                1
            ));

            // Approve and process tx:1002.
            finalize_transaction(
                instruction_counter1002,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance + alice_received_amount_1001 - alice_sent_amount_1002,
                dave_init_balance - dave_sent_amount_1001 + dave_received_amount_1002,
                false,
            );

            // tx_id:1003 => Alice sends 19 assets to Bob.
            let (instruction_counter1003, alice_sent_amount_1003, bob_received_amount_1003) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_pending_balance.clone(),
                    bob_secret_account.clone(),
                    bob_creds.clone(),
                    charlie_creds.clone(),
                    19,
                );

            // Reset Alice's pending state.
            assert_ok!(ConfidentialAsset::reset_ordering_state(
                Origin::signed(alice_creds.key.public()),
                alice_creds.account_id.clone(),
            ));
            // On the Alice's wallet side, she also resets her pending state.
            let alice_init_balance = ConfidentialAsset::mercat_account_balance(
                alice_creds.did.clone(),
                alice_creds.account_id.clone(),
            )
            .to_mercat::<TestStorage>()
            .unwrap();
            // Since tx_1003 has not settled yet, it has to be accounted for in the pending balance.
            let alice_pending_balance = alice_init_balance - alice_sent_amount_1003;

            // tx_id:1004 => Alice sends 55 assets to Dave.
            let (instruction_counter1004, alice_sent_amount_1004, dave_received_amount_1004) =
                initialize_transaction(
                    alice_secret_account.clone(),
                    alice_creds.clone(),
                    alice_pending_balance.clone(),
                    dave_secret_account.clone(),
                    dave_creds.clone(),
                    charlie_creds.clone(),
                    55,
                );

            // Approve and process tx:1004.
            finalize_transaction(
                instruction_counter1004,
                alice_creds.clone(),
                dave_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1004,
                dave_init_balance - dave_sent_amount_1001
                    + dave_received_amount_1002
                    + dave_received_amount_1004,
                false,
            );

            // Approve and process tx:1003.
            finalize_transaction(
                instruction_counter1003,
                alice_creds.clone(),
                bob_creds.clone(),
                charlie_creds.clone(),
                charlie_secret_account.clone(),
                alice_init_balance - alice_sent_amount_1004 - alice_sent_amount_1003,
                bob_init_balance + bob_received_amount_1003,
                false,
            );
        });
}
