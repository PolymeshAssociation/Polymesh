use super::{
    asset_test::enable_investor_count,
    storage::{TestStorage, User},
    ExtBuilder,
};
use confidential_asset::{
    EncryptedAssetIdWrapper, InitializedAssetTxWrapper, MercatAccountId, PubAccountTxWrapper,
};
use core::convert::{TryFrom, TryInto};
use frame_support::{assert_err, assert_ok};
use mercat::{
    account::{convert_asset_ids, AccountCreator},
    asset::AssetIssuer,
    confidential_identity_core::{
        asset_proofs::{AssetId, CommitmentWitness, ElgamalSecretKey},
        curve25519_dalek::scalar::Scalar,
    },
    Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptionKeys, PubAccountTx,
    SecAccount,
};
use pallet_asset::{self as asset, AssetOwnershipRelation};
use pallet_confidential_asset as confidential_asset;
use pallet_identity as identity;
use pallet_statistics as statistics;
use polymesh_primitives::{
    asset::{AssetName, AssetType, FundingRoundName, SecurityToken},
    AssetIdentifier, Ticker,
};
use rand::{rngs::StdRng, SeedableRng};
use sp_runtime::traits::Zero;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Config>::Origin;
type DidRecords = identity::DidRecords<TestStorage>;
type Statistics = statistics::Module<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type ConfidentialAsset = confidential_asset::Module<TestStorage>;

macro_rules! assert_affirm_confidential_instruction {
    ($signer:expr, $instruction_id:expr, $data:expr, $did:expr, $count:expr) => {
        assert_ok!(Settlement::affirm_confidential_instruction(
            $signer,
            $instruction_id,
            $data,
            default_portfolio_vec($did),
            $count
        ));
    };
}

fn create_confidential_token(token_name: &[u8], ticker: Ticker, user: User) {
    assert_ok!(ConfidentialAsset::create_confidential_asset(
        user.origin(),
        AssetName(token_name.into()),
        ticker,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
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

    let asset_id_witness = CommitmentWitness::from((asset_id.clone().into(), &mut rng));
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
    owner: User,
) -> (SecAccount, MercatAccountId, PubAccount, EncryptedAmount) {
    let valid_asset_ids = ConfidentialAsset::confidential_tickers();
    let (secret_account, mercat_account_tx) = gen_account(&mut rng, token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        owner.origin(),
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
        ConfidentialAsset::mercat_accounts(owner.did, account_id.clone())
            .to_mercat::<TestStorage>()
            .unwrap(),
        ConfidentialAsset::mercat_account_balance(owner.did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
    )
}

/// Performs mercat account creation, validation, and minting of the account with `total_supply` tokens.
/// It returns the next the secret portion of the account, the account id, the public portion of the account,
/// and the encrypted balance of `total_supply`.
pub fn create_account_and_mint_token(
    owner: User,
    total_supply: u128,
    token_name: Vec<u8>,
    mut rng: &mut StdRng,
) -> (SecAccount, MercatAccountId, PubAccount, EncryptedAmount) {
    let funding_round_name: FundingRoundName = b"round1".into();

    let token = SecurityToken {
        total_supply,
        owner_did: owner.did,
        divisible: true,
        asset_type: AssetType::default(),
    };
    let ticker = Ticker::try_from(token_name.as_slice()).unwrap();

    assert_ok!(ConfidentialAsset::create_confidential_asset(
        owner.origin(),
        AssetName(token_name.clone()),
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

    let (secret_account, mercat_account_tx) = gen_account(&mut rng, &token_name, valid_asset_ids);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        owner.origin(),
        PubAccountTxWrapper::from(mercat_account_tx.clone())
    ));

    // ------------- Computations that will happen in owner's Wallet ----------
    let amount: u32 = token.total_supply.try_into().unwrap(); // mercat amounts are 32 bit integers.
    let issuer_account = Account {
        secret: secret_account.clone(),
        public: mercat_account_tx.pub_account.clone(),
    };

    let initialized_asset_tx = AssetIssuer
        .initialize_asset_transaction(&issuer_account, &[], amount, &mut rng)
        .unwrap();

    // Wallet submits the transaction to the chain for verification.
    assert_ok!(ConfidentialAsset::mint_confidential_asset(
        owner.origin(),
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
    let stored_balance = ConfidentialAsset::mercat_account_balance(owner.did, account_id.clone())
        .to_mercat::<TestStorage>()
        .unwrap();
    let stored_balance = secret_account
        .enc_keys
        .secret
        .decrypt(&stored_balance)
        .unwrap();

    assert_eq!(stored_balance, amount);

    (
        secret_account,
        account_id.clone(),
        ConfidentialAsset::mercat_accounts(owner.did, &account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
        ConfidentialAsset::mercat_account_balance(owner.did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap(),
    )
}

#[test]
fn issuers_can_create_and_rename_confidential_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        let funding_round_name: FundingRoundName = b"round1".into();
        // Expected token entry
        let token_name = vec![b'A'];
        let token = SecurityToken {
            owner_did: owner.did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token_name.as_slice()).unwrap();
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];

        // Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        // Since the total_supply is zero, the investor count should remain zero.
        assert_eq!(Statistics::investor_count(ticker), 0);

        // A correct entry is added.
        let token_with_zero_supply = SecurityToken {
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            ..Default::default()
        };
        assert_eq!(Asset::token_details(ticker), token_with_zero_supply);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );
        assert_eq!(Asset::funding_round(ticker), funding_round_name.clone());

        // Ticker is added to the list of confidential tokens.
        assert_eq!(
            ConfidentialAsset::confidential_tickers(),
            vec![AssetId {
                id: ticker.as_bytes().clone()
            }]
        );

        // Unauthorized identities cannot rename the token.
        let eve = User::new(AccountKeyring::Eve);
        assert_err!(
            Asset::rename_asset(eve.origin(), ticker, vec![0xde, 0xad, 0xbe, 0xef].into()),
            EAError::UnauthorizedAgent
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker), token_with_zero_supply);
        // Rename the token and check storage has been updated.
        let renamed_token_name = vec![0x42];
        let renamed_token = SecurityToken {
            owner_did: token.owner_did,
            total_supply: token_with_zero_supply.total_supply,
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            ..Default::default()
        };
        assert_ok!(Asset::rename_asset(
            owner.origin(),
            ticker,
            AssetName(renamed_token_name.clone())
        ));
        assert_eq!(Asset::token_details(ticker), renamed_token);
        assert_eq!(Asset::identifiers(ticker), identifiers);

        // Add another STO.
        // Expected token entry.
        let token_name = vec![b'B'];
        let token = SecurityToken {
            owner_did: owner.did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];
        let ticker2 = Ticker::try_from(token_name.as_slice()).unwrap();

        // Second Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker2,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        let token_with_zero_supply = SecurityToken {
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            ..Default::default()
        };

        // A correct entry is added.
        assert_eq!(Asset::token_details(ticker2), token_with_zero_supply);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker2),
            AssetOwnershipRelation::AssetOwned
        );
        assert_eq!(Asset::funding_round(ticker2), funding_round_name.clone());
        // Ticker is added to the list of confidential tokens.
        assert_eq!(
            ConfidentialAsset::confidential_tickers(),
            vec![
                AssetId {
                    id: ticker.as_bytes().clone()
                },
                AssetId {
                    id: ticker2.as_bytes().clone()
                }
            ]
        );
    });
}

#[test]
fn issuers_can_create_and_mint_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        // ------------ Setup

        // Alice is the owner of the token in this test.
        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let funding_round_name: FundingRoundName = b"round1".into();

        let token_names = [[b'A'], [b'B'], [b'C']];
        for token_name in token_names.iter() {
            create_confidential_token(
                &token_name[..],
                Ticker::try_from(&token_name[..]).unwrap(),
                bob, // Alice does not own any of these tokens.
            );
        }
        let total_supply: u128 = 10_000_000;
        // Expected token entry
        let token_name = vec![b'D'];
        let token = SecurityToken {
            owner_did: owner.did,
            total_supply,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token_name.as_slice()).unwrap();
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];

        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));
        enable_investor_count(ticker, owner);

        // In the initial call, the total_supply must be zero.
        assert_eq!(Asset::token_details(ticker).total_supply, Zero::zero());

        // ---------------- Setup: prepare for minting the asset

        let valid_asset_ids: Vec<AssetId> = ConfidentialAsset::confidential_tickers();

        let mut rng = StdRng::from_seed([10u8; 32]);
        let (secret_account, mercat_account_tx) = gen_account(
            &mut rng,
            &token_names[1][..],
            valid_asset_ids,
        );

        ConfidentialAsset::validate_mercat_account(
            owner.origin(),
            PubAccountTxWrapper::from(mercat_account_tx.clone()),
        )
        .unwrap();

        // ------------- START: Computations that will happen in Alice's Wallet ----------
        let amount: u32 = token.total_supply.try_into().unwrap(); // mercat amounts are 32 bit integers.
        let mut rng = StdRng::from_seed([42u8; 32]);
        let issuer_account = Account {
            secret: secret_account.clone(),
            public: mercat_account_tx.pub_account.clone(),
        };

        let initialized_asset_tx = AssetIssuer
            .initialize_asset_transaction(&issuer_account, &[], amount, &mut rng)
            .unwrap();

        // ------------- END: Computations that will happen in the Wallet ----------

        // Wallet submits the transaction to the chain for verification.
        ConfidentialAsset::mint_confidential_asset(
            owner.origin(),
            ticker,
            amount.into(), // convert to u128
            InitializedAssetTxWrapper::from(initialized_asset_tx),
        )
        .unwrap();

        // ------------------------- Ensuring that the asset details are set correctly
        // Check the update investor count for the newly created asset.
        assert_eq!(Statistics::investor_count(ticker), 1);

        // A correct entry is added.
        assert_eq!(Asset::token_details(ticker), token);
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

        // -------------------------- Ensure that the account balance is set properly.
        let account_id = MercatAccountId(
            EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id)
                .0
                .clone(),
        );

        let stored_balance = ConfidentialAsset::mercat_account_balance(owner.did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap();
        let stored_balance = secret_account
            .enc_keys
            .secret
            .decrypt(&stored_balance)
            .unwrap();

        assert_eq!(stored_balance, amount);
    })
}

#[test]
fn account_create_tx() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        // Simulating the case were issuers have registered some tickers and therefore the list of
        // valid asset ids contains some values.
        let token_names = [[b'A'], [b'B'], [b'C']];
        for token_name in token_names.iter() {
            create_confidential_token(
                &token_name[..],
                Ticker::try_from(&token_name[..]).unwrap(),
                bob,
            );
        }

        let valid_asset_ids: Vec<AssetId> = ConfidentialAsset::confidential_tickers();

        // ------------- START: Computations that will happen in Alice's Wallet ----------
        let mut rng = StdRng::from_seed([10u8; 32]);
        let (secret_account, mercat_account_tx) =
            gen_account(&mut rng, &token_names[1][..], valid_asset_ids);
        // ------------- END: Computations that will happen in the Wallet ----------

        // Wallet submits the transaction to the chain for verification.
        ConfidentialAsset::validate_mercat_account(
            alice.origin(),
            PubAccountTxWrapper::from(mercat_account_tx.clone()),
        )
        .unwrap();

        // Ensure that the transaction was verified and that MERCAT account is created on the chain.
        let wrapped_enc_asset_id =
            EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id);
        let account_id = MercatAccountId(wrapped_enc_asset_id.0.clone());
        let stored_account = ConfidentialAsset::mercat_accounts(alice.did, account_id.clone());

        assert_eq!(stored_account.encrypted_asset_id, wrapped_enc_asset_id);
        assert_eq!(
            stored_account
                .encryption_pub_key
                .to_mercat::<TestStorage>()
                .unwrap(),
            mercat_account_tx.pub_account.owner_enc_pub_key,
        );

        // Ensure that the account has an initial balance of zero.
        let stored_balance = ConfidentialAsset::mercat_account_balance(alice.did, account_id)
            .to_mercat::<TestStorage>()
            .unwrap();
        let stored_balance = secret_account
            .enc_keys
            .secret
            .decrypt(&stored_balance)
            .unwrap();
        assert_eq!(stored_balance, 0);
    });
}

// ----------------------------------------- Confidential transfer tests -----------------------------------

#[test]
fn basic_confidential_settlement() {
    test_with_cdd_provider(|_eve| {
        // The rest of rngs are built from it. Its initial value can be set using proptest.
        let mut rng = StdRng::from_seed([10u8; 32]);

        // Setting:
        //   - Alice is the token issuer.
        //   - Alice is also the sender of the token.
        //   - Bob is the receiver of the token.
        //   - Charlie is the mediator.
        //   - Eve is the CDD provider.
        let alice = User::new(AccountKeyring::Alice);

        let bob = User::new(AccountKeyring::Bob);

        let charlie = User::new(AccountKeyring::Charlie);

        // ------------ Setup mercat
        let token_name = b"ACME";
        let ticker = Ticker::try_from(&token_name[..]).unwrap();

        // Create an account for Alice and mint 10,000,000 tokens to ACME.
        // let total_supply = 1_1000_000;
        let total_supply = 500;
        let (
            alice_secret_account,
            alice_account_id,
            alice_public_account,
            alice_encrypted_init_balance,
        ) = create_account_and_mint_token(alice, total_supply, token_name.to_vec(), &mut rng);

        // Create accounts for Bob, and Charlie.
        let (bob_secret_account, bob_account_id, bob_public_account, bob_encrypted_init_balance) =
            init_account(&mut rng, token_name, bob);

        let (charlie_secret_account, _, charlie_public_account, _) =
            init_account(&mut rng, token_name, charlie);

        // Mediator creates a venue
        let venue_counter = Settlement::venue_counter();
        assert_ok!(Settlement::create_venue(
            charlie.origin(),
            VenueDetails::default(),
            vec![charlie.acc()],
            VenueType::Other
        ));

        // Mediator creates an instruction
        let instruction_counter = Settlement::instruction_counter();

        //// Provide scope claim to sender and receiver of the transaction.
        //provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], ticker, alice);
        // TODO: CRYP-172 I think we decided not to do this as it would leak the ticker name

        assert_ok!(Settlement::add_instruction(
            charlie.origin(),
            venue_counter,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg {
                from: PortfolioId::default_portfolio(alice.did),
                to: PortfolioId::default_portfolio(bob.did),
                kind: LegKind::Confidential(ConfidentialLeg {
                    mediator: PortfolioId::default_portfolio(charlie.did),
                    from_account_id: alice_account_id.clone(),
                    to_account_id: bob_account_id.clone(),
                }),
            }]
        ));

        // -------------------------- Perform the transfer
        let amount = 100u32; // This plain format is only used on functions that emulate the work of the wallet.

        println!("-------------> Checking if alice has enough funds.");
        // Ensure that Alice has minted enough tokens.
        assert!(
            alice_secret_account
                .enc_keys
                .secret
                .decrypt(&alice_encrypted_init_balance)
                .unwrap()
                > amount
        );

        // ----- Sender authorizes.
        // Sender computes the proofs in the wallet.
        println!("-------------> Alice is going to authorize.");
        let sender_data = CtxSender
            .create_transaction(
                &Account {
                    public: alice_public_account.clone(),
                    secret: alice_secret_account.clone(),
                },
                &alice_encrypted_init_balance,
                &bob_public_account,
                &charlie_public_account.owner_enc_pub_key,
                &[],
                amount,
                &mut rng,
            )
            .unwrap();
        let alice_encrypted_transfer_amount = sender_data.memo.enc_amount_using_sender;
        let bob_encrypted_transfer_amount = sender_data.memo.enc_amount_using_receiver;
        let initialized_tx =
            MercatTxData::InitializedTransfer(Base64Vec::new(sender_data.encode()));
        // Sender authorizes the instruction and passes in the proofs.
        assert_affirm_confidential_instruction!(
            alice.origin(),
            instruction_counter,
            initialized_tx,
            alice.did,
            1
        );

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
            CtxReceiver
                .finalize_transaction(
                    decoded_initialized_tx,
                    Account {
                        public: bob_public_account.clone(),
                        secret: bob_secret_account.clone(),
                    },
                    amount,
                    &mut rng,
                )
                .unwrap()
                .encode(),
        ));

        // Receiver submits the proof to the chain.
        assert_affirm_confidential_instruction!(
            bob.origin(),
            instruction_counter,
            finalized_tx,
            bob.did,
            1
        );

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
            CtxMediator
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
        assert_affirm_confidential_instruction!(
            charlie.origin(),
            instruction_counter,
            justified_tx,
            charlie.did,
            1
        );

        next_block();

        // Instruction should've settled.
        // Verify by decrypting the new balance of both Alice and Bob.
        let new_alice_balance =
            ConfidentialAsset::mercat_account_balance(alice.did, alice_account_id)
                .to_mercat::<TestStorage>()
                .unwrap();
        let expected_alice_balance = alice_encrypted_init_balance - alice_encrypted_transfer_amount;
        assert_eq!(new_alice_balance, expected_alice_balance);

        let new_alice_balance = alice_secret_account
            .enc_keys
            .secret
            .decrypt(&new_alice_balance)
            .unwrap();
        assert_eq!(new_alice_balance as u128, total_supply - amount as u128);

        let new_bob_balance = ConfidentialAsset::mercat_account_balance(bob.did, bob_account_id)
            .to_mercat::<TestStorage>()
            .unwrap();

        let expected_bob_balance = bob_encrypted_init_balance + bob_encrypted_transfer_amount;
        assert_eq!(new_bob_balance, expected_bob_balance);
        let new_bob_balance = bob_secret_account
            .enc_keys
            .secret
            .decrypt(&new_bob_balance)
            .unwrap();
        assert_eq!(new_bob_balance, amount);
    });
}
