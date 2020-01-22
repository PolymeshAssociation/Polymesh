use crate::{
    asset,
    test::storage::{build_ext, register_keyring_account, TestStorage},
};

/// Build a genesis identity instance owned by account No. 1
fn identity_owned_by_alice() -> sr_io::TestExternalities<Blake2Hasher> {
    let mut t = system::GenesisConfig::default()
        .build_storage::<TestStorage>()
        .unwrap();
    identity::GenesisConfig::<TestStorage> {
        owner: AccountKeyring::Alice.public().into(),
        did_creation_fee: 250,
    }
    .assimilate_storage(&mut t)
    .unwrap();
    self::GenesisConfig::<TestStorage> {
        asset_creation_fee: 0,
        ticker_registration_fee: 0,
        ticker_registration_config: TickerRegistrationConfig {
            max_ticker_length: 12,
            registration_length: Some(10000),
        },
        fee_collector: AccountKeyring::Dave.public().into(),
    }
    .assimilate_storage(&mut t)
    .unwrap();
    sr_io::TestExternalities::new(t)
}

#[test]
fn issuers_can_create_and_rename_tokens() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();
        // Raise the owner's base currency balance
        Balances::make_free_balance_be(&owner_acc, 1_000_000);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };
        assert!(!<identity::DidRecords>::exists(
            Identity::get_token_did(&token.name).unwrap()
        ));
        let ticker_name = token.name.clone();
        assert_err!(
            Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                ticker_name.clone(),
                1_000_000_000_000_000_000_000_000, // Total supply over the limit
                true
            ),
            "Total supply above the limit"
        );

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            ticker_name.clone(),
            token.total_supply,
            true
        ));

        // A correct entry is added
        assert_eq!(Asset::token_details(token.name.clone()), token);
        //assert!(Identity::is_existing_identity(Identity::get_token_did(&token.name).unwrap()));
        assert!(<identity::DidRecords>::exists(
            Identity::get_token_did(&token.name).unwrap()
        ));
        assert_eq!(Asset::token_details(ticker_name.clone()), token);

        // Unauthorized identities cannot rename the token.
        let eve_acc = AccountId::from(AccountKeyring::Eve);
        let (eve_signed, _eve_did) = make_account(&eve_acc).unwrap();
        assert_err!(
            Asset::rename_token(
                eve_signed,
                ticker_name.clone(),
                vec![0xde, 0xad, 0xbe, 0xef]
            ),
            "sender must be a signing key for the token owner DID"
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker_name.clone()), token);
        // Rename the token and check storage has been updated.
        let renamed_token = SecurityToken {
            name: vec![0x42],
            owner_did: token.owner_did,
            total_supply: token.total_supply,
            divisible: token.divisible,
        };
        assert_ok!(Asset::rename_token(
            owner_signed.clone(),
            ticker_name.clone(),
            renamed_token.name.clone()
        ));
        assert_eq!(Asset::token_details(ticker_name.clone()), renamed_token);
    });
}

/// # TODO
/// It should be re-enable once issuer claim is re-enabled.
#[test]
#[ignore]
fn non_issuers_cant_create_tokens() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (_, owner_did) = make_account(&owner_acc).unwrap();

        // Expected token entry
        let _ = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        let wrong_acc = AccountId::from(AccountKeyring::Bob);

        Balances::make_free_balance_be(&wrong_acc, 1_000_000);

        let wrong_did = IdentityId::try_from("did:poly:wrong");
        assert!(wrong_did.is_err());
    });
}

#[test]
fn valid_transfers_pass() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        Balances::make_free_balance_be(&owner_acc, 1_000_000);

        let alice_acc = AccountId::from(AccountKeyring::Alice);
        let (_, alice_did) = make_account(&alice_acc).unwrap();

        Balances::make_free_balance_be(&alice_acc, 1_000_000);

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            token.name.clone(),
            token.total_supply,
            true
        ));

        // A correct entry is added
        assert_eq!(Asset::token_details(token.name.clone()), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            asset_rule
        ));

        assert_ok!(Asset::transfer(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            alice_did,
            500
        ));
    })
}

#[test]
fn valid_custodian_allowance() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        Balances::make_free_balance_be(&owner_acc, 1_000_000);

        let investor1_acc = AccountId::from(AccountKeyring::Bob);
        let (investor1_signed, investor1_did) = make_account(&investor1_acc).unwrap();

        Balances::make_free_balance_be(&investor1_acc, 1_000_000);

        let investor2_acc = AccountId::from(AccountKeyring::Charlie);
        let (investor2_signed, investor2_did) = make_account(&investor2_acc).unwrap();

        Balances::make_free_balance_be(&investor2_acc, 1_000_000);

        let custodian_acc = AccountId::from(AccountKeyring::Eve);
        let (custodian_signed, custodian_did) = make_account(&custodian_acc).unwrap();

        Balances::make_free_balance_be(&custodian_acc, 1_000_000);

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            token.name.clone(),
            token.total_supply,
            true
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), token.owner_did)),
            token.total_supply
        );

        assert_eq!(Asset::token_details(token.name.clone()), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            asset_rule
        ));

        // Mint some tokens to investor1
        assert_ok!(Asset::issue(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            investor1_did,
            200_00_00 as u128,
            vec![0x0]
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), investor1_did)),
            200_00_00 as u128
        );

        // Failed to add custodian because of insufficient balance
        assert_noop!(
            Asset::increase_custody_allowance(
                investor1_signed.clone(),
                token.name.clone(),
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
                token.name.clone(),
                investor1_did,
                custodian_did_not_register,
                50_00_00 as u128
            ),
            "Invalid custodian DID"
        );

        // Add custodian
        assert_ok!(Asset::increase_custody_allowance(
            investor1_signed.clone(),
            token.name.clone(),
            investor1_did,
            custodian_did,
            50_00_00 as u128
        ));

        assert_eq!(
            Asset::custodian_allowance((token.name.clone(), investor1_did, custodian_did)),
            50_00_00 as u128
        );

        assert_eq!(
            Asset::total_custody_allowance((token.name.clone(), investor1_did)),
            50_00_00 as u128
        );

        // Transfer the token upto the limit
        assert_ok!(Asset::transfer(
            investor1_signed.clone(),
            investor1_did,
            token.name.clone(),
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), investor2_did)),
            140_00_00 as u128
        );

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                token.name.clone(),
                investor2_did,
                50_00_00 as u128
            ),
            "Insufficient balance for transfer"
        );

        // Should fail to transfer the token by the custodian because of invalid signing key
        assert_noop!(
            Asset::transfer_by_custodian(
                investor2_signed.clone(),
                token.name.clone(),
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
                token.name.clone(),
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
            token.name.clone(),
            investor1_did,
            custodian_did,
            investor2_did,
            45_00_00 as u128
        ));
    });
}

#[test]
fn valid_custodian_allowance_of() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        // Expected token entry
        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        Balances::make_free_balance_be(&owner_acc, 1_000_000);

        let investor1_acc = AccountId::from(AccountKeyring::Bob);
        let (investor1_signed, investor1_did) = make_account(&investor1_acc).unwrap();

        Balances::make_free_balance_be(&investor1_acc, 1_000_000);

        let investor2_acc = AccountId::from(AccountKeyring::Charlie);
        let (investor2_signed, investor2_did) = make_account(&investor2_acc).unwrap();

        Balances::make_free_balance_be(&investor2_acc, 1_000_000);

        let custodian_acc = AccountId::from(AccountKeyring::Eve);
        let (custodian_signed, custodian_did) = make_account(&custodian_acc).unwrap();

        Balances::make_free_balance_be(&custodian_acc, 1_000_000);

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            token.name.clone(),
            token.total_supply,
            true
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), token.owner_did)),
            token.total_supply
        );

        assert_eq!(Asset::token_details(token.name.clone()), token);

        let asset_rule = general_tm::AssetRule {
            sender_rules: vec![],
            receiver_rules: vec![],
        };

        // Allow all transfers
        assert_ok!(GeneralTM::add_active_rule(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            asset_rule
        ));

        // Mint some tokens to investor1
        assert_ok!(Asset::issue(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            investor1_did,
            200_00_00 as u128,
            vec![0x0]
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), investor1_did)),
            200_00_00 as u128
        );

        let msg = SignData {
            custodian_did: custodian_did,
            holder_did: investor1_did,
            ticker: token.name.clone(),
            value: 50_00_00 as u128,
            nonce: 1,
        };

        let investor1_key = AccountKeyring::Bob;

        // Add custodian
        assert_ok!(Asset::increase_custody_allowance_of(
            investor2_signed.clone(),
            token.name.clone(),
            investor1_did,
            investor1_acc.clone(),
            custodian_did,
            investor2_did,
            50_00_00 as u128,
            1,
            OffChainSignature::from(investor1_key.sign(&msg.encode()))
        ));

        assert_eq!(
            Asset::custodian_allowance((token.name.clone(), investor1_did, custodian_did)),
            50_00_00 as u128
        );

        assert_eq!(
            Asset::total_custody_allowance((token.name.clone(), investor1_did)),
            50_00_00 as u128
        );

        // use the same signature with the same nonce should fail
        assert_noop!(
            Asset::increase_custody_allowance_of(
                investor2_signed.clone(),
                token.name.clone(),
                investor1_did,
                investor1_acc.clone(),
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
                token.name.clone(),
                investor1_did,
                investor1_acc.clone(),
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
            token.name.clone(),
            investor2_did,
            140_00_00 as u128
        ));

        assert_eq!(
            Asset::balance_of((token.name.clone(), investor2_did)),
            140_00_00 as u128
        );

        // Try to Transfer the tokens beyond the limit
        assert_noop!(
            Asset::transfer(
                investor1_signed.clone(),
                investor1_did,
                token.name.clone(),
                investor2_did,
                50_00_00 as u128
            ),
            "Insufficient balance for transfer"
        );

        // Should fail to transfer the token by the custodian because of invalid signing key
        assert_noop!(
            Asset::transfer_by_custodian(
                investor2_signed.clone(),
                token.name.clone(),
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
                token.name.clone(),
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
            token.name.clone(),
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
    for _i in 0..10 {
        // When fuzzing in local, feel free to bump this number to add more fuzz runs.
        with_externalities(&mut identity_owned_by_alice(), || {
            let now = Utc::now();
            <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

            let owner_acc = AccountId::from(AccountKeyring::Dave);
            let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

            // Expected token entry
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: owner_did,
                total_supply: 1_000_000,
                divisible: true,
            };

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (_, bob_did) = make_account(&bob_acc).unwrap();

            // Issuance is successful
            assert_ok!(Asset::create_token(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                owner_signed.clone(),
                owner_did,
                token.name.clone(),
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
                        token.name.clone(),
                        bob_did,
                        1
                    ));
                }
                assert_ok!(Asset::create_checkpoint(
                    owner_signed.clone(),
                    owner_did,
                    token.name.clone(),
                ));
                let x: u64 = u64::try_from(j).unwrap();
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, 0),
                    owner_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, 0),
                    bob_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, 1),
                    owner_balance[1]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, 1),
                    bob_balance[1]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, x - 1),
                    owner_balance[j - 1]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, x - 1),
                    bob_balance[j - 1]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, x),
                    owner_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, x),
                    bob_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, x + 1),
                    owner_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, x + 1),
                    bob_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, owner_did, 1000),
                    owner_balance[j]
                );
                assert_eq!(
                    Asset::get_balance_at(&token.name, bob_did, 1000),
                    bob_balance[j]
                );
            }
        });
    }
}

#[test]
fn register_ticker() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        Balances::make_free_balance_be(&owner_acc, 1_000_000);

        let token = SecurityToken {
            name: vec![0x01],
            owner_did: owner_did,
            total_supply: 1_000_000,
            divisible: true,
        };

        // Issuance is successful
        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            token.name.clone(),
            token.name.clone(),
            token.total_supply,
            true
        ));

        assert_eq!(
            Asset::is_ticker_registry_valid(&token.name, owner_did),
            true
        );
        assert_eq!(Asset::is_ticker_available(&token.name), false);

        assert_err!(
            Asset::register_ticker(owner_signed.clone(), vec![0x01]),
            "token already created"
        );

        assert_err!(
            Asset::register_ticker(
                owner_signed.clone(),
                vec![0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01, 0x01]
            ),
            "ticker length over the limit"
        );

        let ticker = vec![0x01, 0x01];

        assert_eq!(Asset::is_ticker_available(&ticker), true);

        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker.clone()));

        let alice_acc = AccountId::from(AccountKeyring::Alice);
        let (alice_signed, _) = make_account(&alice_acc).unwrap();

        Balances::make_free_balance_be(&alice_acc, 1_000_000);

        assert_err!(
            Asset::register_ticker(alice_signed.clone(), ticker.clone()),
            "ticker registered to someone else"
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64 + 10001);

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner_did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), true);
    })
}

#[test]
fn transfer_ticker() {
    with_externalities(&mut identity_owned_by_alice(), || {
        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        let alice_acc = AccountId::from(AccountKeyring::Alice);
        let (alice_signed, alice_did) = make_account(&alice_acc).unwrap();

        let bob_acc = AccountId::from(AccountKeyring::Bob);
        let (bob_signed, bob_did) = make_account(&bob_acc).unwrap();

        let ticker = vec![0x01, 0x01];

        assert_eq!(Asset::is_ticker_available(&ticker), true);
        assert_ok!(Asset::register_ticker(owner_signed.clone(), ticker.clone()));

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(alice_did),
            AuthorizationData::TransferTicker(ticker.clone()),
            None,
        );

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTicker(ticker.clone()),
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
            AuthorizationData::TransferTicker(ticker.clone()),
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
            AuthorizationData::Custom(ticker.clone()),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_ticker_transfer(bob_signed.clone(), auth_id),
            "Not a ticker transfer auth"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTicker(ticker.clone()),
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
    with_externalities(&mut identity_owned_by_alice(), || {
        let now = Utc::now();
        <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);

        let owner_acc = AccountId::from(AccountKeyring::Dave);
        let (owner_signed, owner_did) = make_account(&owner_acc).unwrap();

        let alice_acc = AccountId::from(AccountKeyring::Alice);
        let (alice_signed, alice_did) = make_account(&alice_acc).unwrap();

        let bob_acc = AccountId::from(AccountKeyring::Bob);
        let (bob_signed, bob_did) = make_account(&bob_acc).unwrap();

        let ticker = vec![0x01, 0x01];

        assert_ok!(Asset::create_token(
            owner_signed.clone(),
            owner_did,
            ticker.clone(),
            ticker.clone(),
            1_000_000,
            true
        ));

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(alice_did),
            AuthorizationData::TransferTokenOwnership(ticker.clone()),
            None,
        );

        Identity::add_auth(
            Signer::from(owner_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(ticker.clone()),
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
            AuthorizationData::TransferTokenOwnership(ticker.clone()),
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
            AuthorizationData::Custom(ticker.clone()),
            Some(now.timestamp() as u64 + 100),
        );
        auth_id = Identity::last_authorization(Signer::from(bob_did));
        assert_err!(
            Asset::accept_token_ownership_transfer(bob_signed.clone(), auth_id),
            "Not a token ownership transfer auth"
        );

        Identity::add_auth(
            Signer::from(alice_did),
            Signer::from(bob_did),
            AuthorizationData::TransferTokenOwnership(vec![0x50]),
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
            AuthorizationData::TransferTokenOwnership(ticker.clone()),
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
 *                system::GenesisConfig::default()
 *                    .build_storage()
 *                    .unwrap()
 *                    .0
 *                    .into()
 *            };
 *
 *            with_externalities(&mut externalities, || {
 *                // Instantiate accounts
 *                for (name, account) in accounts {
 *                    <timestamp::Module<Test>>::set_timestamp(now.timestamp() as u64);
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
 *                        name: ticker.to_owned().into_bytes(),
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
 *                                ticker.to_owned().into_bytes(),
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
 *                            ticker.to_owned().into_bytes(),
 *                            spender_id,
 *                            amount,
 *                        ));
 *                    } else {
 *                        assert!(Asset::approve(
 *                            Origin::signed(sender_id),
 *                            ticker.to_owned().into_bytes(),
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
