use crate::{
    asset::{self, AssetType, IdentifierType, SecurityToken, SignData},
    balances, general_tm, identity,
    test::storage::{build_ext, make_account, TestStorage},
};
use primitives::{AuthorizationData, IdentityId, Signer, Ticker};

use codec::Encode;
use frame_support::{assert_err, assert_noop, assert_ok, traits::Currency, StorageMap};
use sp_runtime::AnySignature;
use test_client::AccountKeyring;

use chrono::prelude::Utc;
use rand::Rng;
use std::convert::TryFrom;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type GeneralTM = general_tm::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;

type OffChainSignature = AnySignature;

#[test]
fn issuers_can_create_and_rename_tokens() {
    build_ext().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice(token.name.as_slice());
        assert!(!<identity::DidRecords>::exists(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifiers = vec![(IdentifierType::default(), b"undefined".to_vec())];
        let ticker = Ticker::from_slice(token.name.as_slice());
        assert_err!(
            Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                1_000_000_000_000_000_000_000_000, // Total supply over the limit
                true,
                token.asset_type.clone(),
                identifiers.clone(),
            ),
            "Total supply above the limit"
        );

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
        ));

        // A correct entry is added
        assert_eq!(Asset::token_details(ticker), token);
        assert!(<identity::DidRecords>::exists(
            Identity::get_token_did(&ticker).unwrap()
        ));
        assert_eq!(Asset::token_details(ticker), token);

        // Unauthorized identities cannot rename the token.
        let (eve_signed, _eve_did) = make_account(AccountKeyring::Eve.public()).unwrap();
        assert_err!(
            Asset::rename_token(eve_signed, ticker, vec![0xde, 0xad, 0xbe, 0xef]),
            "sender must be a signing key for the token owner DID"
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker), token);
        // Rename the token and check storage has been updated.
        let renamed_token = SecurityToken {
            name: vec![0x42],
            owner_did: token.owner_did,
            total_supply: token.total_supply,
            divisible: token.divisible,
            asset_type: token.asset_type.clone(),
        };
        assert_ok!(Asset::rename_token(
            owner_signed.clone(),
            ticker,
            renamed_token.name.clone()
        ));
        assert_eq!(Asset::token_details(ticker), renamed_token);
        for (typ, val) in identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }
    });
}

/// # TODO
/// It should be re-enable once issuer claim is re-enabled.
#[test]
#[ignore]
fn non_issuers_cant_create_tokens() {
    build_ext().execute_with(|| {
        let (_, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let _ = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };

        Balances::make_free_balance_be(&AccountKeyring::Bob.public(), 1_000_000);

        let wrong_did = IdentityId::try_from("did:poly:wrong");
        assert!(wrong_did.is_err());
    });
}

#[test]
fn valid_transfers_pass() {
    build_ext().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice(token.name.as_slice());

        let (_, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
        ));

        // A correct entry is added
        assert_eq!(Asset::token_details(ticker), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            ticker,
            asset_rule
        ));

        assert_ok!(Asset::transfer(
            owner_signed.clone(),
            owner_did,
            ticker,
            alice_did,
            500
        ));
    })
}

#[test]
fn valid_custodian_allowance() {
    build_ext().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice(token.name.as_slice());

        let (investor1_signed, investor1_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let (investor2_signed, investor2_did) =
            make_account(AccountKeyring::Charlie.public()).unwrap();
        let (custodian_signed, custodian_did) = make_account(AccountKeyring::Eve.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
        ));

        assert_eq!(
            Asset::balance_of((ticker, token.owner_did)),
            token.total_supply
        );

        assert_eq!(Asset::token_details(ticker), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            ticker,
            asset_rule
        ));
        let funding_round1 = b"Round One".to_vec();
        assert_ok!(Asset::set_funding_round(
            owner_signed.clone(),
            owner_did,
            ticker,
            funding_round1.clone()
        ));
        // Mint some tokens to investor1
        let num_tokens1: u128 = 2_000_000;
        assert_ok!(Asset::issue(
            owner_signed.clone(),
            owner_did,
            ticker,
            investor1_did,
            num_tokens1,
            vec![0x0]
        ));
        assert_eq!(Asset::funding_round(&ticker), funding_round1.clone());
        assert_eq!(
            Asset::issued_in_funding_round((ticker, funding_round1.clone())),
            num_tokens1
        );
        // Check the expected default behaviour of the map.
        assert_eq!(
            Asset::issued_in_funding_round((ticker, b"No such round".to_vec())),
            0
        );
        assert_eq!(Asset::balance_of((ticker, investor1_did)), num_tokens1,);

        // Failed to add custodian because of insufficient balance
        assert_noop!(
            Asset::increase_custody_allowance(
                investor1_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                250_00_00 as u128
            ),
            "Insufficient balance of holder did"
        );

        // Failed to add/increase the custodian allowance because of Invalid custodian did
        let custodian_did_not_register = IdentityId::from(5u128);
        assert_noop!(
            Asset::increase_custody_allowance(
                investor1_signed.clone(),
                ticker,
                investor1_did,
                custodian_did_not_register,
                50_00_00 as u128
            ),
            "Invalid custodian DID"
        );

        // Add custodian
        assert_ok!(Asset::increase_custody_allowance(
            investor1_signed.clone(),
            ticker,
            investor1_did,
            custodian_did,
            50_00_00 as u128
        ));

        assert_eq!(
            Asset::custodian_allowance((ticker, investor1_did, custodian_did)),
            50_00_00 as u128
        );

        assert_eq!(
            Asset::total_custody_allowance((ticker, investor1_did)),
            50_00_00 as u128
        );

        // Transfer the token upto the limit
        assert_ok!(Asset::transfer(
            investor1_signed.clone(),
            investor1_did,
            ticker,
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(
            Asset::balance_of((ticker, investor2_did)),
            140_00_00 as u128
        );

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                ticker,
                investor2_did,
                50_00_00 as u128
            ),
            "Insufficient balance for transfer"
        );

        // Should fail to transfer the token by the custodian because of invalid signing key
        assert_noop!(
            Asset::transfer_by_custodian(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                45_00_00 as u128
            ),
            "sender must be a signing key for DID"
        );

        // Should fail to transfer the token by the custodian because of insufficient allowance
        assert_noop!(
            Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                55_00_00 as u128
            ),
            "Insufficient allowance"
        );

        // Successfully transfer by the custodian
        assert_ok!(Asset::transfer_by_custodian(
            custodian_signed.clone(),
            ticker,
            investor1_did,
            custodian_did,
            investor2_did,
            45_00_00 as u128
        ));
    });
}

#[test]
fn valid_custodian_allowance_of() {
    build_ext().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice(token.name.as_slice());

        let (investor1_signed, investor1_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let (investor2_signed, investor2_did) =
            make_account(AccountKeyring::Charlie.public()).unwrap();
        let (custodian_signed, custodian_did) = make_account(AccountKeyring::Eve.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
        ));

        assert_eq!(
            Asset::balance_of((ticker, token.owner_did)),
            token.total_supply
        );

        assert_eq!(Asset::token_details(ticker), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            ticker,
            asset_rule
        ));

        // Mint some tokens to investor1
        assert_ok!(Asset::issue(
            owner_signed.clone(),
            owner_did,
            ticker,
            investor1_did,
            200_00_00 as u128,
            vec![0x0]
        ));

        assert_eq!(
            Asset::balance_of((ticker, investor1_did)),
            200_00_00 as u128
        );

        let msg = SignData {
            custodian_did: custodian_did,
            holder_did: investor1_did,
            ticker,
            value: 50_00_00 as u128,
            nonce: 1,
        };

        let investor1_key = AccountKeyring::Bob;

        // Add custodian
        assert_ok!(Asset::increase_custody_allowance_of(
            investor2_signed.clone(),
            ticker,
            investor1_did,
            AccountKeyring::Bob.public(),
            custodian_did,
            investor2_did,
            50_00_00 as u128,
            1,
            OffChainSignature::from(investor1_key.sign(&msg.encode()))
        ));

        assert_eq!(
            Asset::custodian_allowance((ticker, investor1_did, custodian_did)),
            50_00_00 as u128
        );

        assert_eq!(
            Asset::total_custody_allowance((ticker, investor1_did)),
            50_00_00 as u128
        );

        // use the same signature with the same nonce should fail
        assert_noop!(
            Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                AccountKeyring::Bob.public(),
                custodian_did,
                investor2_did,
                50_00_00 as u128,
                1,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ),
            "Signature already used"
        );

        // use the same signature with the different nonce should fail
        assert_noop!(
            Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                AccountKeyring::Bob.public(),
                custodian_did,
                investor2_did,
                50_00_00 as u128,
                3,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ),
            "Invalid signature"
        );

        // Transfer the token upto the limit
        assert_ok!(Asset::transfer(
            investor1_signed.clone(),
            investor1_did,
            ticker,
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(
            Asset::balance_of((ticker, investor2_did)),
            140_00_00 as u128
        );

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                ticker,
                investor2_did,
                50_00_00 as u128
            ),
            "Insufficient balance for transfer"
        );

        // Should fail to transfer the token by the custodian because of invalid signing key
        assert_noop!(
            Asset::transfer_by_custodian(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                45_00_00 as u128
            ),
            "sender must be a signing key for DID"
        );

        // Should fail to transfer the token by the custodian because of insufficient allowance
        assert_noop!(
            Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                custodian_did,
                investor2_did,
                55_00_00 as u128
            ),
            "Insufficient allowance"
        );

        // Successfully transfer by the custodian
        assert_ok!(Asset::transfer_by_custodian(
            custodian_signed.clone(),
            ticker,
            investor1_did,
            custodian_did,
            investor2_did,
            45_00_00 as u128
        ));
    });
}

#[test]
fn checkpoints_fuzz_test() {
    println!("Starting");
    for _ in 0..10 {
        // When fuzzing in local, feel free to bump this number to add more fuzz runs.
        build_ext().execute_with(|| {
            let now = Utc::now();
            Timestamp::set_timestamp(now.timestamp() as u64);

            let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            let (_, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
            ));

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                ticker,
                asset_rule
            ));

            let mut owner_balance: [u128; 100] = [1_000_000; 100];
            let mut bob_balance: [u128; 100] = [0; 100];
            let mut rng = rand::thread_rng();
            for j in 1..100 {
                let transfers = rng.gen_range(0, 10);
                owner_balance[j] = owner_balance[j - 1];
                bob_balance[j] = bob_balance[j - 1];
                for _k in 0..transfers {
                    if j == 1 {
                        owner_balance[0] -= 1;
                        bob_balance[0] += 1;
                    }
                    owner_balance[j] -= 1;
                    bob_balance[j] += 1;
                    assert_ok!(Asset::transfer(
                        owner_signed.clone(),
                        owner_did,
                        ticker,
                        bob_did,
                        1
                    ));
                }
                assert_ok!(Asset::create_checkpoint(
                    owner_signed.clone(),
                    owner_did,
                    ticker,
                ));
                let x: u64 = u64::try_from(j).unwrap();
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, 0),
                    owner_balance[j]
                );
                assert_eq!(Asset::get_balance_at(ticker, bob_did, 0), bob_balance[j]);
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, 1),
                    owner_balance[1]
                );
                assert_eq!(Asset::get_balance_at(ticker, bob_did, 1), bob_balance[1]);
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, x - 1),
                    owner_balance[j - 1]
                );
                assert_eq!(
                    Asset::get_balance_at(ticker, bob_did, x - 1),
                    bob_balance[j - 1]
                );
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, x),
                    owner_balance[j]
                );
                assert_eq!(Asset::get_balance_at(ticker, bob_did, x), bob_balance[j]);
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, x + 1),
                    owner_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(ticker, bob_did, x + 1),
                    bob_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(ticker, owner_did, 1000),
                    owner_balance[j]
                );
                assert_eq!(Asset::get_balance_at(ticker, bob_did, 1000), bob_balance[j]);
            }
        });
    }
}

#[test]
fn register_ticker() {
    build_ext().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let identifiers = vec![(IdentifierType::Custom(b"check".to_vec()), b"me".to_vec())];
        let ticker = Ticker::from_slice(token.name.as_slice());
        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
        ));

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
        let stored_token = Asset::token_details(&ticker);
        assert_eq!(stored_token.asset_type, token.asset_type);
        for (typ, val) in identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }

        assert_err!(
            Asset::register_ticker(owner_signed.clone(), Ticker::from_slice(&[0x01])),
            "token already created"
        );

        assert_err!(
            Asset::register_ticker(
                owner_signed.clone(),
                Ticker::from_slice(&[0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01])
            ),
            "ticker length over the limit"
        );

        let ticker = Ticker::from_slice(&[0x01, 0x01]);

        assert_eq!(Asset::is_ticker_available(&ticker), true);

        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

        let (alice_signed, _) = make_account(AccountKeyring::Alice.public()).unwrap();

        assert_err!(
            Asset::register_ticker(alice_signed.clone(), ticker),
            "ticker registered to someone else"
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        Timestamp::set_timestamp(now.timestamp() as u64 + 10001);

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), true);
    })
}

#[test]
fn transfer_ticker() {
    build_ext().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();
        let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

        let ticker = Ticker::from_slice(&[0x01, 0x01]);

        assert_eq!(Asset::is_ticker_available(&ticker), true);
        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(alice_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        let mut auth_id = Identity::last_authorization(Signer::from(alice_did));

        assert_err!(
            Asset::accept_ticker_transfer(alice_signed.clone(), auth_id + 1),
            "Authorization does not exist"
        );

        assert_ok!(Asset::accept_ticker_transfer(alice_signed.clone(), auth_id));

        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            "Illegal use of Authorization"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            Some(now.timestamp() as u64 - 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            AssetError::NoTickerTransferAuth
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_ok!(Asset::accept_ticker_transfer(bob_signed.clone(), auth_id));

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, bob_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
    })
}

#[test]
fn transfer_token_ownership() {
    build_ext().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();
        let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

        let token_name = vec![0x01, 0x01];
        let ticker = Ticker::from_slice(token_name.as_slice());
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token_name.clone(),
            ticker,
            1_000_000,
            true,
            AssetType::default(),
            vec![],
        ));

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(alice_did),
            AuthorizationData::TransferTokenOwnership(ticker),
            None,
        );

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(ticker),
            None,
        );

        assert_eq!(Asset::token_details(&ticker).owner_did, owner_did);

        let mut auth_id = Identity::last_authorization(Signer::from(alice_did));

        assert_err!(
            Asset::accept_token_ownership_transfer(alice_signed.clone(), auth_id + 1),
            "Authorization does not exist"
        );

        assert_ok!(Asset::accept_token_ownership_transfer(
            alice_signed.clone(),
            auth_id
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, alice_did);

        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
            "Illegal use of Authorization"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(ticker),
            Some(now.timestamp() as u64 - 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
            AssetError::NotTickerOwnershipTransferAuth
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(Ticker::from_slice(&[0x50])),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
            "Token does not exist"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(ticker),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_ok!(Asset::accept_token_ownership_transfer(
            bob_signed.clone(),
            auth_id
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, bob_did);
    })
}

#[test]
fn update_identifiers() {
    build_ext().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".to_vec(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
        };
        let ticker = Ticker::from_slice(token.name.as_slice());
        assert!(!<identity::DidRecords>::exists(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.to_vec())];
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
        ));
        // A correct entry was added
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(
            Asset::identifiers((ticker, IdentifierType::Cusip)),
            identifier_value1.to_vec()
        );
        let identifier_value2 = b"XYZ555";
        let updated_identifiers = vec![
            (IdentifierType::Cusip, Default::default()),
            (IdentifierType::Isin, identifier_value2.to_vec()),
        ];
        assert_ok!(Asset::update_identifiers(
            owner_signed.clone(),
            owner_did,
            ticker,
            updated_identifiers.clone(),
        ));
        for (typ, val) in updated_identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }
    });
}

/*
 *    #[test]
 *    /// This test loads up a YAML of testcases and checks each of them
 *    fn transfer_scenarios_external() {
 *        let mut yaml_path_buf = PathBuf::new();
 *        yaml_path_buf.push(env!("CARGO_MANIFEST_DIR")); // This package's root
 *        yaml_path_buf.push("tests/asset_transfers.yml");
 *
 *        println!("Loading YAML from {:?}", yaml_path_buf);
 *
 *        let yaml_string = read_to_string(yaml_path_buf.as_path())
 *            .expect("Could not load the YAML file to a string");
 *
 *        // Parse the YAML
 *        let yaml = YamlLoader::load_from_str(&yaml_string).expect("Could not parse the YAML file");
 *
 *        let yaml = &yaml[0];
 *
 *        let now = Utc::now();
 *
 *        for case in yaml["test_cases"]
 *            .as_vec()
 *            .expect("Could not reach test_cases")
 *        {
 *            println!("Case: {:#?}", case);
 *
 *            let accounts = case["named_accounts"]
 *                .as_hash()
 *                .expect("Could not view named_accounts as a hashmap");
 *
 *            let mut externalities = if let Some(identity_owner) =
 *                accounts.get(&Yaml::String("identity-owner".to_owned()))
 *            {
 *                identity_owned_by(
 *                    identity_owner["id"]
 *                        .as_i64()
 *                        .expect("Could not get identity owner's ID") as u64,
 *                )
 *            } else {
 *                frame_system::GenesisConfig::default()
 *                    .build_storage()
 *                    .unwrap()
 *                    .0
 *                    .into()
 *            };
 *
 *            with_externalities(&mut externalities, || {
 *                // Instantiate accounts
 *                for (name, account) in accounts {
 *                    Timestamp::set_timestamp(now.timestamp() as u64);
 *                    let name = name
 *                        .as_str()
 *                        .expect("Could not take named_accounts key as string");
 *                    let id = account["id"].as_i64().expect("id is not a number") as u64;
 *                    let balance = account["balance"]
 *                        .as_i64()
 *                        .expect("balance is not a number");
 *
 *                    println!("Preparing account {}", name);
 *
 *                    Balances::make_free_balance_be(&id, balance.clone() as u128);
 *                    println!("{}: gets {} initial balance", name, balance);
 *                    if account["issuer"]
 *                        .as_bool()
 *                        .expect("Could not check if account is an issuer")
 *                    {
 *                        assert_ok!(identity::Module::<Test>::do_create_issuer(id));
 *                        println!("{}: becomes issuer", name);
 *                    }
 *                    if account["investor"]
 *                        .as_bool()
*                        .expect("Could not check if account is an investor")
*                    {
 *                        assert_ok!(identity::Module::<Test>::do_create_investor(id));
 *                        println!("{}: becomes investor", name);
 *                    }
 *                }
 *
 *                // Issue tokens
 *                let tokens = case["tokens"]
*                    .as_hash()
 *                    .expect("Could not view tokens as a hashmap");
 *
 *                for (ticker, token) in tokens {
 *                    let ticker = ticker.as_str().expect("Can't parse ticker as string");
 *                    println!("Preparing token {}:", ticker);
 *
 *                    let owner = token["owner"]
 *                        .as_str()
 *                        .expect("Can't parse owner as string");
 *
 *                    let owner_id = accounts
 *                        .get(&Yaml::String(owner.to_owned()))
 *                        .expect("Can't get owner record")["id"]
 *                        .as_i64()
 *                        .expect("Can't parse owner id as i64")
 *                        as u64;
 *                    let total_supply = token["total_supply"]
 *                        .as_i64()
 *                        .expect("Can't parse the total supply as i64")
 *                        as u128;
 *
 *                    let token_struct = SecurityToken {
 *                        name: *ticker.into_bytes(),
 *                        owner: owner_id,
 *                        total_supply,
 *                        divisible: true,
 *                    };
 *                    println!("{:#?}", token_struct);
 *
 *                    // Check that issuing succeeds/fails as expected
 *                    if token["issuance_succeeds"]
 *                        .as_bool()
 *                        .expect("Could not check if issuance should succeed")
 *                    {
 *                        assert_ok!(Asset::create_token(
 *                            Origin::signed(token_struct.owner),
 *                            token_struct.name.clone(),
 *                            token_struct.name.clone(),
 *                            token_struct.total_supply,
 *                            true
 *                        ));
 *
 *                        // Also check that the new token matches what we asked to create
 *                        assert_eq!(
 *                            Asset::token_details(token_struct.name.clone()),
 *                            token_struct
 *                        );
 *
 *                        // Check that the issuer's balance corresponds to total supply
 *                        assert_eq!(
 *                            Asset::balance_of((token_struct.name, token_struct.owner)),
 *                            token_struct.total_supply
 *                        );
 *
 *                        // Add specified whitelist entries
 *                        let whitelists = token["whitelist_entries"]
 *                            .as_vec()
 *                            .expect("Could not view token whitelist entries as vec");
 *
 *                        for wl_entry in whitelists {
 *                            let investor = wl_entry["investor"]
 *                                .as_str()
 *                                .expect("Can't parse investor as string");
 *                            let investor_id = accounts
 *                                .get(&Yaml::String(investor.to_owned()))
 *                                .expect("Can't get investor account record")["id"]
 *                                .as_i64()
 *                                .expect("Can't parse investor id as i64")
 *                                as u64;
 *
 *                            let expiry = wl_entry["expiry"]
 *                                .as_i64()
 *                                .expect("Can't parse expiry as i64");
 *
 *                            let wl_id = wl_entry["whitelist_id"]
 *                                .as_i64()
 *                                .expect("Could not parse whitelist_id as i64")
 *                                as u32;
 *
 *                            println!(
 *                                "Token {}: processing whitelist entry for {}",
 *                                ticker, investor
 *                            );
 *
 *                            general_tm::Module::<Test>::add_to_whitelist(
 *                                Origin::signed(owner_id),
 *                                *ticker.into_bytes(),
 *                                wl_id,
 *                                investor_id,
 *                                (now + Duration::hours(expiry)).timestamp() as u64,
 *                            )
 *                            .expect("Could not create whitelist entry");
 *                        }
 *                    } else {
 *                        assert!(Asset::create_token(
 *                            Origin::signed(token_struct.owner),
 *                            token_struct.name.clone(),
 *                            token_struct.name.clone(),
 *                            token_struct.total_supply,
 *                            true
 *                        )
 *                        .is_err());
 *                    }
 *                }
 *
 *                // Set up allowances
 *                let allowances = case["allowances"]
*                    .as_vec()
 *                    .expect("Could not view allowances as a vec");
 *
 *                for allowance in allowances {
 *                    let sender = allowance["sender"]
 *                        .as_str()
 *                        .expect("Could not view sender as str");
 *                    let sender_id = case["named_accounts"][sender]["id"]
 *                        .as_i64()
 *                        .expect("Could not view sender id as i64")
 *                        as u64;
 *                    let spender = allowance["spender"]
 *                        .as_str()
 *                        .expect("Could not view spender as str");
 *                    let spender_id = case["named_accounts"][spender]["id"]
 *                        .as_i64()
 *                        .expect("Could not view sender id as i64")
 *                        as u64;
 *                    let amount = allowance["amount"]
 *                        .as_i64()
 *                        .expect("Could not view amount as i64")
 *                        as u128;
 *                    let ticker = allowance["ticker"]
 *                        .as_str()
 *                        .expect("Could not view ticker as str");
 *                    let succeeds = allowance["succeeds"]
 *                        .as_bool()
 *                        .expect("Could not determine if allowance should succeed");
 *
 *                    if succeeds {
 *                        assert_ok!(Asset::approve(
 *                            Origin::signed(sender_id),
 *                            *ticker.into_bytes(),
 *                            spender_id,
 *                            amount,
 *                        ));
 *                    } else {
 *                        assert!(Asset::approve(
 *                            Origin::signed(sender_id),
 *                            *ticker.into_bytes(),
 *                            spender_id,
 *                            amount,
 *                        )
 *                        .is_err())
 *                    }
 *                }
 *
 *                println!("Transfers:");
 *                // Perform regular transfers
 *                let transfers = case["transfers"]
*                    .as_vec()
 *                    .expect("Could not view transfers as vec");
 *                for transfer in transfers {
 *                    let from = transfer["from"]
 *                        .as_str()
 *                        .expect("Could not view from as str");
 *                    let from_id = case["named_accounts"][from]["id"]
 *                        .as_i64()
 *                        .expect("Could not view from_id as i64")
 *                        as u64;
 *                    let to = transfer["to"].as_str().expect("Could not view to as str");
 *                    let to_id = case["named_accounts"][to]["id"]
 *                        .as_i64()
 *                        .expect("Could not view to_id as i64")
 *                        as u64;
 *                    let amount = transfer["amount"]
 *                        .as_i64()
 *                        .expect("Could not view amount as i64")
 *                        as u128;
 *                    let ticker = transfer["ticker"]
 *                        .as_str()
 *                        .expect("Coule not view ticker as str")
 *                        .to_owned();
 *                    let succeeds = transfer["succeeds"]
 *                        .as_bool()
 *                        .expect("Could not view succeeds as bool");
 *
 *                    println!("{} of token {} from {} to {}", amount, ticker, from, to);
 *                    let ticker = ticker.into_bytes();
 *
 *                    // Get sender's investor data
 *                    let investor_data = <InvestorList<Test>>::get(from_id);
 *
 *                    println!("{}'s investor data: {:#?}", from, investor_data);
 *
 *                    if succeeds {
 *                        assert_ok!(Asset::transfer(
 *                            Origin::signed(from_id),
 *                            ticker,
 *                            to_id,
 *                            amount
 *                        ));
 *                    } else {
 *                        assert!(
 *                            Asset::transfer(Origin::signed(from_id), ticker, to_id, amount)
 *                                .is_err()
 *                        );
 *                    }
 *                }
 *
 *                println!("Approval-based transfers:");
 *                // Perform allowance transfers
 *                let transfer_froms = case["transfer_froms"]
*                    .as_vec()
 *                    .expect("Could not view transfer_froms as vec");
 *                for transfer_from in transfer_froms {
 *                    let from = transfer_from["from"]
 *                        .as_str()
 *                        .expect("Could not view from as str");
 *                    let from_id = case["named_accounts"][from]["id"]
 *                        .as_i64()
 *                        .expect("Could not view from_id as i64")
 *                        as u64;
 *                    let spender = transfer_from["spender"]
 *                        .as_str()
 *                        .expect("Could not view spender as str");
 *                    let spender_id = case["named_accounts"][spender]["id"]
 *                        .as_i64()
 *                        .expect("Could not view spender_id as i64")
 *                        as u64;
 *                    let to = transfer_from["to"]
 *                        .as_str()
 *                        .expect("Could not view to as str");
 *                    let to_id = case["named_accounts"][to]["id"]
 *                        .as_i64()
 *                        .expect("Could not view to_id as i64")
 *                        as u64;
 *                    let amount = transfer_from["amount"]
 *                        .as_i64()
 *                        .expect("Could not view amount as i64")
 *                        as u128;
 *                    let ticker = transfer_from["ticker"]
 *                        .as_str()
 *                        .expect("Coule not view ticker as str")
 *                        .to_owned();
 *                    let succeeds = transfer_from["succeeds"]
 *                        .as_bool()
 *                        .expect("Could not view succeeds as bool");
 *
 *                    println!(
 *                        "{} of token {} from {} to {} spent by {}",
 *                        amount, ticker, from, to, spender
 *                    );
 *                    let ticker = ticker.into_bytes();
 *
 *                    // Get sender's investor data
 *                    let investor_data = <InvestorList<Test>>::get(spender_id);
 *
 *                    println!("{}'s investor data: {:#?}", from, investor_data);
 *
 *                    if succeeds {
 *                        assert_ok!(Asset::transfer_from(
 *                            Origin::signed(spender_id),
 *                            ticker,
 *                            from_id,
 *                            to_id,
 *                            amount
 *                        ));
 *                    } else {
 *                        assert!(Asset::transfer_from(
 *                            Origin::signed(from_id),
 *                            ticker,
 *                            from_id,
 *                            to_id,
 *                            amount
 *                        )
 *                        .is_err());
 *                    }
 *                }
 *            });
 *        }
 *    }
 */
