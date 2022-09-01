use super::{
    storage::{register_keyring_account, TestStorage},
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
    cryptography_core::{
        asset_proofs::{CommitmentWitness, ElgamalSecretKey},
        curve25519_dalek::scalar::Scalar,
        AssetId,
    },
    Account, AccountCreatorInitializer, AssetTransactionIssuer, EncryptionKeys, PubAccountTx,
    SecAccount,
};
use pallet_asset as asset;
use pallet_confidential_asset as confidential_asset;
use pallet_identity as identity;
use pallet_statistics as statistics;
use polymesh_primitives::{
    asset::{AssetOwnershipRelation, AssetType, FundingRoundName, SecurityToken},
    AssetIdentifier, Ticker,
};
use rand::{rngs::StdRng, SeedableRng};
use sp_core::sr25519::Public;
use sp_runtime::traits::Zero;
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type DidRecords = identity::DidRecords<TestStorage>;
type Statistics = statistics::Module<TestStorage>;
type System = frame_system::Module<TestStorage>;
type ConfidentialAsset = confidential_asset::Module<TestStorage>;

fn create_confidential_token(token_name: &[u8], ticker: Ticker, keyring: Public) {
    assert_ok!(ConfidentialAsset::create_confidential_asset(
        Origin::signed(keyring),
        token_name.into(),
        ticker,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
}

fn gen_account(
    seed: [u8; 32],
    token_name: &[u8],
    valid_asset_ids: Vec<AssetId>,
) -> (SecAccount, PubAccountTx) {
    // These are the encryptions keys used by MERCAT and are different from the signing keys
    // that Polymesh uses.
    let mut rng = StdRng::from_seed(seed);
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

#[test]
fn issuers_can_create_and_rename_confidential_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let funding_round_name: FundingRoundName = b"round1".into();
        // Expected token entry
        let token = SecurityToken {
            name: vec![b'A'].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: None,
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];

        // Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner_signed.clone(),
            token.name.clone(),
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
            name: token.name,
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            primary_issuance_agent: None,
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
        let eve_signed = Origin::signed(AccountKeyring::Eve.public());
        let _eve_did = register_keyring_account(AccountKeyring::Eve).unwrap();
        assert_err!(
            Asset::rename_asset(eve_signed, ticker, vec![0xde, 0xad, 0xbe, 0xef].into()),
            AssetError::Unauthorized
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker), token_with_zero_supply);
        // Rename the token and check storage has been updated.
        let renamed_token = SecurityToken {
            name: vec![0x42].into(),
            owner_did: token.owner_did,
            total_supply: token_with_zero_supply.total_supply,
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            primary_issuance_agent: None,
            ..Default::default()
        };
        assert_ok!(Asset::rename_asset(
            owner_signed.clone(),
            ticker,
            renamed_token.name.clone()
        ));
        assert_eq!(Asset::token_details(ticker), renamed_token);
        assert_eq!(Asset::identifiers(ticker), identifiers);

        // Add another STO.
        // Expected token entry.
        let token = SecurityToken {
            name: vec![b'B'].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: None,
            ..Default::default()
        };
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];
        let ticker2 = Ticker::try_from(token.name.as_slice()).unwrap();

        // Second Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker2,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        let token_with_zero_supply = SecurityToken {
            name: token.name,
            owner_did: token.owner_did,
            total_supply: Zero::zero(),
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            primary_issuance_agent: None,
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
        let owner = AccountKeyring::Alice.public();
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob = AccountKeyring::Bob.public();
        let _bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
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
        let token = SecurityToken {
            name: vec![b'D'].into(),
            owner_did,
            total_supply,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: None,
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        let identifier_value1 = b"037833100";
        let identifiers = vec![AssetIdentifier::cusip(*identifier_value1).unwrap()];

        assert_ok!(ConfidentialAsset::create_confidential_asset(
            Origin::signed(owner),
            token.name.clone(),
            ticker,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        // In the initial call, the total_supply must be zero.
        assert_eq!(Asset::token_details(ticker).total_supply, Zero::zero());

        // ---------------- Setup: prepare for minting the asset

        let valid_asset_ids: Vec<AssetId> = ConfidentialAsset::confidential_tickers();

        let (secret_account, mercat_account_tx) = gen_account(
            [10u8; 32], // seed
            &token_names[1][..],
            valid_asset_ids,
        );

        ConfidentialAsset::validate_mercat_account(
            Origin::signed(owner),
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
            Origin::signed(owner),
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

        let stored_balance = ConfidentialAsset::mercat_account_balance(owner_did, account_id)
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
        let alice = AccountKeyring::Alice.public();
        let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob = AccountKeyring::Bob.public();
        let _bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
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
        let (secret_account, mercat_account_tx) =
            gen_account([10u8; 32], &token_names[1][..], valid_asset_ids);
        // ------------- END: Computations that will happen in the Wallet ----------

        // Wallet submits the transaction to the chain for verification.
        ConfidentialAsset::validate_mercat_account(
            Origin::signed(alice),
            PubAccountTxWrapper::from(mercat_account_tx.clone()),
        )
        .unwrap();

        // Ensure that the transaction was verified and that MERCAT account is created on the chain.
        let wrapped_enc_asset_id =
            EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id);
        let account_id = MercatAccountId(wrapped_enc_asset_id.0.clone());
        let stored_account = ConfidentialAsset::mercat_accounts(alice_id, account_id.clone());

        assert_eq!(stored_account.encrypted_asset_id, wrapped_enc_asset_id);
        assert_eq!(
            stored_account
                .encryption_pub_key
                .to_mercat::<TestStorage>()
                .unwrap(),
            mercat_account_tx.pub_account.owner_enc_pub_key,
        );

        // Ensure that the account has an initial balance of zero.
        let stored_balance = ConfidentialAsset::mercat_account_balance(alice_id, account_id)
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
