use super::{
    storage::{add_signing_item, make_account, register_keyring_account, TestStorage},
    ExtBuilder,
};

use pallet_asset::{
    self as asset, AssetType, FundingRoundName, IdentifierType, SecurityToken, SignData,
};
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use polymesh_common_utilities::{
    constants::*, traits::asset::IssueAssetItem, traits::balances::Memo,
};
use polymesh_primitives::{
    AccountKey, AuthorizationData, Document, IdentityId, LinkData, Signatory, SmartExtension,
    SmartExtensionType, Ticker,
};

use chrono::prelude::Utc;
use codec::Encode;
use frame_support::{
    assert_err, assert_noop, assert_ok, traits::Currency, StorageDoubleMap, StorageMap,
};
use hex_literal::hex;
use ink_primitives::hash as FunctionSelectorHasher;
use rand::Rng;
use sp_runtime::AnySignature;
use std::{
    convert::{TryFrom, TryInto},
    mem,
};
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn check_the_test_hex() {
    ExtBuilder::default().build().execute_with(|| {
        let selector: [u8; 4] = (FunctionSelectorHasher::keccak256("verify_transfer".as_bytes())
            [0..4])
            .try_into()
            .unwrap();
        println!("{:#X}", u32::from_be_bytes(selector));
        let data = hex!("D9386E41");
        println!("{:?}", data);
    });
}

#[test]
fn issuers_can_create_and_rename_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();
        let funding_round_name: FundingRoundName = b"round1".into();
        // Expected token entry
        let mut token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifiers = vec![(IdentifierType::default(), b"undefined".into())];
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert_err!(
            Asset::create_asset(
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

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            Some(funding_round_name.clone())
        ));

        let token_link = Identity::get_link(
            Signatory::from(owner_did),
            Asset::token_details(ticker).link_id,
        );
        assert_eq!(token_link.link_data, LinkData::AssetOwned(ticker));
        assert_eq!(token_link.expiry, None);

        let ticker_link = Identity::get_link(
            Signatory::from(owner_did),
            Asset::ticker_registration(ticker).link_id,
        );

        assert_eq!(ticker_link.link_data, LinkData::TickerOwned(ticker));
        assert_eq!(ticker_link.expiry, None);

        token.link_id = Asset::token_details(ticker).link_id;
        // A correct entry is added
        assert_eq!(Asset::token_details(ticker), token);
        assert!(<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        assert_eq!(Asset::funding_round(ticker), funding_round_name.clone());

        // Unauthorized identities cannot rename the token.
        let (eve_signed, _eve_did) = make_account(AccountKeyring::Eve.public()).unwrap();
        assert_err!(
            Asset::rename_asset(eve_signed, ticker, vec![0xde, 0xad, 0xbe, 0xef].into()),
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
            link_id: Asset::token_details(ticker).link_id,
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
    });
}

/// # TODO
/// It should be re-enable once issuer claim is re-enabled.
#[test]
#[ignore]
fn non_issuers_cant_create_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let (_, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let _ = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        Balances::make_free_balance_be(&AccountKeyring::Bob.public(), 1_000_000);

        let wrong_did = IdentityId::try_from("did:poly:wrong");
        assert!(wrong_did.is_err());
    });
}

#[test]
fn valid_transfers_pass() {
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        let (_, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));

        // Allow all transfers
        assert_ok!(ComplianceManager::add_active_rule(
            owner_signed.clone(),
            ticker,
            vec![],
            vec![]
        ));

        assert_ok!(Asset::transfer(
            owner_signed.clone(),
            ticker,
            alice_did,
            500
        ));

        let mut cap_table = <asset::BalanceOf<TestStorage>>::iter_prefix(ticker);
        let balance_alice = cap_table.next().unwrap();
        let balance_owner = cap_table.next().unwrap();
        assert_eq!(balance_owner, 1_000_000 - 500);
        assert_eq!(balance_alice, 500);
    })
}

#[test]
fn valid_custodian_allowance() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        let (investor1_signed, investor1_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let (_, investor2_did) = make_account(AccountKeyring::Charlie.public()).unwrap();
        let (custodian_signed, custodian_did) = make_account(AccountKeyring::Eve.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));

        assert_eq!(
            Asset::balance(&ticker, &token.owner_did),
            token.total_supply
        );

        // Allow all transfers
        assert_ok!(ComplianceManager::add_active_rule(
            owner_signed.clone(),
            ticker,
            vec![],
            vec![]
        ));
        let funding_round1: FundingRoundName = b"Round One".into();
        assert_ok!(Asset::set_funding_round(
            owner_signed.clone(),
            ticker,
            funding_round1.clone()
        ));
        // Mint some tokens to investor1
        let num_tokens1: u128 = 2_000_000;
        assert_ok!(Asset::issue(
            owner_signed.clone(),
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
        let no_such_round: FundingRoundName = b"No such round".into();
        assert_eq!(Asset::issued_in_funding_round((ticker, no_such_round)), 0);
        assert_eq!(Asset::balance(&ticker, &investor1_did), num_tokens1);

        // Failed to add custodian because of insufficient balance
        assert_noop!(
            Asset::increase_custody_allowance(
                investor1_signed.clone(),
                ticker,
                custodian_did,
                250_00_00 as u128
            ),
            AssetError::InsufficientBalance
        );

        // Failed to add/increase the custodian allowance because of Invalid custodian did
        let custodian_did_not_register = IdentityId::from(5u128);
        assert_noop!(
            Asset::increase_custody_allowance(
                investor1_signed.clone(),
                ticker,
                custodian_did_not_register,
                50_00_00 as u128
            ),
            AssetError::InvalidCustodianDid
        );

        // Add custodian
        assert_ok!(Asset::increase_custody_allowance(
            investor1_signed.clone(),
            ticker,
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
            ticker,
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(Asset::balance(&ticker, &investor2_did), 140_00_00 as u128);

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                ticker,
                investor2_did,
                50_00_00 as u128
            ),
            AssetError::InsufficientBalance
        );

        // Should fail to transfer the token by the custodian because of insufficient allowance
        assert_noop!(
            Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                investor2_did,
                55_00_00 as u128
            ),
            AssetError::InsufficientAllowance
        );

        // Successfully transfer by the custodian
        assert_ok!(Asset::transfer_by_custodian(
            custodian_signed.clone(),
            ticker,
            investor1_did,
            investor2_did,
            45_00_00 as u128
        ));
    });
}

#[test]
fn valid_custodian_allowance_of() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        let (investor1_signed, investor1_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let (investor2_signed, investor2_did) =
            make_account(AccountKeyring::Charlie.public()).unwrap();
        let (custodian_signed, custodian_did) = make_account(AccountKeyring::Eve.public()).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None
        ));

        assert_eq!(
            Asset::balance(&ticker, &token.owner_did),
            token.total_supply
        );

        // Allow all transfers
        assert_ok!(ComplianceManager::add_active_rule(
            owner_signed.clone(),
            ticker,
            vec![],
            vec![]
        ));

        // Mint some tokens to investor1
        assert_ok!(Asset::issue(
            owner_signed.clone(),
            ticker,
            investor1_did,
            200_00_00 as u128,
            vec![0x0]
        ));

        assert_eq!(Asset::balance(&ticker, &investor1_did), 200_00_00 as u128);

        let msg = SignData {
            custodian_did,
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
                50_00_00 as u128,
                1,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ),
            AssetError::SignatureAlreadyUsed
        );

        // use the same signature with the different nonce should fail
        assert_noop!(
            Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                ticker,
                investor1_did,
                AccountKeyring::Bob.public(),
                custodian_did,
                50_00_00 as u128,
                3,
                OffChainSignature::from(investor1_key.sign(&msg.encode()))
            ),
            AssetError::InvalidSignature
        );

        // Transfer the token upto the limit
        assert_ok!(Asset::transfer(
            investor1_signed.clone(),
            ticker,
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(Asset::balance(&ticker, &investor2_did), 140_00_00 as u128);

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                ticker,
                investor2_did,
                50_00_00 as u128
            ),
            AssetError::InsufficientBalance
        );

        // Should fail to transfer the token by the custodian because of insufficient allowance
        assert_noop!(
            Asset::transfer_by_custodian(
                custodian_signed.clone(),
                ticker,
                investor1_did,
                investor2_did,
                55_00_00 as u128
            ),
            AssetError::InsufficientAllowance
        );

        // Successfully transfer by the custodian
        assert_ok!(Asset::transfer_by_custodian(
            custodian_signed.clone(),
            ticker,
            investor1_did,
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
        ExtBuilder::default().build().execute_with(|| {
            let now = Utc::now();
            Timestamp::set_timestamp(now.timestamp() as u64);

            let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01].into(),
                owner_did,
                total_supply: 1_000_000,
                divisible: true,
                asset_type: AssetType::default(),
                ..Default::default()
            };
            let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
            let (_, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

            // Issuance is successful
            assert_ok!(Asset::create_asset(
                owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None
            ));

            // Allow all transfers
            assert_ok!(ComplianceManager::add_active_rule(
                owner_signed.clone(),
                ticker,
                vec![],
                vec![]
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
                    assert_ok!(Asset::transfer(owner_signed.clone(), ticker, bob_did, 1));
                }
                assert_ok!(Asset::create_checkpoint(owner_signed.clone(), ticker,));
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
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let identifiers = vec![(IdentifierType::Isin, b"0123".into())];
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
        let stored_token = Asset::token_details(&ticker);
        assert_eq!(stored_token.asset_type, token.asset_type);
        for (typ, val) in identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }

        assert_err!(
            Asset::register_ticker(owner_signed.clone(), Ticker::try_from(&[0x01][..]).unwrap()),
            AssetError::AssetAlreadyCreated
        );

        assert_err!(
            Asset::register_ticker(
                owner_signed.clone(),
                Ticker::try_from(&[0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01][..])
                    .unwrap()
            ),
            AssetError::TickerTooLong
        );

        let ticker = Ticker::try_from(&[0x01, 0x01][..]).unwrap();

        assert_eq!(Asset::is_ticker_available(&ticker), true);

        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

        let ticker_link = Identity::get_link(
            Signatory::from(owner_did),
            Asset::ticker_registration(ticker).link_id,
        );

        assert_eq!(ticker_link.link_data, LinkData::TickerOwned(ticker));

        let (alice_signed, _) = make_account(AccountKeyring::Alice.public()).unwrap();

        assert_err!(
            Asset::register_ticker(alice_signed.clone(), ticker),
            AssetError::TickerAlreadyRegistered
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
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();
        let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

        let ticker = Ticker::try_from(&[0x01, 0x01][..]).unwrap();

        assert_eq!(Asset::is_ticker_available(&ticker), true);
        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

        let auth_id_alice = Identity::add_auth(
            Signatory::from(owner_did),
            Signatory::from(alice_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            Signatory::from(owner_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        assert_err!(
            Asset::accept_ticker_transfer(alice_signed.clone(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        let old_ticker = Asset::ticker_registration(ticker);
        let old_ticker_link =
            Identity::get_link(Signatory::from(old_ticker.owner), old_ticker.link_id);
        assert_eq!(old_ticker_link.link_data, LinkData::TickerOwned(ticker));

        assert_ok!(Asset::accept_ticker_transfer(
            alice_signed.clone(),
            auth_id_alice
        ));

        assert!(!<identity::Links<TestStorage>>::contains_key(
            Signatory::from(old_ticker.owner),
            old_ticker.link_id
        ));

        let ticker_link = Identity::get_link(
            Signatory::from(alice_did),
            Asset::ticker_registration(ticker).link_id,
        );
        assert_eq!(ticker_link.link_data, LinkData::TickerOwned(ticker));

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id_bob),
            "Illegal use of Authorization"
        );

        let mut auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            Some(now.timestamp() as u64 - 100),
        );

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            AssetError::NoTickerTransferAuth
        );

        auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_ok!(Asset::accept_ticker_transfer(bob_signed.clone(), auth_id));

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice_did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, bob_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
    })
}

#[test]
fn transfer_token_ownership() {
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();
        let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();

        let token_name = vec![0x01, 0x01];
        let ticker = Ticker::try_from(token_name.as_slice()).unwrap();
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token_name.into(),
            ticker,
            1_000_000,
            true,
            AssetType::default(),
            vec![],
            None
        ));

        let auth_id_alice = Identity::add_auth(
            Signatory::from(owner_did),
            Signatory::from(alice_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            Signatory::from(owner_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        assert_eq!(Asset::token_details(&ticker).owner_did, owner_did);

        assert_err!(
            Asset::accept_asset_ownership_transfer(alice_signed.clone(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        let old_ticker = Asset::ticker_registration(ticker);
        let old_ticker_link =
            Identity::get_link(Signatory::from(old_ticker.owner), old_ticker.link_id);

        assert_eq!(old_ticker_link.link_data, LinkData::TickerOwned(ticker));

        let old_token = Asset::token_details(ticker);
        let old_token_link =
            Identity::get_link(Signatory::from(old_token.owner_did), old_token.link_id);

        assert_eq!(old_token_link.link_data, LinkData::AssetOwned(ticker));

        assert_ok!(Asset::accept_asset_ownership_transfer(
            alice_signed.clone(),
            auth_id_alice
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, alice_did);
        assert!(!<identity::Links<TestStorage>>::contains_key(
            Signatory::from(old_ticker.owner),
            old_ticker.link_id
        ));
        assert!(!<identity::Links<TestStorage>>::contains_key(
            Signatory::from(old_token.owner_did),
            old_token.link_id
        ));

        let ticker_link = Identity::get_link(
            Signatory::from(alice_did),
            Asset::ticker_registration(ticker).link_id,
        );
        assert_eq!(ticker_link.link_data, LinkData::TickerOwned(ticker));
        let token_link = Identity::get_link(
            Signatory::from(alice_did),
            Asset::token_details(ticker).link_id,
        );
        assert_eq!(token_link.link_data, LinkData::AssetOwned(ticker));

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id_bob),
            "Illegal use of Authorization"
        );

        let mut auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            Some(now.timestamp() as u64 - 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            AssetError::NotTickerOwnershipTransferAuth
        );

        auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(Ticker::try_from(&[0x50][..]).unwrap()),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            AssetError::NoSuchAsset
        );

        auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob_signed.clone(),
            auth_id
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, bob_did);
    })
}

#[test]
fn update_identifiers() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let mut token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));

        token.link_id = Asset::token_details(ticker).link_id;
        // A correct entry was added
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(
            Asset::identifiers((ticker, IdentifierType::Cusip)),
            identifier_value1.into()
        );
        let identifier_value2 = b"XYZ555";
        let updated_identifiers = vec![
            (IdentifierType::Cusip, Default::default()),
            (IdentifierType::Isin, identifier_value2.into()),
        ];
        assert_ok!(Asset::update_identifiers(
            owner_signed.clone(),
            ticker,
            updated_identifiers.clone(),
        ));
        for (typ, val) in updated_identifiers {
            assert_eq!(Asset::identifiers((ticker, typ)), val);
        }
    });
}

#[test]
fn adding_removing_documents() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));

        let identifiers = vec![(IdentifierType::default(), b"undefined".into())];
        let ticker_did = Identity::get_token_did(&ticker).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));

        let documents = vec![
            Document {
                name: b"A".into(),
                uri: b"www.a.com".into(),
                content_hash: b"0x1".into(),
            },
            Document {
                name: b"B".into(),
                uri: b"www.b.com".into(),
                content_hash: b"0x2".into(),
            },
        ];

        assert_ok!(Asset::batch_add_document(
            owner_signed.clone(),
            documents,
            ticker
        ));

        let mut docs = <identity::Links<TestStorage>>::iter_prefix(Signatory::from(ticker_did));
        let mut doc1 = docs.next().unwrap();
        let mut doc2 = docs.next().unwrap();
        if doc1.link_id > doc2.link_id {
            mem::swap(&mut doc1, &mut doc2);
        }

        assert_eq!(
            doc1.link_data,
            LinkData::DocumentOwned(Document {
                name: b"A".into(),
                uri: b"www.a.com".into(),
                content_hash: b"0x1".into(),
            })
        );
        assert_eq!(doc1.expiry, None);

        assert_eq!(
            doc2.link_data,
            LinkData::DocumentOwned(Document {
                name: b"B".into(),
                uri: b"www.b.com".into(),
                content_hash: b"0x2".into()
            })
        );

        assert_eq!(doc2.expiry, None);

        assert_ok!(Asset::batch_update_document(
            owner_signed.clone(),
            vec![
                (
                    doc1.link_id,
                    Document {
                        name: b"C".into(),
                        uri: b"www.c.com".into(),
                        content_hash: b"0x3".into(),
                    }
                ),
                (
                    doc2.link_id,
                    Document {
                        name: b"D".into(),
                        uri: b"www.d.com".into(),
                        content_hash: b"0x4".into(),
                    }
                ),
            ],
            ticker
        ));

        docs = <identity::Links<TestStorage>>::iter_prefix(Signatory::from(ticker_did));
        doc1 = docs.next().unwrap();
        doc2 = docs.next().unwrap();
        if doc1.link_id > doc2.link_id {
            mem::swap(&mut doc1, &mut doc2);
        }

        assert_eq!(
            doc1.link_data,
            LinkData::DocumentOwned(Document {
                name: b"C".into(),
                uri: b"www.c.com".into(),
                content_hash: b"0x3".into(),
            })
        );

        assert_eq!(doc1.expiry, None);

        assert_eq!(
            doc2.link_data,
            LinkData::DocumentOwned(Document {
                name: b"D".into(),
                uri: b"www.d.com".into(),
                content_hash: b"0x4".into(),
            })
        );
        assert_eq!(doc2.expiry, None);
    });
}

#[test]
fn add_extension_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, _) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));

        // Add smart extension
        let extension_name = b"PTM".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::TransferManager,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone(),
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager)))[0],
            extension_id
        );
    });
}

#[test]
fn add_same_extension_should_fail() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));

        // Add smart extension
        let extension_name = b"PTM".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::TransferManager,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone()
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details.clone()
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager)))[0],
            extension_id
        );

        assert_err!(
            Asset::add_extension(owner_signed.clone(), ticker, extension_details),
            AssetError::ExtensionAlreadyPresent
        );
    });
}

#[test]
fn should_successfully_archive_extension() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));
        // Add smart extension
        let extension_name = b"STO".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::Offerings,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone()
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings)))[0],
            extension_id
        );

        assert_ok!(Asset::archive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));

        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            true
        );
    });
}

#[test]
fn should_fail_to_archive_an_already_archived_extension() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));
        // Add smart extension
        let extension_name = b"STO".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::Offerings,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone()
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings)))[0],
            extension_id
        );

        assert_ok!(Asset::archive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));

        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            true
        );

        assert_err!(
            Asset::archive_extension(owner_signed.clone(), ticker, extension_id),
            AssetError::AlreadyArchived
        );
    });
}

#[test]
fn should_fail_to_archive_a_non_existent_extension() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));
        // Add smart extension
        let extension_id = AccountKeyring::Bob.public();

        assert_err!(
            Asset::archive_extension(owner_signed.clone(), ticker, extension_id),
            AssetError::NoSuchSmartExtension
        );
    });
}

#[test]
fn should_successfuly_unarchive_an_extension() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));
        // Add smart extension
        let extension_name = b"STO".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::Offerings,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone()
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings)))[0],
            extension_id
        );

        assert_ok!(Asset::archive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));

        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            true
        );

        assert_ok!(Asset::unarchive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));
        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            false
        );
    });
}

#[test]
fn should_fail_to_unarchive_an_already_unarchived_extension() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Dave.public()).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<identity::DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        let identifier_value1 = b"ABC123";
        let identifiers = vec![(IdentifierType::Cusip, identifier_value1.into())];
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None
        ));
        // Add smart extension
        let extension_name = b"STO".into();
        let extension_id = AccountKeyring::Bob.public();

        let extension_details = SmartExtension {
            extension_type: SmartExtensionType::Offerings,
            extension_name,
            extension_id: extension_id.clone(),
            is_archive: false,
        };

        assert_ok!(Asset::add_extension(
            owner_signed.clone(),
            ticker,
            extension_details.clone(),
        ));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id)),
            extension_details
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings))).len(),
            1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::Offerings)))[0],
            extension_id
        );

        assert_ok!(Asset::archive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));

        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            true
        );

        assert_ok!(Asset::unarchive_extension(
            owner_signed.clone(),
            ticker,
            extension_id
        ));
        assert_eq!(
            (Asset::extension_details((ticker, extension_id))).is_archive,
            false
        );

        assert_err!(
            Asset::unarchive_extension(owner_signed.clone(), ticker, extension_id),
            AssetError::AlreadyUnArchived
        );
    });
}

#[test]
fn freeze_unfreeze_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);
        let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let token_name = b"COOL";
        let ticker = Ticker::try_from(&token_name[..]).unwrap();
        assert_ok!(Asset::create_asset(
            alice_signed.clone(),
            token_name.into(),
            ticker,
            1_000_000,
            true,
            AssetType::default(),
            vec![],
            None
        ));

        // Allow all transfers.
        assert_ok!(ComplianceManager::add_active_rule(
            alice_signed.clone(),
            ticker,
            vec![],
            vec![]
        ));
        assert_err!(
            Asset::freeze(bob_signed.clone(), ticker),
            AssetError::Unauthorized
        );
        assert_err!(
            Asset::unfreeze(alice_signed.clone(), ticker),
            AssetError::NotFrozen
        );
        assert_ok!(Asset::freeze(alice_signed.clone(), ticker));
        assert_err!(
            Asset::freeze(alice_signed.clone(), ticker),
            AssetError::AlreadyFrozen
        );

        // Attempt to mint tokens.
        assert_err!(
            Asset::issue(alice_signed.clone(), ticker, bob_did, 1, vec![]),
            AssetError::InvalidTransfer
        );
        assert_err!(
            Asset::transfer(alice_signed.clone(), ticker, bob_did, 1),
            AssetError::InvalidTransfer
        );
        // Attempt to transfer token ownership.
        let auth_id = Identity::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob_signed.clone(),
            auth_id
        ));

        // `batch_issue` fails when the vector of recipients is not empty.
        assert_err!(
            Asset::batch_issue(
                bob_signed.clone(),
                vec![IssueAssetItem {
                    investor_did: bob_did,
                    value: 1
                }],
                ticker
            ),
            AssetError::InvalidTransfer
        );
        // `batch_issue` fails with the empty vector of investors with a different error message.
        assert_err!(
            Asset::batch_issue(bob_signed.clone(), vec![], ticker),
            AssetError::NoInvestors
        );
        assert_ok!(Asset::unfreeze(bob_signed.clone(), ticker));
        assert_err!(
            Asset::unfreeze(bob_signed.clone(), ticker),
            AssetError::NotFrozen
        );
        // Transfer some balance.
        assert_ok!(Asset::transfer(alice_signed.clone(), ticker, bob_did, 1));
    });
}
#[test]
fn frozen_signing_keys_create_asset() {
    ExtBuilder::default()
        .build()
        .execute_with(frozen_signing_keys_create_asset_we);
}

fn frozen_signing_keys_create_asset_we() {
    // 0. Create identities.
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let _charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let bob = AccountKeyring::Bob.public();

    // 1. Add Bob as signatory to Alice ID.
    assert_ok!(Balances::top_up_identity_balance(
        Origin::signed(alice),
        alice_id,
        100_000
    ));
    let bob_signatory = Signatory::from(AccountKey::from(AccountKeyring::Bob.public().0));
    add_signing_item(alice_id, bob_signatory);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        1_000,
        Some(Memo::from("Bob funding"))
    ));

    // 2. Bob can create token
    let token_1 = SecurityToken {
        name: vec![0x01].into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        link_id: 18,
        ..Default::default()
    };
    let ticker_1 = Ticker::try_from(token_1.name.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        Origin::signed(bob),
        token_1.name.clone(),
        ticker_1,
        token_1.total_supply,
        true,
        token_1.asset_type.clone(),
        vec![],
        None,
    ));
    assert_eq!(Asset::token_details(ticker_1), token_1);

    // 3. Alice freezes her signing keys.
    assert_ok!(Identity::freeze_signing_keys(Origin::signed(alice)));

    // 4. Bob cannot create a token.
    let token_2 = SecurityToken {
        name: vec![0x01].into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let _ticker_2 = Ticker::try_from(token_2.name.as_slice()).unwrap();
    // commenting this because `default_identity` feature is not allowing to access None identity.
    // let create_token_result = Asset::create_asset(
    //     Origin::signed(bob),
    //     token_2.name.clone(),
    //     ticker_2,
    //     token_2.total_supply,
    //     true,
    //     token_2.asset_type.clone(),
    //     vec![],
    //     None,
    // );
    // assert_err!(
    //     create_token_result,
    //     DispatchError::Other("Current identity is none and key is not linked to any identity")
    // );
}

#[test]
fn test_can_transfer_rpc() {
    ExtBuilder::default()
        .cdd_providers(vec![AccountKeyring::Eve.public()])
        .monied(true)
        .existential_deposit(1)
        .build()
        .execute_with(|| {
            let (alice_signed, alice_did) = make_account(AccountKeyring::Alice.public()).unwrap();
            let (_bob_signed, bob_did) = make_account(AccountKeyring::Bob.public()).unwrap();
            let (_custodian_signed, custodian_did) =
                make_account(AccountKeyring::Charlie.public()).unwrap();
            let token_name = b"COOL";
            let ticker = Ticker::try_from(&token_name[..]).unwrap();
            assert_ok!(Asset::create_asset(
                alice_signed.clone(),
                token_name.into(),
                ticker,
                1_000 * currency::ONE_UNIT,
                false, // Asset divisibility is false
                AssetType::default(),
                vec![],
                None
            ));

            // check the balance of the alice Identity
            assert_eq!(
                Asset::balance_of(&ticker, alice_did),
                1_000 * currency::ONE_UNIT
            );

            // case 1: When passed invalid granularity
            assert_eq!(
                Asset::unsafe_can_transfer(
                    AccountKeyring::Alice.public(),
                    ticker,
                    Some(alice_did),
                    Some(bob_did),
                    100 // It only passed when it should be the multiple of currency::ONE_UNIT
                )
                .unwrap(),
                INVALID_GRANULARITY
            );

            // Case 2: when from_did balance is 0
            assert_eq!(
                Asset::unsafe_can_transfer(
                    AccountKeyring::Bob.public(),
                    ticker,
                    Some(bob_did),
                    Some(alice_did),
                    100 * currency::ONE_UNIT
                )
                .unwrap(),
                ERC1400_INSUFFICIENT_BALANCE
            );

            // Case 3: When custody allowance is provided and amount of transfer is more than free balance
            // 3.1: Add custody provider
            assert_ok!(Asset::increase_custody_allowance(
                alice_signed.clone(),
                ticker,
                custodian_did,
                900 * currency::ONE_UNIT
            ));

            // 3.2: Execute can_transfer
            assert_eq!(
                Asset::unsafe_can_transfer(
                    AccountKeyring::Alice.public(),
                    ticker,
                    Some(alice_did),
                    Some(bob_did),
                    901 * currency::ONE_UNIT
                )
                .unwrap(),
                ERC1400_INSUFFICIENT_BALANCE
            );

            // Comment below test case, These will be un-commented when we improved the DID check.

            // // Case 4: When sender doesn't posses a valid cdd
            // // 4.1: Create Identity who doesn't posses cdd
            // let (from_without_cdd_signed, from_without_cdd_did) = make_account(AccountKeyring::Ferdie.public()).unwrap();
            // // Execute can_transfer
            // assert_eq!(
            //     Asset::unsafe_can_transfer(
            //         AccountKeyring::Ferdie.public(),
            //         ticker,
            //         Some(from_without_cdd_did),
            //         Some(bob_did),
            //         5 * currency::ONE_UNIT
            //     ).unwrap(),
            //     INVALID_SENDER_DID
            // );

            // // Case 5: When receiver doesn't posses a valid cdd
            // assert_eq!(
            //     Asset::unsafe_can_transfer(
            //         AccountKeyring::Alice.public(),
            //         ticker,
            //         Some(alice_did),
            //         Some(from_without_cdd_did),
            //         5 * currency::ONE_UNIT
            //     ).unwrap(),
            //     INVALID_RECEIVER_DID
            // );

            // Case 6: When Asset transfer is frozen
            // 6.1: pause the transfer
            assert_ok!(Asset::freeze(alice_signed.clone(), ticker));
            assert_eq!(
                Asset::unsafe_can_transfer(
                    AccountKeyring::Alice.public(),
                    ticker,
                    Some(alice_did),
                    Some(bob_did),
                    20 * currency::ONE_UNIT
                )
                .unwrap(),
                ERC1400_TRANSFERS_HALTED
            );
            assert_ok!(Asset::unfreeze(alice_signed.clone(), ticker));

            // Case 7: when transaction get success by the compliance_manager
            // Allow all transfers.
            assert_ok!(ComplianceManager::add_active_rule(
                alice_signed.clone(),
                ticker,
                vec![],
                vec![]
            ));

            assert_eq!(
                Asset::unsafe_can_transfer(
                    AccountKeyring::Alice.public(),
                    ticker,
                    Some(alice_did),
                    Some(bob_did),
                    20 * currency::ONE_UNIT
                )
                .unwrap(),
                ERC1400_TRANSFER_SUCCESS
            );
        })
}
