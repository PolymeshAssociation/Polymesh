use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use codec::Decode;
use codec::Encode;
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
use frame_support::assert_ok;
use pallet_asset::{self as asset, AssetType, IdentifierType, SecurityToken};
use pallet_confidential_asset as confidential_asset;
use polymesh_primitives::Ticker;
use rand::{rngs::StdRng, SeedableRng};
use test_client::AccountKeyring;

type ConfidentialAsset = confidential_asset::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn account_create_tx() {
    ExtBuilder::default().build().execute_with(|| {
        // Simulating the case were issuers have registered some tickers and therefore the list of
        // valid asset ids contains some values.
        let issuer = AccountKeyring::Bob.public();

        let valid_asset_ids: Vec<AssetId> =
            vec![2, 1, 5].iter().cloned().map(AssetId::from).collect();

        ConfidentialAsset::create_confidential_asset(
            Origin::signed(issuer),
            valid_asset_ids.clone(),
        )
        .unwrap();

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

        let asset_id = AssetId::from(2); // Positive test: 2 is one of the registered asset ids.
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
            ConfidentialAsset::mercat_account(alice_id, wrapped_enc_asset_id.clone());

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
