use super::{
    storage::{TestStorage, User},
    ExtBuilder,
};
use core::convert::TryInto;
use frame_support::assert_ok;

use mercat::{
    account::AccountCreator,
    asset::AssetIssuer,
    confidential_identity_core::{
        asset_proofs::ElgamalSecretKey, curve25519_dalek::scalar::Scalar,
    },
    transaction::{CtxMediator, CtxReceiver, CtxSender},
    Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptedAmount, EncryptionKeys,
    PubAccount, PubAccountTx, SecAccount, TransferTransactionMediator, TransferTransactionReceiver,
    TransferTransactionSender,
};
use pallet_confidential_asset::{
    ConfidentialAssetDetails, InitializedAssetTxWrapper, MercatAccount, PubAccountTxWrapper,
    TransactionLeg, TransactionLegId, TransactionLegProofs, VenueId,
};
use polymesh_primitives::{
    asset::{AssetName, AssetType},
    Ticker,
};
use rand::{rngs::StdRng, SeedableRng};
use sp_runtime::traits::Zero;
use test_client::AccountKeyring;

type ConfidentialAsset = pallet_confidential_asset::Module<TestStorage>;

macro_rules! assert_affirm_confidential_transaction {
    ($signer:expr, $transaction_id:expr, $data:expr) => {
        assert_ok!(ConfidentialAsset::affirm_transaction(
            $signer,
            $transaction_id,
            TransactionLegId(0),
            $data,
        ));
    };
}

fn create_confidential_token(token_name: &[u8], ticker: Ticker, user: User) {
    assert_ok!(ConfidentialAsset::create_confidential_asset(
        user.origin(),
        AssetName(token_name.into()),
        ticker,
        AssetType::default(),
    ));
}

/// Creates a mercat account and returns its secret part (to be stored in the wallet) and
/// the account creation proofs (to be submitted to the chain).
pub fn gen_account(mut rng: &mut StdRng) -> (SecAccount, PubAccountTx) {
    // These are the encryptions keys used by MERCAT and are different from the signing keys
    // that Polymesh uses.
    let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
    let elg_pub = elg_secret.get_public_key();
    let enc_keys = EncryptionKeys {
        public: elg_pub.into(),
        secret: elg_secret.into(),
    };

    let secret_account = SecAccount { enc_keys };
    let mercat_account_tx = AccountCreator.create(&secret_account, &mut rng).unwrap();

    (secret_account, mercat_account_tx)
}

/// Creates a mercat account for the `owner` and submits the proofs to the chain and validates them.
/// It then return the secret part of the account, the account id, the public portion of the account and the initial
/// encrypted balance of zero.
pub fn init_account(
    mut rng: &mut StdRng,
    ticker: Ticker,
    owner: User,
) -> (SecAccount, MercatAccount, PubAccount, EncryptedAmount) {
    let (secret_account, mercat_account_tx) = gen_account(&mut rng);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        owner.origin(),
        ticker,
        PubAccountTxWrapper::from(mercat_account_tx.clone())
    ));

    let pub_account = &mercat_account_tx.pub_account;
    let account: MercatAccount = pub_account.into();
    let balance = *ConfidentialAsset::mercat_account_balance(&account, ticker);
    (secret_account, account, pub_account.clone(), balance)
}

/// Performs mercat account creation, validation, and minting of the account with `total_supply` tokens.
/// It returns the next the secret portion of the account, the account id, the public portion of the account,
/// and the encrypted balance of `total_supply`.
pub fn create_account_and_mint_token(
    owner: User,
    total_supply: u128,
    token_name: Vec<u8>,
    mut rng: &mut StdRng,
) -> (SecAccount, MercatAccount, PubAccount, EncryptedAmount) {
    let token = ConfidentialAssetDetails {
        total_supply,
        owner_did: owner.did,
        asset_type: AssetType::default(),
    };
    let ticker = Ticker::from_slice_truncated(token_name.as_slice());

    assert_ok!(ConfidentialAsset::create_confidential_asset(
        owner.origin(),
        AssetName(token_name.clone()),
        ticker,
        token.asset_type.clone(),
    ));

    // In the initial call, the total_supply must be zero.
    assert_eq!(
        ConfidentialAsset::confidential_asset_details(ticker).total_supply,
        Zero::zero()
    );

    // ---------------- prepare for minting the asset

    let (secret_account, mercat_account_tx) = gen_account(&mut rng);

    assert_ok!(ConfidentialAsset::validate_mercat_account(
        owner.origin(),
        ticker,
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
        ConfidentialAsset::confidential_asset_details(ticker).owner_did,
        token.owner_did
    );

    // -------------------------- Ensure the encrypted balance matches the minted amount.
    let pub_account = &mercat_account_tx.pub_account;
    let account: MercatAccount = pub_account.into();
    let balance = *ConfidentialAsset::mercat_account_balance(&account, ticker);
    let stored_balance = secret_account.enc_keys.secret.decrypt(&balance).unwrap();

    assert_eq!(stored_balance, amount);

    (secret_account, account, pub_account.clone(), balance)
}

#[test]
fn issuers_can_create_and_rename_confidential_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);
        // Expected token entry
        let token_name = vec![b'A'];
        let token = ConfidentialAssetDetails {
            owner_did: owner.did,
            total_supply: 1_000_000,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice_truncated(token_name.as_slice());

        // Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker,
            token.asset_type.clone(),
        ));

        // A correct entry is added.
        let token_with_zero_supply = ConfidentialAssetDetails {
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            asset_type: token.asset_type.clone(),
        };
        assert_eq!(
            ConfidentialAsset::confidential_asset_details(ticker),
            token_with_zero_supply
        );

        /*
        // Unauthorized identities cannot rename the token.
        let eve = User::new(AccountKeyring::Eve);
        assert_err!(
            Asset::rename_asset(eve.origin(), ticker, vec![0xde, 0xad, 0xbe, 0xef].into()),
            EAError::UnauthorizedAgent
        );
        // The token should remain unchanged in storage.
        assert_eq!(ConfidentialAsset::confidential_asset_details(ticker), token_with_zero_supply);
        // Rename the token and check storage has been updated.
        let renamed_token_name = vec![0x42];
        let renamed_token = ConfidentialAssetDetails {
            owner_did: token.owner_did,
            total_supply: token_with_zero_supply.total_supply,
            asset_type: token.asset_type.clone(),
        };
        assert_ok!(Asset::rename_asset(
            owner.origin(),
            ticker,
            AssetName(renamed_token_name.clone())
        ));
        assert_eq!(ConfidentialAsset::confidential_asset_details(ticker), renamed_token);
        */

        // Add another STO.
        // Expected token entry.
        let token_name = vec![b'B'];
        let token = ConfidentialAssetDetails {
            owner_did: owner.did,
            total_supply: 1_000_000,
            asset_type: AssetType::default(),
        };
        let ticker2 = Ticker::from_slice_truncated(token_name.as_slice());

        // Second Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker2,
            token.asset_type.clone(),
        ));

        let token_with_zero_supply = ConfidentialAssetDetails {
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            asset_type: token.asset_type.clone(),
        };

        // A correct entry is added.
        assert_eq!(
            ConfidentialAsset::confidential_asset_details(ticker2),
            token_with_zero_supply
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

        let token_names = [[b'A'], [b'B'], [b'C']];
        for token_name in token_names.iter() {
            create_confidential_token(
                &token_name[..],
                Ticker::from_slice_truncated(&token_name[..]),
                bob, // Alice does not own any of these tokens.
            );
        }
        let total_supply: u128 = 10_000_000;
        // Expected token entry
        let token_name = vec![b'D'];
        let token = ConfidentialAssetDetails {
            owner_did: owner.did,
            total_supply,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice_truncated(token_name.as_slice());

        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner.origin(),
            AssetName(token_name.clone()),
            ticker,
            token.asset_type.clone(),
        ));

        // In the initial call, the total_supply must be zero.
        assert_eq!(
            ConfidentialAsset::confidential_asset_details(ticker).total_supply,
            Zero::zero()
        );

        // ---------------- Setup: prepare for minting the asset

        let mut rng = StdRng::from_seed([10u8; 32]);
        let (secret_account, mercat_account_tx) = gen_account(&mut rng);

        ConfidentialAsset::validate_mercat_account(
            owner.origin(),
            ticker,
            PubAccountTxWrapper::from(mercat_account_tx.clone()),
        )
        .unwrap();
        let pub_account = mercat_account_tx.pub_account;
        let account: MercatAccount = pub_account.clone().into();

        // ------------- START: Computations that will happen in Alice's Wallet ----------
        let amount: u32 = token.total_supply.try_into().unwrap(); // mercat amounts are 32 bit integers.
        let issuer_account = Account {
            secret: secret_account.clone(),
            public: pub_account,
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

        // A correct entry is added.
        assert_eq!(ConfidentialAsset::confidential_asset_details(ticker), token);

        // -------------------------- Ensure that the account balance is set properly.
        let balance = *ConfidentialAsset::mercat_account_balance(&account, ticker);

        secret_account
            .enc_keys
            .secret
            .verify(&balance, &amount.into())
            .expect("verify new balance");
    })
}

#[test]
fn account_create_tx() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        // Simulating the case were issuers have registered some tickers and therefore the list of
        // valid asset ids contains some values.
        let ticker = Ticker::from_slice_truncated(b"A".as_ref());

        // ------------- START: Computations that will happen in Alice's Wallet ----------
        let mut rng = StdRng::from_seed([10u8; 32]);
        let (secret_account, mercat_account_tx) = gen_account(&mut rng);
        // ------------- END: Computations that will happen in the Wallet ----------

        // Wallet submits the transaction to the chain for verification.
        ConfidentialAsset::validate_mercat_account(
            alice.origin(),
            ticker,
            PubAccountTxWrapper::from(mercat_account_tx.clone()),
        )
        .unwrap();

        // Ensure that the transaction was verified and that MERCAT account is created on the chain.
        let pub_account = &mercat_account_tx.pub_account;
        let account: MercatAccount = pub_account.into();

        assert_eq!(
            *account.pub_key,
            mercat_account_tx.pub_account.owner_enc_pub_key,
        );

        // Ensure that the account has an initial balance of zero.
        let stored_balance = *ConfidentialAsset::mercat_account_balance(&account, ticker);
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
    let cdd = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![cdd.clone()])
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
            let alice = User::new(AccountKeyring::Alice);

            let bob = User::new(AccountKeyring::Bob);

            let charlie = User::new(AccountKeyring::Charlie);

            // ------------ Setup mercat
            let token_name = b"ACME";
            let ticker = Ticker::from_slice_truncated(&token_name[..]);

            // Create an account for Alice and mint 10,000,000 tokens to ACME.
            // let total_supply = 1_1000_000;
            let total_supply = 500;
            let (
                alice_secret_account,
                alice_account,
                alice_public_account,
                alice_encrypted_init_balance,
            ) = create_account_and_mint_token(alice, total_supply, token_name.to_vec(), &mut rng);

            // Create accounts for Bob, and Charlie.
            let (bob_secret_account, bob_account, bob_public_account, bob_encrypted_init_balance) =
                init_account(&mut rng, ticker, bob);

            let (charlie_secret_account, _, charlie_public_account, _) =
                init_account(&mut rng, ticker, charlie);

            // Mediator creates a venue
            let venue_counter = VenueId(0); /*ConfidentialAsset::venue_counter();
                                            assert_ok!(ConfidentialAsset::create_venue(
                                                charlie.origin(),
                                                VenueDetails::default(),
                                                vec![charlie.acc()],
                                                VenueType::Other
                                            ));
                                            */

            // Mediator creates an transaction
            let transaction_counter = ConfidentialAsset::transaction_counter();

            //// Provide scope claim to sender and receiver of the transaction.
            //provide_scope_claim_to_multiple_parties(&[alice.did, bob.did], ticker, alice);
            // TODO: CRYP-172 I think we decided not to do this as it would leak the ticker name

            assert_ok!(ConfidentialAsset::add_transaction(
                charlie.origin(),
                venue_counter,
                vec![TransactionLeg {
                    ticker,
                    sender: alice_account.clone(),
                    receiver: bob_account.clone(),
                    mediator: charlie.did,
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
            let sender_tx = CtxSender
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
            let alice_encrypted_transfer_amount = sender_tx.memo.enc_amount_using_sender;
            let bob_encrypted_transfer_amount = sender_tx.memo.enc_amount_using_receiver;
            let initialized_tx = TransactionLegProofs::new_sender(sender_tx);
            // Sender authorizes the transaction and passes in the proofs.
            assert_affirm_confidential_transaction!(
                alice.origin(),
                transaction_counter,
                initialized_tx
            );

            // ------ Receiver authorizes.
            // Receiver reads the sender's proof from the chain.
            println!("-------------> Bob is going to authorize.");
            let tx_data =
                ConfidentialAsset::transaction_proofs(transaction_counter, TransactionLegId(0));
            let init_tx = match tx_data {
                TransactionLegProofs {
                    sender: Some(init),
                    receiver: None,
                    mediator: None,
                } => init,
                _ => {
                    println!("{:?}", tx_data);
                    panic!("Unexpected data type");
                }
            };

            // Receiver computes the proofs in the wallet.
            let finalized_tx = TransactionLegProofs::new_receiver(
                CtxReceiver
                    .finalize_transaction(
                        &init_tx,
                        Account {
                            public: bob_public_account.clone(),
                            secret: bob_secret_account.clone(),
                        },
                        amount,
                    )
                    .unwrap(),
            );

            // Receiver submits the proof to the chain.
            assert_affirm_confidential_transaction!(
                bob.origin(),
                transaction_counter,
                finalized_tx
            );

            // ------ Mediator authorizes.
            // Mediator reads the receiver's proofs from the chain (it contains the sender's proofs as well).
            println!("-------------> Charlie is going to authorize.");
            let tx_data =
                ConfidentialAsset::transaction_proofs(transaction_counter, TransactionLegId(0));
            let (init_tx, finalized_tx) = match tx_data {
                TransactionLegProofs {
                    sender: Some(init),
                    receiver: Some(finalized),
                    mediator: None,
                } => (init, finalized),
                _ => {
                    panic!("Unexpected data type");
                }
            };

            // Mediator verifies the proofs in the wallet.
            let justified_tx = TransactionLegProofs::new_mediator(
                CtxMediator
                    .justify_transaction(
                        &init_tx,
                        &finalized_tx,
                        &charlie_secret_account.enc_keys,
                        &alice_public_account,
                        &alice_encrypted_init_balance,
                        &bob_public_account,
                        &[],
                        &mut rng,
                    )
                    .unwrap(),
            );

            println!("-------------> This should trigger the execution");
            assert_affirm_confidential_transaction!(
                charlie.origin(),
                transaction_counter,
                justified_tx
            );

            // Execute affirmed transaction.
            assert_ok!(ConfidentialAsset::execute_transaction(
                charlie.origin(),
                transaction_counter,
                1,
            ));

            // Transaction should've settled.
            // Verify by decrypting the new balance of both Alice and Bob.
            let new_alice_balance =
                *ConfidentialAsset::mercat_account_balance(&alice_account, ticker);
            let expected_alice_balance =
                alice_encrypted_init_balance - alice_encrypted_transfer_amount;
            assert_eq!(new_alice_balance, expected_alice_balance);

            let new_alice_balance = alice_secret_account
                .enc_keys
                .secret
                .decrypt(&new_alice_balance)
                .unwrap();
            assert_eq!(new_alice_balance as u128, total_supply - amount as u128);

            let new_bob_balance = *ConfidentialAsset::mercat_account_balance(&bob_account, ticker);

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
