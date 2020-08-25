use crate::{
    storage::{add_secondary_key, register_keyring_account, account_from, make_account_without_cdd, AccountId, TestStorage},
    ExtBuilder,
};

use pallet_asset::{
    self as asset, AssetOwnershipRelation, AssetType, FundingRoundName, IdentifierType,
    SecurityToken, SignData,
};
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use pallet_statistics as statistics;
use polymesh_common_utilities::{constants::*, traits::balances::Memo, compliance_manager::Trait};
use polymesh_primitives::{
    AuthorizationData, Document, DocumentName, IdentityId, Signatory, SmartExtension,
    SmartExtensionType, Ticker, SmartExtensionName, Claim, Rule, RuleType
};

use chrono::prelude::Utc;
use codec::Encode;
use frame_support::{
    assert_err, assert_noop, assert_ok, traits::Currency, StorageDoubleMap, StorageMap, dispatch::DispatchResult
};
use hex_literal::hex;
use ink_primitives::hash as FunctionSelectorHasher;
use rand::Rng;
use sp_runtime::AnySignature;
use std::convert::{TryFrom, TryInto};
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type DidRecords = identity::DidRecords<TestStorage>;
type Statistics = statistics::Module<TestStorage>;

macro_rules! assert_add_claim {
    ($signer:expr, $target:expr, $claim:expr) => {
        assert_ok!(
            Identity::add_claim(
                $signer,
                $target,
                $claim,
                None,
            )
        );
    };
}

macro_rules! assert_revoke_claim {
    ($signer:expr, $target:expr, $claim:expr) => {
        assert_ok!(
            Identity::revoke_claim(
                $signer,
                $target,
                $claim,
            )
        );
    };
}

fn smart_extension_addition(account_id: AccountId, extension_name: SmartExtensionName, signer: Origin, ticker: Ticker) -> DispatchResult{
    let extension_details = SmartExtension {
        extension_type: SmartExtensionType::TransferManager,
        extension_name: extension_name,
        extension_id: account_id,
        is_archive: false,
    };

    Asset::add_extension(
        signer,
        ticker,
        extension_details.clone(),
    )
}

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
        assert!(!<DidRecords>::contains_key(
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
            Some(funding_round_name.clone()),
        ));

        // Check the update investor count for the newly created asset
        assert_eq!(Statistics::investor_count_per_asset(ticker), 1);

        // A correct entry is added
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );
        assert!(<DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));
        assert_eq!(Asset::funding_round(ticker), funding_round_name.clone());

        // Unauthorized identities cannot rename the token.
        let eve_signed = Origin::signed(AccountKeyring::Eve.public());
        let _eve_did = register_keyring_account(AccountKeyring::Eve).unwrap();
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
    });
}

/// # TODO
/// It should be re-enable once issuer claim is re-enabled.
#[test]
#[ignore]
fn non_issuers_cant_create_tokens() {
    ExtBuilder::default().build().execute_with(|| {
        let _ = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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

fn valid_custodian_allowance() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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

        let investor1_signed = Origin::signed(AccountKeyring::Bob.public());
        let investor1_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let investor2_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let custodian_signed = Origin::signed(AccountKeyring::Eve.public());
        let custodian_did = register_keyring_account(AccountKeyring::Eve).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None,
        ));

        assert_eq!(
            Asset::balance_of(&ticker, &token.owner_did),
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

        assert_ok!(Asset::issue(owner_signed.clone(), ticker, num_tokens1));
        assert_ok!(Asset::unsafe_transfer(
            owner_did,
            &ticker,
            owner_did,
            investor1_did,
            num_tokens1
        ));

        assert_eq!(Asset::funding_round(&ticker), funding_round1.clone());
        assert_eq!(
            Asset::issued_in_funding_round((ticker, funding_round1.clone())),
            num_tokens1
        );
        // Check the expected default behaviour of the map.
        let no_such_round: FundingRoundName = b"No such round".into();
        assert_eq!(Asset::issued_in_funding_round((ticker, no_such_round)), 0);
        assert_eq!(Asset::balance_of(&ticker, &investor1_did), num_tokens1);

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

        // Successfully transfer by the custodian
        assert_ok!(Asset::transfer_by_custodian(
            custodian_signed.clone(),
            ticker,
            investor1_did,
            investor2_did,
            45_00_00 as u128
        ));

        assert_eq!(
            Asset::custodian_allowance((ticker, investor1_did, custodian_did)),
            50_000 as u128
        );

        assert_eq!(
            Asset::total_custody_allowance((ticker, investor1_did)),
            50_000 as u128
        );
    });
}

#[test]
fn valid_custodian_allowance_of() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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

        let investor1_signed = Origin::signed(AccountKeyring::Bob.public());
        let investor1_did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let investor2_signed = Origin::signed(AccountKeyring::Charlie.public());
        let investor2_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let custodian_signed = Origin::signed(AccountKeyring::Eve.public());
        let custodian_did = register_keyring_account(AccountKeyring::Eve).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None,
        ));

        assert_eq!(
            Asset::balance(&ticker, token.owner_did).total,
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
            200_00_00 as u128,
        ));
        assert_ok!(Asset::unsafe_transfer(
            owner_did,
            &ticker,
            owner_did,
            investor1_did,
            2_000_000 as u128
        ));

        assert_eq!(
            Asset::balance(&ticker, investor1_did).total,
            2_000_000 as u128
        );

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

            let owner_signed = Origin::signed(AccountKeyring::Dave.public());
            let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

            // Issuance is successful
            assert_ok!(Asset::create_asset(
                owner_signed.clone(),
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                token.asset_type.clone(),
                vec![],
                None,
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
                    assert_ok!(Asset::unsafe_transfer(
                        owner_did, &ticker, owner_did, bob_did, 1
                    ));
                }
                assert_ok!(Asset::create_checkpoint(owner_signed.clone(), ticker));
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

        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
            None,
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

        assert_eq!(
            Asset::asset_ownership_relation(owner_did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        let alice_signed = Origin::signed(AccountKeyring::Alice.public());
        let _ = register_keyring_account(AccountKeyring::Alice).unwrap();

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

        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let alice_signed = Origin::signed(AccountKeyring::Alice.public());
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_signed = Origin::signed(AccountKeyring::Bob.public());
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

        let ticker = Ticker::try_from(&[0x01, 0x01][..]).unwrap();

        assert_eq!(Asset::is_ticker_available(&ticker), true);
        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker));

        let auth_id_alice = Identity::add_auth(
            owner_did,
            Signatory::from(alice_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            owner_did,
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

        assert_eq!(
            Asset::asset_ownership_relation(owner_did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_ok!(Asset::accept_ticker_transfer(
            alice_signed.clone(),
            auth_id_alice
        ));

        assert_eq!(
            Asset::asset_ownership_relation(owner_did, ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Asset::asset_ownership_relation(alice_did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_eq!(
            Asset::asset_ownership_relation(alice_did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id_bob),
            "Illegal use of Authorization"
        );

        let mut auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            Some(now.timestamp() as u64 - 100),
        );

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            AssetError::NoTickerTransferAuth
        );

        auth_id = Identity::add_auth(
            alice_did,
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
fn transfer_primary_issuance_agent() {
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let owner_signed = Origin::signed(AccountKeyring::Alice.public());
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let primary_issuance_signed = Origin::signed(AccountKeyring::Bob.public());
        let primary_issuance_agent = register_keyring_account(AccountKeyring::Bob).unwrap();

        let ticker = Ticker::try_from(&[0x01, 0x01][..]).unwrap();
        let token = SecurityToken {
            name: ticker.as_slice().into(),
            total_supply: 1_000_000,
            owner_did,
            divisible: true,
            asset_type: Default::default(),
            primary_issuance_agent: Some(owner_did),
        };

        assert_ok!(Asset::create_asset(
            owner_signed,
            token.name.clone(),
            ticker.clone(),
            token.total_supply,
            token.divisible,
            token.asset_type.clone(),
            Default::default(),
            Default::default(),
        ));

        assert!(!Asset::is_ticker_available(&ticker));
        assert_eq!(Asset::token_details(&ticker), token);

        let auth_id = Identity::add_auth(
            owner_did,
            Signatory::from(primary_issuance_agent),
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
            Some(now.timestamp() as u64 - 100),
        );

        assert_err!(
            Asset::accept_primary_issuance_agent_transfer(primary_issuance_signed.clone(), auth_id),
            "Authorization expired"
        );
        assert_eq!(Asset::token_details(&ticker), token);

        let auth_id = Identity::add_auth(
            owner_did,
            Signatory::from(owner_did),
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
            None,
        );

        assert_err!(
            Asset::accept_primary_issuance_agent_transfer(primary_issuance_signed.clone(), auth_id),
            "Authorization does not exist"
        );
        assert_eq!(Asset::token_details(&ticker), token);

        let auth_id = Identity::add_auth(
            primary_issuance_agent,
            Signatory::from(primary_issuance_agent),
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
            None,
        );

        assert_err!(
            Asset::accept_primary_issuance_agent_transfer(primary_issuance_signed.clone(), auth_id),
            "Illegal use of Authorization"
        );
        assert_eq!(Asset::token_details(&ticker), token);

        let auth_id = Identity::add_auth(
            owner_did,
            Signatory::from(primary_issuance_agent),
            AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
            None,
        );

        assert_ok!(Asset::accept_primary_issuance_agent_transfer(
            primary_issuance_signed.clone(),
            auth_id
        ));
        let mut new_token = token.clone();
        new_token.primary_issuance_agent = Some(primary_issuance_agent);
        assert_eq!(Asset::token_details(&ticker), new_token);
    })
}

#[test]
fn transfer_token_ownership() {
    ExtBuilder::default().build().execute_with(|| {
        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();
        let alice_signed = Origin::signed(AccountKeyring::Alice.public());
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_signed = Origin::signed(AccountKeyring::Bob.public());
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();

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
            None,
        ));

        let auth_id_alice = Identity::add_auth(
            owner_did,
            Signatory::from(alice_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            owner_did,
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        assert_eq!(Asset::token_details(&ticker).owner_did, owner_did);

        assert_err!(
            Asset::accept_asset_ownership_transfer(alice_signed.clone(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        assert_eq!(
            Asset::asset_ownership_relation(owner_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            alice_signed.clone(),
            auth_id_alice
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, alice_did);
        assert_eq!(
            Asset::asset_ownership_relation(owner_did, ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Asset::asset_ownership_relation(alice_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id_bob),
            "Illegal use of Authorization"
        );

        let mut auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            Some(now.timestamp() as u64 - 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            "Authorization expired"
        );

        auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::Custom(ticker),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            AssetError::NotTickerOwnershipTransferAuth
        );

        auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(Ticker::try_from(&[0x50][..]).unwrap()),
            Some(now.timestamp() as u64 + 100),
        );

        assert_err!(
            Asset::accept_asset_ownership_transfer(bob_signed.clone(), auth_id),
            AssetError::NoSuchAsset
        );

        auth_id = Identity::add_auth(
            alice_did,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: Some(owner_did),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<DidRecords>::contains_key(
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
            None,
        ));

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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        assert!(!<DidRecords>::contains_key(
            Identity::get_token_did(&ticker).unwrap()
        ));

        let identifiers = vec![(IdentifierType::default(), b"undefined".into())];
        let _ticker_did = Identity::get_token_did(&ticker).unwrap();

        // Issuance is successful
        assert_ok!(Asset::create_asset(
            owner_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            identifiers.clone(),
            None,
        ));

        let documents = vec![
            (
                b"A".into(),
                Document {
                    uri: b"www.a.com".into(),
                    content_hash: b"0x1".into(),
                },
            ),
            (
                b"B".into(),
                Document {
                    uri: b"www.b.com".into(),
                    content_hash: b"0x2".into(),
                },
            ),
        ];

        assert_ok!(Asset::batch_add_document(
            owner_signed.clone(),
            documents.clone(),
            ticker
        ));

        assert_eq!(
            Asset::asset_documents(ticker, DocumentName::from(b"A")),
            documents[0].1
        );
        assert_eq!(
            Asset::asset_documents(ticker, DocumentName::from(b"B")),
            documents[1].1
        );

        assert_ok!(Asset::batch_remove_document(
            owner_signed.clone(),
            vec![b"A".into(), b"B".into()],
            ticker
        ));

        assert_eq!(
            <asset::AssetDocuments>::iter_prefix_values(ticker).count(),
            0
        );
    });
}

#[test]
fn add_extension_successfully() {
    ExtBuilder::default()
    .set_max_tms_allowed(2)
    .build()
    .execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let _ = register_keyring_account(AccountKeyring::Dave).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: b"TEST".into(),
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };

        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        assert!(!<DidRecords>::contains_key(
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
            None,
        ));

        // Add smart extension
        let extension_name_1: SmartExtensionName = b"PTM1".into();
        let extension_name_2 = b"PTM2".into();
        let extension_name_3 = b"PTM3".into();
        let extension_id_1 = account_from(1);
        let extension_id_2 = account_from(2);
        let extension_id_3 = account_from(3);

        assert_ok!(smart_extension_addition(extension_id_1, extension_name_1.clone(), owner_signed.clone(), ticker));
        assert_ok!(smart_extension_addition(extension_id_2, extension_name_2, owner_signed.clone(), ticker));

        // verify the data within the runtime
        assert_eq!(
            Asset::extension_details((ticker, extension_id_1)).extension_name,
            extension_name_1
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager))).len(),
            2
        );
        assert_eq!(
            (Asset::extensions((ticker, SmartExtensionType::TransferManager)))[0],
            extension_id_1
        );

        assert_err!(smart_extension_addition(extension_id_3, extension_name_3, owner_signed.clone(), ticker), AssetError::MaximumTMExtensionLimitReached);
    });
}

#[test]
fn add_same_extension_should_fail() {
    ExtBuilder::default()
    .set_max_tms_allowed(10)
    .build()
    .execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let owner_signed = Origin::signed(AccountKeyring::Dave.public());
        let owner_did = register_keyring_account(AccountKeyring::Dave).unwrap();

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
        assert!(!<DidRecords>::contains_key(
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
            None,
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
        let alice_signed = Origin::signed(AccountKeyring::Alice.public());
        let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let bob_signed = Origin::signed(AccountKeyring::Bob.public());
        let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
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
            None,
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

        // Attempt to transfer token ownership.
        let auth_id = Identity::add_auth(
            alice_did,
            Signatory::from(bob_did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob_signed.clone(),
            auth_id
        ));

        assert_ok!(Asset::unfreeze(bob_signed.clone(), ticker));
        assert_err!(
            Asset::unfreeze(bob_signed.clone(), ticker),
            AssetError::NotFrozen
        );
    });
}
#[test]
fn frozen_secondary_keys_create_asset() {
    ExtBuilder::default()
        .build()
        .execute_with(frozen_secondary_keys_create_asset_we);
}

fn frozen_secondary_keys_create_asset_we() {
    // 0. Create identities.
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let _charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let bob = AccountKeyring::Bob.public();

    // 1. Add Bob as signatory to Alice ID.
    let bob_signatory = Signatory::Account(AccountKeyring::Bob.public());
    add_secondary_key(alice_id, bob_signatory);
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
        primary_issuance_agent: Some(alice_id),
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

    // 3. Alice freezes her secondary keys.
    assert_ok!(Identity::freeze_secondary_keys(Origin::signed(alice)));

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
        .balance_factor(1)
        .build()
        .execute_with(|| {
            let alice_signed = Origin::signed(AccountKeyring::Alice.public());
            let alice_did = register_keyring_account(AccountKeyring::Alice).unwrap();
            let _bob_signed = Origin::signed(AccountKeyring::Bob.public());
            let bob_did = register_keyring_account(AccountKeyring::Bob).unwrap();
            let _custodian_signed = Origin::signed(AccountKeyring::Charlie.public());
            let custodian_did = register_keyring_account(AccountKeyring::Charlie).unwrap();

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
                None,
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

#[test]
fn can_set_primary_issuance_agent() {
    ExtBuilder::default()
        .build()
        .execute_with(can_set_primary_issuance_agent_we);
}

fn can_set_primary_issuance_agent_we() {
    let alice = AccountKeyring::Alice.public();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob = AccountKeyring::Bob.public();
    let bob_id = register_keyring_account(AccountKeyring::Bob).unwrap();
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice),
        bob,
        1_000,
        Some(Memo::from("Bob funding"))
    ));
    let mut token = SecurityToken {
        name: vec![0x01].into(),
        owner_did: alice_id,
        total_supply: 1_000_000,
        divisible: true,
        asset_type: AssetType::default(),
        primary_issuance_agent: Some(bob_id),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

    assert_ok!(Asset::create_asset(
        Origin::signed(alice),
        token.name.clone(),
        ticker,
        token.total_supply,
        true,
        token.asset_type.clone(),
        vec![],
        None,
    ));
    let auth_id = Identity::add_auth(
        token.owner_did,
        Signatory::from(bob_id),
        AuthorizationData::TransferPrimaryIssuanceAgent(ticker),
        None,
    );
    assert_ok!(Asset::accept_primary_issuance_agent_transfer(
        Origin::signed(bob),
        auth_id
    ));
    assert_eq!(Asset::token_details(ticker), token);
    assert_ok!(Asset::remove_primary_issuance_agent(
        Origin::signed(alice),
        ticker
    ));
    token.primary_issuance_agent = None;
    assert_eq!(Asset::token_details(ticker), token);
}

#[test]
fn test_weights_for_is_valid_transfer() {

    ExtBuilder::default()
    .set_max_tms_allowed(4)
    .build()
    .execute_with(|| {
        let alice = AccountKeyring::Alice.public();
        let (alice_signed, alice_did) = make_account_without_cdd(alice).unwrap();

        let bob = AccountKeyring::Bob.public();
        let (bob_signed, bob_did) = make_account_without_cdd(bob).unwrap();

        let charlie = AccountKeyring::Charlie.public();
        let (charlie_signed, charlie_did) = make_account_without_cdd(charlie).unwrap();
        
        let eve = AccountKeyring::Eve.public();
        let (eve_signed, eve_did) = make_account_without_cdd(eve).unwrap();

        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: alice_did,
            total_supply: 1_000_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            primary_issuance_agent: Some(alice_did),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();

        // Get token Id.
        let ticker_id = Identity::get_token_did(&ticker).unwrap();
    
        assert_ok!(Asset::create_asset(
            alice_signed.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None,
        ));

        // Adding different claim rules 
        assert_ok!(ComplianceManager::add_active_rule(
            alice_signed.clone(),
            ticker,
            vec![
                Rule {
                    rule_type: RuleType::IsPresent(Claim::Accredited(ticker_id)),
                    issuers: vec![eve_did]
                },
                Rule {
                    rule_type: RuleType::IsAbsent(Claim::BuyLockup(ticker_id)),
                    issuers: vec![eve_did]
                }
            ],
            vec![
                Rule {
                    rule_type: RuleType::IsPresent(Claim::Accredited(ticker_id)),
                    issuers: vec![eve_did]
                },
                Rule {
                    rule_type: RuleType::IsAnyOf(vec![Claim::BuyLockup(ticker_id), Claim::KnowYourCustomer(ticker_id)]),
                    issuers: vec![eve_did]
                }
            ]
        ));

        // Providing claim to sender and receiver
        // For Alice
        assert_add_claim!(eve_signed.clone(), alice_did, Claim::Accredited(alice_did));
        assert_add_claim!(eve_signed.clone(), alice_did, Claim::BuyLockup(ticker_id));
        // For Bob
        assert_add_claim!(eve_signed.clone(), bob_did, Claim::Accredited(ticker_id));
        assert_add_claim!(eve_signed.clone(), bob_did, Claim::KnowYourCustomer(ticker_id));

        // Add Tms
        assert_ok!(
            Asset::add_extension(
                alice_signed.clone(),
                ticker,
                SmartExtension {
                    extension_type: SmartExtensionType::TransferManager,
                    extension_name: b"ABC".into(),
                    extension_id: account_from(1),
                    is_archive: false
                }
            )
        );

        assert_ok!(
            Asset::add_extension(
                alice_signed.clone(),
                ticker,
                SmartExtension {
                    extension_type: SmartExtensionType::TransferManager,
                    extension_name: b"ABC".into(),
                    extension_id: account_from(2),
                    is_archive: false
                }
            )
        );

        // call is_valid_transfer()
        let result = Asset::_is_valid_transfer(&ticker, alice, Some(alice_did), Some(bob_did), 100).unwrap().1;
        let weight_from_verify_transfer = ComplianceManager::verify_restriction(&ticker, Some(alice_did), Some(bob_did), 100, Some(alice_did)).unwrap().1;
        assert!(matches!(result, weight_from_verify_transfer)); // Only sender rules are processed.

        assert_revoke_claim!(eve_signed.clone(), alice_did, Claim::BuyLockup(ticker_id));
        assert_add_claim!(eve_signed.clone(), alice_did, Claim::Accredited(ticker_id));

        let result = Asset::_is_valid_transfer(&ticker, alice, Some(alice_did), Some(bob_did), 100).unwrap().1;
        let weight_from_verify_transfer = ComplianceManager::verify_restriction(&ticker, Some(alice_did), Some(bob_did), 100, Some(alice_did)).unwrap().1;
        let computed_weight = Asset::compute_transfer_result(false, 2, weight_from_verify_transfer).1;
        assert!(matches!(result, computed_weight)); // Sender & receiver rules are processed.

        // Adding different claim rules 
        assert_ok!(ComplianceManager::add_active_rule(
            alice_signed.clone(),
            ticker,
            vec![
                Rule {
                    rule_type: RuleType::IsPresent(Claim::Exempted(ticker_id)),
                    issuers: vec![eve_did]
                }
            ],
            vec![
                Rule {
                    rule_type: RuleType::IsPresent(Claim::Blocked(ticker_id)),
                    issuers: vec![eve_did]
                }
            ]
        ));

        let result = Asset::_is_valid_transfer(&ticker, alice, Some(alice_did), Some(bob_did), 100).unwrap();
        let verify_restriction_result = ComplianceManager::verify_restriction(&ticker, Some(alice_did), Some(bob_did), 100, Some(alice_did)).unwrap();
        let weight_from_verify_transfer = verify_restriction_result.1;
        assert_eq!(verify_restriction_result.0, 81);
        let computed_weight = Asset::compute_transfer_result(false, 2, weight_from_verify_transfer).1;
        let transfer_weight = result.1;
        assert!(matches!(transfer_weight, computed_weight)); // Sender & receiver rules are processed.

        // pause transfer rules
        assert_ok!(ComplianceManager::pause_asset_rules(alice_signed, ticker));

        let result = Asset::_is_valid_transfer(&ticker, alice, Some(alice_did), Some(bob_did), 100).unwrap();
        let weight_from_verify_transfer = ComplianceManager::verify_restriction(&ticker, Some(alice_did), Some(bob_did), 100, Some(alice_did)).unwrap().1;
        let computed_weight = Asset::compute_transfer_result(false, 2, weight_from_verify_transfer).1;
        assert!(matches!(transfer_weight, computed_weight));
    });
}