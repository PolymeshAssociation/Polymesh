use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use codec::{Decode, Encode};
use confidential_asset::EncryptedAssetIdWrapper;
use core::convert::TryFrom;
use cryptography::{
    asset_proofs::{CommitmentWitness, ElgamalSecretKey},
    mercat::{
        account::{convert_asset_ids, AccountCreator},
        AccountCreatorInitializer, EncryptedAmount, EncryptionKeys, SecAccount,
    },
    AssetId,
};
use curve25519_dalek::scalar::Scalar;
use frame_support::IterableStorageMap;
use pallet_asset as asset;
use pallet_confidential_asset as confidential_asset;
use pallet_statistics as statistics;
use rand::{rngs::StdRng, SeedableRng};
use test_client::AccountKeyring;

use frame_support::{assert_err, assert_ok, StorageDoubleMap, StorageMap};
use pallet_identity as identity;
use polymesh_primitives::{
    AssetOwnershipRelation, AssetType, FundingRoundName, IdentifierType, SecurityToken, Ticker,
};
use sp_core::sr25519::Public;

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
        100_000,
        true,
        AssetType::default(),
        vec![],
        None,
    ));
}

#[test]
fn issuers_can_create_and_rename_confidential_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let funding_round_name: FundingRoundName = b"round1".into();
        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: Some(owner_did),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        let identifiers = vec![(IdentifierType::default(), b"undefined".into())];
        assert_err!(
            ConfidentialAsset::create_confidential_asset(
                owner_signed.clone(),
                token.name.clone(),
                ticker,
                1_000_000_000_000_000_000_000_000, // Total supply over the limit
                true,
                token.asset_type.clone(),
                identifiers.clone(),
                Some(funding_round_name.clone()),
            ),
            AssetError::TotalSupplyAboveLimit
        );

        // Ticker is not added to the list of confidential tokens.
        assert_eq!(ConfidentialAsset::confidential_tickers(), vec![]);

        // Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        // A correct entry is added.
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );
        assert!(<DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
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
            Asset::rename_asset(eve_signed, ticker, b"ABCD".into()),
            AssetError::Unauthorized
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker), token);
        // Rename the token and check storage has been updated.
        let renamed_token = SecurityToken {
            name: vec![0x42].into(),
            owner_did: token.owner_did,
            total_supply: token.total_supply,
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
            primary_issuance_agent: Some(token.owner_did),
            ..Default::default()
        };
        assert_ok!(Asset::rename_asset(
            owner_signed.clone(),
            ticker,
            renamed_token.name.clone()
        ));
        assert_eq!(Asset::token_details(ticker), renamed_token);
        for (typ, val) in identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }

        // Add another STO.
        // Expected token entry.
        let token = SecurityToken {
            name: vec![0x02].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: Some(owner_did),
            ..Default::default()
        };
        let identifiers = vec![(IdentifierType::default(), b"undefined".into())];
        let ticker2 = Ticker::try_from(token.name.as_slice()).unwrap();

        // Second Issuance is successful.
        assert_ok!(ConfidentialAsset::create_confidential_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker2,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone()),
        ));

        // A correct entry is added.
        assert_eq!(Asset::token_details(ticker2), token);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker2),
            AssetOwnershipRelation::AssetOwned
        );
        assert!(<DidRecords>::contains_key(
            Identity::get_token_did(&ticker2).unwrap()
        ));
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
fn account_create_tx() {
    ExtBuilder::default().build().execute_with(|| {
        // Simulating the case were issuers have registered some tickers and therefore the list of
        // valid asset ids contains some values.
        let token_names = [[2u8], [1u8], [5u8]];
        for token_name in token_names.iter() {
            create_confidential_token(
                &token_name[..],
                Ticker::try_from(&token_name[..]).unwrap(),
                AccountKeyring::Bob.public(),
            );
        }

        let valid_asset_ids = ConfidentialAsset::confidential_tickers();

        // ------------- START: Computations that will happen in Alice's Wallet ----------
        let alice = AccountKeyring::Alice.public();
        let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();

        // These are the encryptions keys used by MERCAT and are different from the signing keys
        // that Polymesh uses.
        let mut rng = StdRng::from_seed([10u8; 32]);
        let elg_secret = ElgamalSecretKey::new(Scalar::random(&mut rng));
        let elg_pub = elg_secret.get_public_key();
        let enc_keys = EncryptionKeys {
            pblc: elg_pub.into(),
            scrt: elg_secret.into(),
        };

        // Positive test: use one of the registered asset ids.
        let asset_id = AssetId {
            id: *Ticker::try_from(&token_names[1][..]).unwrap().as_bytes(),
        };
        let asset_id_witness = CommitmentWitness::from((asset_id.clone().into(), &mut rng));
        let scrt_account = SecAccount {
            enc_keys,
            asset_id_witness,
        };

        let valid_asset_ids = convert_asset_ids(valid_asset_ids);
        let account_id = 0;
        let mercat_tx_id = 0;
        let mercat_account_tx = AccountCreator {}
            .create(
                mercat_tx_id,
                &scrt_account,
                &valid_asset_ids,
                account_id,
                &mut rng,
            )
            .unwrap();

        // ------------- END: Computations that will happen in the Wallet ----------

        // Wallet submits the transaction to the chain for verification.
        ConfidentialAsset::validate_mercat_account(
            Origin::signed(alice),
            mercat_account_tx.clone(),
        )
        .unwrap();

        // Ensure that the transaction was verified and that MERCAT account is created on the chain.
        let wrapped_enc_asset_id =
            EncryptedAssetIdWrapper::from(mercat_account_tx.pub_account.enc_asset_id.encode());
        let stored_account =
            ConfidentialAsset::mercat_accounts(alice_id, wrapped_enc_asset_id.clone());

        assert_eq!(stored_account.encrypted_asset_id, wrapped_enc_asset_id,);
        assert_eq!(
            stored_account.encryption_pub_key,
            mercat_account_tx.pub_account.owner_enc_pub_key,
        );

        // Ensure that the account has an initial balance of zero.
        let stored_balance =
            ConfidentialAsset::mercat_account_balance(alice_id, wrapped_enc_asset_id.clone());
        let stored_balance = EncryptedAmount::decode(&mut &stored_balance.0[..]).unwrap();
        let stored_balance = scrt_account.enc_keys.scrt.decrypt(&stored_balance).unwrap();
        assert_eq!(stored_balance, 0);
    });
}
