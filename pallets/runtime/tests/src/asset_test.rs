use crate::{
    //contract_test::{create_se_template, flipper},
    ext_builder::{ExtBuilder, MockProtocolBaseFees},
    pips_test::assert_balance,
    storage::{
        add_secondary_key, make_account_without_cdd, provide_scope_claim,
        provide_scope_claim_to_multiple_parties, register_keyring_account, root, Checkpoint,
        TestStorage, User,
    },
};
use chrono::prelude::Utc;
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
    IterableStorageDoubleMap, IterableStorageMap, StorageDoubleMap, StorageMap,
};
use hex_literal::hex;
use ink_primitives::hash as FunctionSelectorHasher;
use pallet_asset::checkpoint::ScheduleSpec;
use pallet_asset::{
    self as asset, AssetOwnershipRelation, ClassicTickerImport, ClassicTickerRegistration,
    ClassicTickers, Config as AssetConfig, ScopeIdOf, SecurityToken, TickerRegistration,
    TickerRegistrationConfig, Tickers,
};
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use pallet_statistics as statistics;
use polymesh_common_utilities::{
    constants::*,
    protocol_fee::ProtocolOp,
    traits::balances::Memo,
    traits::checkpoint::{ScheduleId, StoredSchedule},
    traits::CddAndFeeDetails as _,
    SystematicIssuers,
};
use polymesh_primitives::ethereum;
use polymesh_primitives::{
    agent::AgentGroup,
    asset::{AssetName, AssetType, FundingRoundName},
    calendar::{
        CalendarPeriod, CalendarUnit, CheckpointId, CheckpointSchedule, FixedOrVariableCalendarUnit,
    },
    AccountId, AssetIdentifier, AssetPermissions, AuthorizationData, AuthorizationError, Document,
    DocumentId, IdentityId, InvestorUid, Moment, Permissions, PortfolioId, PortfolioName,
    SecondaryKey, Signatory, Ticker,
};
use rand::Rng;
use sp_io::hashing::keccak_256;
use sp_runtime::AnySignature;
use sp_std::{
    convert::{From, TryFrom, TryInto},
    iter,
};
use test_client::AccountKeyring;

type Identity = identity::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
//type Contracts = pallet_contracts::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type AssetError = asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Config>::Origin;
type DidRecords = identity::DidRecords<TestStorage>;
type Statistics = statistics::Module<TestStorage>;
type AssetGenesis = asset::GenesisConfig<TestStorage>;
type System = frame_system::Module<TestStorage>;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type FeeError = pallet_protocol_fee::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type StoreCallMetadata = pallet_permissions::StoreCallMetadata<TestStorage>;

fn now() -> u64 {
    Utc::now().timestamp() as _
}

fn set_time_to_now() {
    Timestamp::set_timestamp(now());
}

crate fn max_len() -> u32 {
    <TestStorage as pallet_base::Config>::MaxLen::get()
}

crate fn max_len_bytes<R: From<Vec<u8>>>(add: u32) -> R {
    bytes_of_len(b'A', (max_len() + add) as usize)
}

macro_rules! assert_too_long {
    ($e:expr) => {
        let e_result = $e;
        assert_noop!(e_result, pallet_base::Error::<TestStorage>::TooLong);
    };
}

crate fn token(name: &[u8], owner_did: IdentityId) -> (Ticker, SecurityToken) {
    let ticker = Ticker::try_from(name).unwrap();
    let token = SecurityToken {
        owner_did,
        total_supply: TOTAL_SUPPLY,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    (ticker, token)
}

crate fn a_token(owner_did: IdentityId) -> (Ticker, SecurityToken) {
    token(b"A", owner_did)
}

crate fn an_asset(owner: User, divisible: bool) -> Ticker {
    let (ticker, mut token) = a_token(owner.did);
    token.divisible = divisible;
    assert_ok!(basic_asset(owner, ticker, &token));
    ticker
}

fn has_ticker_record(ticker: Ticker) -> bool {
    DidRecords::contains_key(Identity::get_token_did(&ticker).unwrap())
}

fn asset_with_ids(
    owner: User,
    ticker: Ticker,
    token: &SecurityToken,
    ids: Vec<AssetIdentifier>,
) -> DispatchResult {
    Asset::base_create_asset_and_mint(
        owner.origin(),
        ticker.as_ref().into(),
        ticker,
        token.total_supply,
        token.divisible,
        token.asset_type.clone(),
        ids,
        None,
    )?;
    assert_eq!(Asset::balance_of(&ticker, owner.did), token.total_supply);
    Ok(())
}

crate fn basic_asset(owner: User, ticker: Ticker, token: &SecurityToken) -> DispatchResult {
    asset_with_ids(owner, ticker, token, vec![])
}

crate fn create_token(owner: User) -> (Ticker, SecurityToken) {
    let r = a_token(owner.did);
    assert_ok!(basic_asset(owner, r.0, &r.1));
    r
}

crate fn allow_all_transfers(ticker: Ticker, owner: User) {
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        vec![]
    ));
}

/*
fn setup_se_template(creator: User, create_instance: bool) -> AccountId {
    let (code_hash, wasm) = flipper();

    // Create SE template and instantiate with empty salt.
    if create_instance {
        create_se_template(creator.acc(), creator.did, 0, code_hash, wasm);
    }

    Contracts::contract_address(&creator.acc(), &code_hash, &[])
}
*/

crate fn transfer(ticker: Ticker, from: User, to: User, amount: u128) -> DispatchResult {
    Asset::base_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        &ticker,
        amount,
    )
}

fn cusip() -> AssetIdentifier {
    AssetIdentifier::cusip(*b"037833100").unwrap()
}

const ASSET_IDENTIFIERS: Vec<AssetIdentifier> = Vec::new();
const FUNDING_ROUND: Option<FundingRoundName> = None;
const TOTAL_SUPPLY: u128 = 1_000_000_000u128;

/// Generates a new portfolio for `owner` using the given `name`.
fn new_portfolio(owner: AccountId, name: &str) -> PortfolioId {
    let portfolio_name = PortfolioName::from(name);
    let did = Identity::key_to_identity_dids(owner.clone());

    Portfolio::create_portfolio(Origin::signed(owner), portfolio_name.clone())
        .expect("New portfolio cannot be created");

    let (portfolio_num, _) = pallet_portfolio::Portfolios::iter_prefix(&did)
        .find(|(_, name)| name == &portfolio_name)
        .unwrap();

    PortfolioId::user_portfolio(did, portfolio_num)
}

/// Returns a `FundingRoundName` which exceeds the maximum length defined in `AssetConfig`.
fn exceeded_funding_round_name() -> FundingRoundName {
    let funding_round_max_length =
        <TestStorage as AssetConfig>::FundingRoundNameMaxLength::get() + 1;

    iter::repeat(b'A')
        .take(funding_round_max_length as usize)
        .collect::<Vec<_>>()
        .into()
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
        let owner = User::new(AccountKeyring::Dave);

        // Expected token entry
        let (ticker, token) = a_token(owner.did);
        assert!(!has_ticker_record(ticker));

        let funding_round_name: FundingRoundName = b"round1".into();
        let create = |supply| {
            Asset::base_create_asset_and_mint(
                owner.origin(),
                ticker.as_ref().into(),
                ticker,
                supply,
                true,
                token.asset_type.clone(),
                Vec::new(),
                Some(funding_round_name.clone()),
            )
        };

        assert_noop!(
            create(1_000_000_000_000_000_000_000_000), // Total supply over the limit.
            AssetError::TotalSupplyAboveLimit
        );

        // Issuance is successful.
        assert_ok!(create(token.total_supply));

        // Check the update investor count for the newly created asset
        assert_eq!(Statistics::investor_count(ticker), 1);

        // A correct entry is added
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(
            Asset::asset_ownership_relation(token.owner_did, ticker),
            AssetOwnershipRelation::AssetOwned
        );
        assert!(has_ticker_record(ticker));
        assert_eq!(Asset::funding_round(ticker), funding_round_name.clone());

        // Unauthorized agents cannot rename the token.
        let eve = User::new(AccountKeyring::Eve);
        assert_noop!(
            Asset::rename_asset(eve.origin(), ticker, vec![0xde, 0xad, 0xbe, 0xef].into()),
            EAError::UnauthorizedAgent
        );
        // The token should remain unchanged in storage.
        assert_eq!(Asset::token_details(ticker), token);
        // Rename the token and check storage has been updated.
        let new: AssetName = [0x42].into();
        assert_ok!(Asset::rename_asset(owner.origin(), ticker, new.clone()));
        assert_eq!(Asset::asset_names(ticker), new);
        assert!(Asset::identifiers(ticker).is_empty());
    });
}

#[test]
fn valid_transfers_pass() {
    let eve = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![eve.clone()])
        .build()
        .execute_with(|| {
            set_time_to_now();

            let owner = User::new(AccountKeyring::Dave);
            let alice = User::new(AccountKeyring::Alice);

            // Create asset.
            let (ticker, token) = a_token(owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice.did, owner.did], ticker, eve);

            allow_all_transfers(ticker, owner);

            // Should fail as sender matches receiver.
            let transfer = |from, to| transfer(ticker, from, to, 500);
            assert_noop!(transfer(owner, owner), AssetError::InvalidTransfer);
            assert_ok!(transfer(owner, alice));

            assert_eq!(Asset::balance_of(&ticker, owner.did), TOTAL_SUPPLY - 500);
            assert_eq!(Asset::balance_of(&ticker, alice.did), 500);
        })
}

#[test]
fn issuers_can_redeem_tokens() {
    let alice = AccountKeyring::Alice.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![alice.clone()])
        .build()
        .execute_with(|| {
            set_time_to_now();

            let owner = User::new(AccountKeyring::Dave);
            let bob = User::new(AccountKeyring::Bob);

            // Create asset.
            let (ticker, token) = a_token(owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[owner.did], ticker, alice);

            assert_noop!(
                Asset::redeem(bob.origin(), ticker, token.total_supply),
                EAError::UnauthorizedAgent
            );

            assert_noop!(
                Asset::redeem(owner.origin(), ticker, token.total_supply + 1),
                PortfolioError::InsufficientPortfolioBalance
            );

            assert_ok!(Asset::redeem(owner.origin(), ticker, token.total_supply));

            assert_eq!(Asset::balance_of(&ticker, owner.did), 0);
            assert_eq!(Asset::token_details(&ticker).total_supply, 0);

            assert_noop!(
                Asset::redeem(owner.origin(), ticker, 1),
                PortfolioError::InsufficientPortfolioBalance
            );
        })
}

fn default_transfer(from: User, to: User, ticker: Ticker, val: u128) {
    assert_ok!(Asset::unsafe_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        &ticker,
        val,
    ));
}

#[test]
fn checkpoints_fuzz_test() {
    println!("Starting");
    for _ in 0..10 {
        // When fuzzing in local, feel free to bump this number to add more fuzz runs.
        ExtBuilder::default().build().execute_with(|| {
            set_time_to_now();

            let owner = User::new(AccountKeyring::Dave);
            let bob = User::new(AccountKeyring::Bob);

            // Create asset.
            let (ticker, token) = a_token(owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));

            allow_all_transfers(ticker, owner);

            let mut owner_balance: [u128; 100] = [TOTAL_SUPPLY; 100];
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
                    default_transfer(owner, bob, ticker, 1);
                }
                assert_ok!(Checkpoint::create_checkpoint(owner.origin(), ticker));
                let bal_at = |id, did| Asset::get_balance_at(ticker, did, CheckpointId(id));
                let check = |id, idx| {
                    assert_eq!(bal_at(id, owner.did), owner_balance[idx]);
                    assert_eq!(bal_at(id, bob.did), bob_balance[idx]);
                };
                let x: u64 = u64::try_from(j).unwrap();
                check(0, j);
                check(0, j);
                check(1, 1);
                check(x - 1, j - 1);
                check(x, j);
                check(x + 1, j);
                check(1000, j);
            }
        });
    }
}

#[test]
fn register_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (ticker, token) = a_token(owner.did);
        let identifiers = vec![AssetIdentifier::isin(*b"US0378331005").unwrap()];
        assert_ok!(asset_with_ids(owner, ticker, &token, identifiers.clone()));

        let register = |ticker| Asset::register_ticker(owner.origin(), ticker);

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
        let stored_token = Asset::token_details(&ticker);
        assert_eq!(stored_token.asset_type, token.asset_type);
        assert_eq!(Asset::identifiers(ticker), identifiers);
        assert_noop!(
            register(Ticker::try_from(&[b'A'][..]).unwrap()),
            AssetError::AssetAlreadyCreated
        );

        assert_noop!(
            register(
                Ticker::try_from(&[b'A', b'A', b'A', b'A', b'A', b'A', b'A', b'A', b'A'][..])
                    .unwrap()
            ),
            AssetError::TickerTooLong
        );

        let ticker = Ticker::try_from(&[b'A', b'A'][..]).unwrap();

        assert_eq!(Asset::is_ticker_available(&ticker), true);

        assert_ok!(register(ticker));

        assert_eq!(
            Asset::asset_ownership_relation(owner.did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_noop!(
            Asset::register_ticker(alice.origin(), ticker),
            AssetError::TickerAlreadyRegistered
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        Timestamp::set_timestamp(now() + 10001);

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), true);

        for bs in &[
            [b'A', 31, b'B'].as_ref(),
            [127, b'A'].as_ref(),
            [b'A', 0, 0, 0, b'A'].as_ref(),
        ] {
            assert_noop!(
                register(Ticker::try_from(&bs[..]).unwrap()),
                AssetError::TickerNotAscii
            );
        }

        assert_ok!(register(Ticker::try_from(&[b' ', b'A', b'~'][..]).unwrap()));
    })
}

#[test]
fn transfer_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let ticker = Ticker::try_from(&[b'A', b'A'][..]).unwrap();

        assert_eq!(Asset::is_ticker_available(&ticker), true);
        assert_ok!(Asset::register_ticker(owner.origin(), ticker));

        let auth_id_alice = Identity::add_auth(
            owner.did,
            Signatory::from(alice.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), true);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice.did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), false);

        assert_noop!(
            Asset::accept_ticker_transfer(alice.origin(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        assert_eq!(
            Asset::asset_ownership_relation(owner.did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_ok!(Asset::accept_ticker_transfer(alice.origin(), auth_id_alice));

        assert_eq!(
            Asset::asset_ownership_relation(owner.did, ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Asset::asset_ownership_relation(alice.did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_eq!(
            Asset::asset_ownership_relation(alice.did, ticker),
            AssetOwnershipRelation::TickerOwned
        );

        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id_bob),
            "Illegal use of Authorization"
        );

        let add_auth = |auth, expiry| {
            Identity::add_auth(alice.did, Signatory::from(bob.did), auth, Some(expiry))
        };

        let auth_id = add_auth(AuthorizationData::TransferTicker(ticker), now() - 100);

        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id),
            "Authorization expired"
        );

        let auth_id = add_auth(AuthorizationData::Custom(ticker), now() + 100);

        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id),
            AuthorizationError::BadType
        );

        let auth_id = add_auth(AuthorizationData::TransferTicker(ticker), now() + 100);

        assert_ok!(Asset::accept_ticker_transfer(bob.origin(), auth_id));

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, alice.did), false);
        assert_eq!(Asset::is_ticker_registry_valid(&ticker, bob.did), true);
        assert_eq!(Asset::is_ticker_available(&ticker), false);
    })
}

#[test]
fn controller_transfer() {
    let eve = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![eve.clone()])
        .build()
        .execute_with(|| {
            set_time_to_now();

            let owner = User::new(AccountKeyring::Dave);
            let alice = User::new(AccountKeyring::Alice);

            // Create asset.
            let (ticker, token) = a_token(owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));

            // Provide scope claim to sender and receiver of the transaction.
            provide_scope_claim_to_multiple_parties(&[alice.did, owner.did], ticker, eve);

            allow_all_transfers(ticker, owner);

            // Should fail as sender matches receiver.
            assert_noop!(
                transfer(ticker, owner, owner, 500),
                AssetError::InvalidTransfer
            );

            assert_ok!(transfer(ticker, owner, alice, 500));

            let balance_of = |did| Asset::balance_of(&ticker, did);
            let balance_alice = balance_of(alice.did);
            let balance_owner = balance_of(owner.did);
            assert_eq!(balance_owner, TOTAL_SUPPLY - 500);
            assert_eq!(balance_alice, 500);

            assert_ok!(Asset::controller_transfer(
                owner.origin(),
                ticker,
                100,
                PortfolioId::default_portfolio(alice.did),
            ));
            assert_eq!(balance_of(owner.did), balance_owner + 100);
            assert_eq!(balance_of(alice.did), balance_alice - 100);
        })
}

#[test]
fn transfer_token_ownership() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create asset.
        let (ticker, token) = a_token(owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        let auth_id_alice = Identity::add_auth(
            owner.did,
            Signatory::from(alice.did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        let auth_id_bob = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );

        assert_eq!(Asset::token_details(&ticker).owner_did, owner.did);

        assert_noop!(
            Asset::accept_asset_ownership_transfer(alice.origin(), auth_id_alice + 1),
            "Authorization does not exist"
        );

        assert_eq!(
            Asset::asset_ownership_relation(owner.did, ticker),
            AssetOwnershipRelation::AssetOwned
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            alice.origin(),
            auth_id_alice
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, alice.did);
        assert_eq!(
            Asset::asset_ownership_relation(owner.did, ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Asset::asset_ownership_relation(alice.did, ticker),
            AssetOwnershipRelation::AssetOwned
        );

        assert_ok!(ExternalAgents::unchecked_add_agent(
            ticker,
            alice.did,
            AgentGroup::Full
        ));
        assert_ok!(ExternalAgents::abdicate(owner.origin(), ticker));
        assert_noop!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id_bob),
            EAError::UnauthorizedAgent
        );

        let mut auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(ticker),
            Some(now() - 100),
        );

        assert_noop!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            "Authorization expired"
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::Custom(ticker),
            Some(now() + 100),
        );

        assert_noop!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            AuthorizationError::BadType
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(Ticker::try_from(&[0x50][..]).unwrap()),
            Some(now() + 100),
        );

        assert_noop!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            AssetError::NoSuchAsset
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(ticker),
            Some(now() + 100),
        );

        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob.origin(),
            auth_id
        ));
        assert_eq!(Asset::token_details(&ticker).owner_did, bob.did);
    })
}

#[test]
fn update_identifiers() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Expected token entry
        let (ticker, token) = a_token(owner.did);
        assert!(!has_ticker_record(ticker));

        let create = |idents| asset_with_ids(owner, ticker, &token, idents);
        let update = |idents| Asset::update_identifiers(owner.origin(), ticker, idents);

        // Create: A correct entry was added.
        assert_ok!(create(vec![cusip()]));
        assert_eq!(Asset::token_details(ticker), token);
        assert_eq!(Asset::identifiers(ticker), vec![cusip()]);

        // Create: A bad entry was rejected.
        assert_noop!(
            update(vec![cusip(), AssetIdentifier::CUSIP(*b"aaaa_aaaa")]),
            AssetError::InvalidAssetIdentifier
        );

        // Update: A bad entry was rejected.
        let mut updated_identifiers = vec![
            AssetIdentifier::cusip(*b"17275R102").unwrap(),
            AssetIdentifier::isin(*b"US0378331005").unwrap(),
            AssetIdentifier::CUSIP(*b"aaaa_aaaa"),
        ];
        assert_noop!(
            update(updated_identifiers.clone()),
            AssetError::InvalidAssetIdentifier
        );

        // Update: A correct entry was added.
        updated_identifiers.pop().unwrap();
        assert_ok!(update(updated_identifiers.clone()));
        assert_eq!(Asset::identifiers(ticker), updated_identifiers);
    });
}

#[test]
fn adding_removing_documents() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Create asset.
        let (ticker, token) = a_token(owner.did);
        assert!(!has_ticker_record(ticker));
        assert_ok!(basic_asset(owner, ticker, &token));

        let documents = vec![
            Document {
                name: b"A".into(),
                uri: b"www.a.com".into(),
                content_hash: [1u8; 64][..].try_into().unwrap(),
                doc_type: Some(b"foo".into()),
                filing_date: Some(42),
            },
            Document {
                name: b"B".into(),
                uri: b"www.b.com".into(),
                content_hash: [2u8; 64][..].try_into().unwrap(),
                doc_type: None,
                filing_date: None,
            },
        ];

        assert_ok!(Asset::add_documents(
            owner.origin(),
            documents.clone(),
            ticker
        ));

        for (idx, doc) in documents.into_iter().enumerate() {
            assert_eq!(doc, Asset::asset_documents(ticker, DocumentId(idx as u32)));
        }

        assert_ok!(Asset::remove_documents(
            owner.origin(),
            (0..=1).map(DocumentId).collect(),
            ticker
        ));

        assert_eq!(asset::AssetDocuments::iter_prefix_values(ticker).count(), 0);
    });
}

/*
fn add_smart_ext(
    owner: User,
    ticker: Ticker,
    ty: SmartExtensionType,
) -> (AccountId, SmartExtension<AccountId>) {
    let id = setup_se_template(owner, true);
    let details = SmartExtension {
        extension_type: ty.clone(),
        extension_name: b"EXT".into(),
        extension_id: id.clone(),
        is_archive: false,
    };
    assert_ok!(Asset::add_extension(
        owner.origin(),
        ticker,
        details.clone(),
    ));
    assert_eq!(Asset::extension_details((ticker, id.clone())), details);
    assert_eq!(Asset::extensions((ticker, ty)), [id.clone()]);
    (id, details)
}

#[track_caller]
fn smart_ext_test(logic: impl FnOnce(User, Ticker)) {
    ExtBuilder::default()
        .set_max_tms_allowed(10)
        .set_contracts_put_code(true)
        .build()
        .execute_with(|| {
            let owner = User::new(AccountKeyring::Dave);
            let (ticker, token) = a_token(owner.did);
            assert!(!has_ticker_record(ticker));
            assert_ok!(asset_with_ids(owner, ticker, &token, vec![cusip()]));
            logic(owner, ticker)
        });
}

#[test]
fn add_extension_limited() {
    smart_ext_test(|owner, ticker| {
        let id = setup_se_template(owner, true);
        let add_ext = |ty: &Vec<u8>, name: &Vec<u8>| {
            let details = SmartExtension {
                extension_type: SmartExtensionType::Custom(ty.clone().into()),
                extension_name: name.clone().into(),
                extension_id: id.clone().into(),
                is_archive: false,
            };
            Asset::add_extension(owner.origin(), ticker.clone(), details)
        };
        let id: Vec<u8> = max_len_bytes(0);
        let invalid_id: Vec<u8> = max_len_bytes(1);

        assert_too_long!(add_ext(&invalid_id, &id));
        assert_too_long!(add_ext(&id, &invalid_id));
        pallet_asset::CompatibleSmartExtVersion::insert(
            SmartExtensionType::Custom(id.clone().into()),
            5000,
        );
        assert_ok!(add_ext(&id, &id));
    });
}

#[test]
fn add_extension_successfully() {
    smart_ext_test(|owner, ticker| {
        add_smart_ext(owner, ticker, SmartExtensionType::TransferManager);
    });
}

#[test]
fn add_same_extension_should_fail() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let (_, details) = add_smart_ext(owner, ticker, SmartExtensionType::TransferManager);

        // And again, unsuccessfully.
        assert_noop!(
            Asset::add_extension(owner.origin(), ticker, details),
            AssetError::ExtensionAlreadyPresent
        );
    });
}

#[test]
fn should_successfully_archive_extension() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let (id, _) = add_smart_ext(owner, ticker, SmartExtensionType::Offerings);

        // Archive it.
        assert_ok!(Asset::archive_extension(owner.origin(), ticker, id.clone()));
        assert_eq!(Asset::extension_details((ticker, id)).is_archive, true);
    });
}

#[test]
fn should_fail_to_archive_an_already_archived_extension() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let (id, _) = add_smart_ext(owner, ticker, SmartExtensionType::Offerings);

        // Archive it.
        let archive = || Asset::archive_extension(owner.origin(), ticker, id.clone());
        assert_ok!(archive());
        assert_eq!(
            Asset::extension_details((ticker, id.clone())).is_archive,
            true
        );

        // And again, unsuccessfully.
        assert_noop!(archive(), AssetError::AlreadyArchived);
    });
}

#[test]
fn should_fail_to_archive_a_non_existent_extension() {
    smart_ext_test(|owner, ticker| {
        // Archive that which doesn't exist.
        assert_noop!(
            Asset::archive_extension(owner.origin(), ticker, AccountKeyring::Bob.to_account_id()),
            AssetError::NoSuchSmartExtension
        );
    });
}

#[test]
fn should_successfully_unarchive_an_extension() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let (id, _) = add_smart_ext(owner, ticker, SmartExtensionType::Offerings);

        // Archive it.
        let is_archive = || Asset::extension_details((ticker, id.clone())).is_archive;
        let archive = || {
            assert_ok!(Asset::archive_extension(owner.origin(), ticker, id.clone()));
            assert_eq!(is_archive(), true);
        };
        archive();

        // Unarchive it.
        assert_ok!(Asset::unarchive_extension(
            owner.origin(),
            ticker,
            id.clone()
        ));
        assert_eq!(is_archive(), false);

        // Roundtrip.
        archive();
    });
}

#[test]
fn should_fail_to_unarchive_an_already_unarchived_extension() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let (id, _) = add_smart_ext(owner, ticker, SmartExtensionType::Offerings);

        // Archive it.
        assert_ok!(Asset::archive_extension(owner.origin(), ticker, id.clone()));
        let is_archive = || Asset::extension_details((ticker, id.clone())).is_archive;
        assert_eq!(is_archive(), true);

        // Unarchive it.
        let unarchive = || Asset::unarchive_extension(owner.origin(), ticker, id.clone());
        assert_ok!(unarchive());
        assert_eq!(is_archive(), false);

        // And again, unsuccessfully.
        assert_noop!(unarchive(), AssetError::AlreadyUnArchived);
    });
}
*/

#[test]
fn freeze_unfreeze_asset() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let (ticker, token) = a_token(owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        allow_all_transfers(ticker, owner);

        assert_noop!(
            Asset::freeze(bob.origin(), ticker),
            EAError::UnauthorizedAgent
        );
        assert_noop!(
            Asset::unfreeze(owner.origin(), ticker),
            AssetError::NotFrozen
        );
        assert_ok!(Asset::freeze(owner.origin(), ticker));
        assert_noop!(
            Asset::freeze(owner.origin(), ticker),
            AssetError::AlreadyFrozen
        );

        // Attempt to transfer token ownership.
        let auth_id = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(ticker),
            None,
        );
        assert_ok!(Asset::accept_asset_ownership_transfer(
            bob.origin(),
            auth_id
        ));

        // Not enough; bob needs to become an agent.
        assert_noop!(
            Asset::unfreeze(bob.origin(), ticker),
            EAError::UnauthorizedAgent
        );

        assert_ok!(ExternalAgents::unchecked_add_agent(
            ticker,
            bob.did,
            AgentGroup::Full
        ));
        assert_ok!(Asset::unfreeze(bob.origin(), ticker));
        assert_noop!(Asset::unfreeze(bob.origin(), ticker), AssetError::NotFrozen);
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
    let alice = AccountKeyring::Alice.to_account_id();
    let alice_id = register_keyring_account(AccountKeyring::Alice).unwrap();
    let _charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();
    let bob = AccountKeyring::Bob.to_account_id();

    // 1. Add Bob as signatory to Alice ID.
    let bob_signatory = Signatory::Account(AccountKeyring::Bob.to_account_id());
    add_secondary_key(alice_id, bob_signatory);
    assert_ok!(Balances::transfer_with_memo(
        Origin::signed(alice.clone()),
        bob.clone().into(),
        1_000,
        Some(Memo::from("Bob funding"))
    ));

    // 2. Bob can create token
    let (ticker_1, token_1) = a_token(alice_id);
    assert_ok!(Asset::base_create_asset_and_mint(
        Origin::signed(bob),
        ticker_1.as_ref().into(),
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
    let (_ticker_2, _token_2) = a_token(alice_id);
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
    // assert_noop!(
    //     create_token_result,
    //     DispatchError::Other("Current identity is none and key is not linked to any identity")
    // );
}

#[test]
fn test_can_transfer_rpc() {
    let eve = AccountKeyring::Eve.to_account_id();
    ExtBuilder::default()
        .cdd_providers(vec![eve.clone()])
        .monied(true)
        .balance_factor(1)
        .build()
        .execute_with(|| {
            let owner = User::new(AccountKeyring::Alice);
            let bob = User::new(AccountKeyring::Bob);

            // Create asset.
            let (ticker, mut token) = a_token(owner.did);
            token.divisible = false;
            token.total_supply = 1_000 * currency::ONE_UNIT;
            assert_ok!(basic_asset(owner, ticker, &token));

            // Provide scope claim for sender and receiver.
            provide_scope_claim_to_multiple_parties(&[owner.did, bob.did], ticker, eve);

            let unsafe_can_transfer_result = |from_did, to_did, amount| {
                Asset::unsafe_can_transfer(
                    None,
                    PortfolioId::default_portfolio(from_did),
                    None,
                    PortfolioId::default_portfolio(to_did),
                    &ticker,
                    amount, // It only passed when it should be the multiple of currency::ONE_UNIT
                )
                .unwrap()
            };

            // case 1: When passed invalid granularity
            assert_eq!(
                unsafe_can_transfer_result(owner.did, bob.did, 100),
                INVALID_GRANULARITY
            );

            // Case 2: when from_did balance is 0
            assert_eq!(
                unsafe_can_transfer_result(bob.did, owner.did, 100 * currency::ONE_UNIT),
                ERC1400_INSUFFICIENT_BALANCE
            );

            // Comment below test case, These will be un-commented when we improved the DID check.

            // // Case 4: When sender doesn't posses a valid cdd
            // // 4.1: Create Identity who doesn't posses cdd
            // let (from_without_cdd_signed, from_without_cdd_did) = make_account_with_uid(AccountKeyring::Ferdie.to_account_id()).unwrap();
            // // Execute can_transfer
            // assert_eq!(
            //     Asset::unsafe_can_transfer(
            //         AccountKeyring::Ferdie.to_account_id(),
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
            //         AccountKeyring::Alice.to_account_id(),
            //         ticker,
            //         Some(alice_did),
            //         Some(from_without_cdd_did),
            //         5 * currency::ONE_UNIT
            //     ).unwrap(),
            //     INVALID_RECEIVER_DID
            // );

            // Case 6: When Asset transfer is frozen
            // 6.1: pause the transfer
            assert_ok!(Asset::freeze(owner.origin(), ticker));
            assert_eq!(
                unsafe_can_transfer_result(owner.did, bob.did, 20 * currency::ONE_UNIT),
                ERC1400_TRANSFERS_HALTED
            );
            assert_ok!(Asset::unfreeze(owner.origin(), ticker));

            // Case 7: when transaction get success by the compliance_manager
            allow_all_transfers(ticker, owner);
            assert_eq!(
                unsafe_can_transfer_result(owner.did, bob.did, 20 * currency::ONE_UNIT),
                ERC1400_TRANSFER_SUCCESS
            );
        })
}

/*
#[test]
fn check_functionality_of_remove_extension() {
    smart_ext_test(|owner, ticker| {
        // Add it.
        let ty = SmartExtensionType::TransferManager;
        let (id, _) = add_smart_ext(owner, ticker, ty.clone());

        // Remove it
        let remove = || Asset::remove_smart_extension(owner.origin(), ticker, id.clone());
        assert_ok!(remove());
        assert_eq!(Asset::extensions((ticker, ty)), vec![]);

        // Remove it again => error.
        assert_noop!(remove(), AssetError::NoSuchSmartExtension);
    });
}
*/

// Classic token tests:

fn ticker(name: &str) -> Ticker {
    name.as_bytes().try_into().unwrap()
}

fn default_classic() -> ClassicTickerImport {
    ClassicTickerImport {
        eth_owner: <_>::default(),
        ticker: <_>::default(),
        is_created: false,
        is_contract: false,
    }
}

fn default_reg_config() -> TickerRegistrationConfig<u64> {
    TickerRegistrationConfig {
        max_ticker_length: 8,
        registration_length: Some(10000),
    }
}

fn alice_secret_key() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn bob_secret_key() -> secp256k1::SecretKey {
    secp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
}

fn sorted<K: Ord + Clone, V>(iter: impl IntoIterator<Item = (K, V)>) -> Vec<(K, V)> {
    let mut vec: Vec<_> = iter.into_iter().collect();
    vec.sort_by_key(|x| x.0.clone());
    vec
}

fn with_asset_genesis(genesis: AssetGenesis) -> ExtBuilder {
    ExtBuilder::default().adjust(Box::new(move |storage| {
        genesis.assimilate_storage(storage).unwrap();
    }))
}

fn test_asset_genesis(genesis: AssetGenesis) {
    with_asset_genesis(genesis).build().execute_with(|| {});
}

#[test]
#[should_panic = "lowercase ticker"]
fn classic_ticker_genesis_lowercase() {
    test_asset_genesis(AssetGenesis {
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker: ticker("lower"),
            ..default_classic()
        }],
        ..<_>::default()
    });
}

#[test]
#[should_panic = "TickerTooLong"]
fn classic_ticker_genesis_too_long() {
    test_asset_genesis(AssetGenesis {
        classic_migration_tconfig: TickerRegistrationConfig {
            max_ticker_length: 3,
            registration_length: None,
        },
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker: ticker("ACME"),
            ..default_classic()
        }],
        ..<_>::default()
    });
}

#[test]
#[should_panic = "TickerAlreadyRegistered"]
fn classic_ticker_genesis_already_registered_sys_did() {
    let import = ClassicTickerImport {
        ticker: ticker("ACME"),
        ..default_classic()
    };
    test_asset_genesis(AssetGenesis {
        classic_migration_tconfig: default_reg_config(),
        classic_migration_tickers: vec![import.clone(), import],
        ..<_>::default()
    });
}

#[test]
#[should_panic = "TickerAlreadyRegistered"]
fn classic_ticker_genesis_already_registered_other_did() {
    let import_a = ClassicTickerImport {
        ticker: ticker("ACME"),
        ..default_classic()
    };
    let import_b = ClassicTickerImport {
        is_contract: true,
        ..import_a.clone()
    };
    test_asset_genesis(AssetGenesis {
        classic_migration_contract_did: 1.into(),
        classic_migration_tconfig: default_reg_config(),
        classic_migration_tickers: vec![import_a, import_b],
        ..<_>::default()
    });
}

#[test]
fn classic_ticker_genesis_works() {
    let alice_eth = ethereum::EthereumAddress(*b"0x012345678987654321");
    let bob_eth = ethereum::EthereumAddress(*b"0x212345678987654321");
    let charlie_eth = ethereum::EthereumAddress(*b"0x512345678987654321");

    // Define actual on-genesis asset config.
    let classic_migration_tickers = vec![
        ClassicTickerImport {
            eth_owner: alice_eth,
            ticker: ticker("ALPHA"),
            is_created: false,
            is_contract: false,
        },
        ClassicTickerImport {
            eth_owner: alice_eth,
            ticker: ticker("BETA"),
            is_created: true,
            is_contract: false,
        },
        ClassicTickerImport {
            eth_owner: bob_eth,
            ticker: ticker("GAMMA"),
            is_created: false,
            is_contract: true,
        },
        ClassicTickerImport {
            eth_owner: charlie_eth,
            ticker: ticker("DELTA"),
            is_created: true,
            is_contract: true,
        },
    ];
    let contract_did = IdentityId::from(1337);
    let registration_length = Some(42);
    let standard_config = default_reg_config();
    let genesis = AssetGenesis {
        classic_migration_tickers,
        ticker_registration_config: standard_config.clone(),
        classic_migration_contract_did: contract_did,
        classic_migration_tconfig: TickerRegistrationConfig {
            registration_length,
            ..standard_config
        },
        reserved_country_currency_codes: vec![],
        versions: vec![],
    };

    // Define expected ticker data after genesis.
    let reg = |owner, expiry| TickerRegistration { expiry, owner };
    let cm_did = SystematicIssuers::ClassicMigration.as_id();
    let mut tickers = vec![
        (ticker("ALPHA"), reg(cm_did, registration_length)),
        (ticker("BETA"), reg(cm_did, registration_length)),
        (ticker("GAMMA"), reg(contract_did, registration_length)),
        (ticker("DELTA"), reg(contract_did, registration_length)),
    ];

    // Define expected classic ticker data after genesis.
    let classic_reg = |eth_owner, is_created| ClassicTickerRegistration {
        eth_owner,
        is_created,
    };
    let classic_tickers = vec![
        (ticker("ALPHA"), classic_reg(alice_eth, false)),
        (ticker("BETA"), classic_reg(alice_eth, true)),
        (ticker("GAMMA"), classic_reg(bob_eth, false)),
        (ticker("DELTA"), classic_reg(charlie_eth, true)),
    ];

    with_asset_genesis(genesis).build().execute_with(move || {
        // Dave enters the room.
        let rt = User::new(AccountKeyring::Dave);

        // Ensure we have cm_did != contract_did != rt_did.
        assert_ne!(cm_did, contract_did);
        assert_ne!(rt.did, cm_did);
        assert_ne!(rt.did, contract_did);

        // Add another ticker to contrast expiry and DID and expect it.
        let expiry = standard_config.registration_length;
        assert_ok!(Asset::register_ticker(rt.origin(), ticker("EPSILON")));
        tickers.push((ticker("EPSILON"), reg(rt.did, expiry)));

        // Ensure actual permutes expected.
        assert_eq!(sorted(Tickers::<TestStorage>::iter()), sorted(tickers));
        assert_eq!(sorted(ClassicTickers::iter()), sorted(classic_tickers));
    });
}

#[test]
fn classic_ticker_register_works() {
    let mut config = default_reg_config();
    with_asset_genesis(AssetGenesis {
        ticker_registration_config: config.clone(),
        classic_migration_tconfig: config.clone(),
        // Let there be one classic ticker reserved already, to cause a conflict.
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker: ticker("ACME"),
            ..default_classic()
        }],
        ..<_>::default()
    })
    .build()
    .execute_with(move || {
        let alice_eth = ethereum::EthereumAddress(*b"0x012345678987654321");
        let mut classic = ClassicTickerImport {
            eth_owner: alice_eth,
            ticker: ticker("ACME"),
            is_created: true,
            is_contract: false,
        };

        let alice = User::new(AccountKeyring::Alice);

        // Only root can call.
        assert_noop!(
            Asset::reserve_classic_ticker(alice.origin(), classic, alice.did, config.clone()),
            DispatchError::BadOrigin
        );

        // ACME was already registered.
        assert_noop!(
            Asset::reserve_classic_ticker(root(), classic, alice.did, config.clone()),
            AssetError::TickerAlreadyRegistered
        );

        // Create BETA as an asset and fail to register it.
        classic.ticker = ticker("BETA");
        assert_ok!(Asset::base_create_asset_and_mint(
            alice.origin(),
            b"".into(),
            classic.ticker,
            0,
            false,
            <_>::default(),
            vec![],
            None,
        ));
        assert_noop!(
            Asset::reserve_classic_ticker(root(), classic, alice.did, config.clone()),
            AssetError::AssetAlreadyCreated
        );

        // Set max-length to 1 and check that it's enforced.
        // Also set and expect the expiry length.
        config.max_ticker_length = 1;
        classic.ticker = ticker("AB");
        assert_noop!(
            Asset::reserve_classic_ticker(root(), classic, alice.did, config.clone()),
            AssetError::TickerTooLong
        );
        classic.ticker = ticker("A");
        config.registration_length = Some(1337);
        let expiry = Some(Timestamp::get() + 1337);
        let assert = |classic, owner, is_created| {
            assert_ok!(Asset::reserve_classic_ticker(
                root(),
                classic,
                alice.did,
                config.clone()
            ));
            assert_eq!(
                Asset::ticker_registration(classic.ticker),
                TickerRegistration { owner, expiry },
            );
            assert_eq!(
                Asset::classic_ticker_registration(classic.ticker).unwrap(),
                ClassicTickerRegistration {
                    is_created,
                    eth_owner: alice_eth,
                },
            );
        };
        assert(classic, SystematicIssuers::ClassicMigration.as_id(), true);

        // Also test the contract situation.
        classic.ticker = ticker("B");
        classic.is_contract = true;
        classic.is_created = false;
        assert(classic, alice.did, false);
    });
}

#[test]
fn classic_ticker_no_such_classic_ticker() {
    let user = AccountKeyring::Alice.to_account_id();
    let cdd = AccountKeyring::Eve.to_account_id();
    let ticker_acme = ticker("ACME");
    let ticker_emca = ticker("EMCA");

    with_asset_genesis(AssetGenesis {
        classic_migration_tconfig: default_reg_config(),
        // There is a classic ticker, but not the one we're claiming.
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker: ticker_acme,
            ..default_classic()
        }],
        ..<_>::default()
    })
    .add_regular_users_from_accounts(&[user.clone()])
    .cdd_providers(vec![cdd])
    .build()
    .execute_with(|| {
        let signer = Origin::signed(user.clone());
        assert_noop!(
            Asset::claim_classic_ticker(signer, ticker_emca, ethereum::EcdsaSignature([0; 65])),
            AssetError::NoSuchClassicTicker
        );
    });
}

#[test]
fn classic_ticker_registered_by_other() {
    let user = AccountKeyring::Alice.to_account_id();
    let cdd = AccountKeyring::Bob.to_account_id();
    let ticker = ticker("ACME");

    with_asset_genesis(AssetGenesis {
        classic_migration_tconfig: default_reg_config(),
        // There is a classic ticker, but its not owned by sys DID.
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker,
            is_contract: true,
            ..default_classic()
        }],
        ..<_>::default()
    })
    .add_regular_users_from_accounts(&[user.clone()])
    .cdd_providers(vec![cdd])
    .build()
    .execute_with(|| {
        let signer = Origin::signed(user);
        assert_noop!(
            Asset::claim_classic_ticker(signer, ticker, ethereum::EcdsaSignature([0; 65])),
            AssetError::TickerAlreadyRegistered
        );
    });
}

#[test]
fn classic_ticker_expired_thus_available() {
    let ticker = ticker("ACME");
    let user = AccountKeyring::Dave.to_account_id();

    with_asset_genesis(AssetGenesis {
        classic_migration_tconfig: TickerRegistrationConfig {
            registration_length: Some(0),
            ..default_reg_config()
        },
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker,
            ..default_classic()
        }],
        ..<_>::default()
    })
    .add_regular_users_from_accounts(&[user.clone()])
    .cdd_providers(vec![AccountKeyring::Alice.to_account_id()])
    .build()
    .execute_with(|| {
        let signer = Origin::signed(user);
        Timestamp::set_timestamp(1);
        assert_noop!(
            Asset::claim_classic_ticker(signer, ticker, ethereum::EcdsaSignature([0; 65])),
            AssetError::TickerRegistrationExpired
        );
    });
}

#[test]
fn classic_ticker_garbage_signature() {
    let ticker = ticker("ACME");
    with_asset_genesis(AssetGenesis {
        classic_migration_tconfig: default_reg_config(),
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker,
            ..default_classic()
        }],
        ..<_>::default()
    })
    .build()
    .execute_with(|| {
        let rt_signer = Origin::signed(AccountKeyring::Dave.to_account_id());
        assert_noop!(
            Asset::claim_classic_ticker(rt_signer, ticker, ethereum::EcdsaSignature([0; 65])),
            pallet_permissions::Error::<TestStorage>::UnauthorizedCaller
        );
    });
}

#[test]
fn classic_ticker_not_owner() {
    let ticker = ticker("ACME");
    with_asset_genesis(AssetGenesis {
        classic_migration_tconfig: default_reg_config(),
        classic_migration_tickers: vec![ClassicTickerImport {
            ticker,
            eth_owner: ethereum::address(&alice_secret_key()),
            ..default_classic()
        }],
        ..<_>::default()
    })
    .build()
    .execute_with(|| {
        let signer = Origin::signed(AccountKeyring::Bob.to_account_id());
        let did = register_keyring_account(AccountKeyring::Bob).unwrap();
        let eth_sig = ethereum::eth_msg(did, b"classic_claim", &bob_secret_key());
        assert_noop!(
            Asset::claim_classic_ticker(signer, ticker, eth_sig),
            AssetError::NotAnOwner
        );
    });
}

#[test]
fn classic_ticker_claim_works() {
    let eth_owner = ethereum::address(&alice_secret_key());

    // Define all the classic ticker imports.
    let import = |name, is_created| ClassicTickerImport {
        eth_owner,
        ticker: ticker(name),
        is_created,
        is_contract: false,
    };
    let tickers = vec![
        import("ALPHA", false),
        import("BETA", true),
        import("GAMMA", true),
        import("DELTA", true),
        import("EPSILON", true),
        import("ZETA", true),
    ];

    // Complete the genesis definition.
    let expire_after = 42;
    let genesis = AssetGenesis {
        classic_migration_tickers: tickers.clone(),
        ticker_registration_config: default_reg_config(),
        classic_migration_tconfig: TickerRegistrationConfig {
            registration_length: Some(expire_after),
            ..default_reg_config()
        },
        classic_migration_contract_did: 0.into(),
        reserved_country_currency_codes: vec![],
        versions: vec![],
    };

    // Define the fees and initial balance.
    let init_bal = 150;
    let fee = 50;
    let fees = MockProtocolBaseFees(vec![(ProtocolOp::AssetCreateAsset, fee)]);

    let ext = with_asset_genesis(genesis).set_protocol_base_fees(fees);
    ext.build().execute_with(move || {
        System::set_block_number(1);

        let focus_user = |key: AccountKeyring, bal| {
            let user = User::new(key).balance(bal);
            TestStorage::set_payer_context(Some(user.acc()));
            user
        };

        // Initialize Alice and let them claim classic tickers successfully.
        let alice = focus_user(AccountKeyring::Alice, init_bal);
        let eth_sig = ethereum::eth_msg(alice.did, b"classic_claim", &alice_secret_key());
        for ClassicTickerImport { ticker, .. } in tickers {
            assert_ok!(Asset::claim_classic_ticker(
                alice.origin(),
                ticker,
                eth_sig.clone()
            ));
            assert_eq!(alice.did, Tickers::<TestStorage>::get(ticker).owner);
            assert!(matches!(
                &*System::events(),
                [
                    ..,
                    frame_system::EventRecord {
                        event: super::storage::EventTest::pallet_asset(
                            pallet_asset::RawEvent::ClassicTickerClaimed(..)
                        ),
                        ..
                    }
                ]
            ));
        }

        // Create `ALPHA` asset; this will cost.
        let create = |user: User, name: &str, bal_after| {
            let asset = name.try_into().unwrap();
            let ticker = ticker(name);
            let ret = Asset::base_create_asset_and_mint(
                user.origin(),
                asset,
                ticker,
                1,
                true,
                <_>::default(),
                vec![],
                None,
            );
            assert_balance(user.acc(), bal_after, 0);
            ret
        };
        assert_ok!(create(alice, "ALPHA", init_bal - fee));

        // Create `BETA`; fee is waived as `is_created: true`.
        assert_ok!(create(alice, "BETA", init_bal - fee));

        // Fast forward, thereby expiring `GAMMA` for which `is_created: true` holds.
        // Thus, fee isn't waived and is charged.
        Timestamp::set_timestamp(expire_after + 1);
        assert_ok!(create(alice, "GAMMA", init_bal - fee - fee));

        // Now `DELTA` has expired as well. Bob registers it, so its not classic anymore and fee is charged.
        let bob = focus_user(AccountKeyring::Bob, 0);
        assert!(ClassicTickers::get(&ticker("DELTA")).is_some());
        assert_ok!(Asset::register_ticker(bob.origin(), ticker("DELTA")));
        assert_eq!(ClassicTickers::get(&ticker("DELTA")), None);
        assert_noop!(
            create(bob, "DELTA", 0),
            FeeError::InsufficientAccountBalance
        );

        // Repeat for `EPSILON`, but directly `create_asset` instead.
        let charlie = focus_user(AccountKeyring::Charlie, 2 * fee);
        assert!(ClassicTickers::get(&ticker("EPSILON")).is_some());
        assert_ok!(create(charlie, "EPSILON", 1 * fee));
        assert_eq!(ClassicTickers::get(&ticker("EPSILON")), None);

        // Travel back in time to unexpire `ZETA`,
        // transfer it to Charlie, and ensure its not classic anymore.
        Timestamp::set_timestamp(0);
        let zeta = ticker("ZETA");
        assert!(ClassicTickers::get(&zeta).is_some());
        let auth_id_alice = Identity::add_auth(
            alice.did,
            Signatory::from(charlie.did),
            AuthorizationData::TransferTicker(zeta),
            None,
        );
        assert_ok!(Asset::accept_ticker_transfer(
            charlie.origin(),
            auth_id_alice
        ));
        assert_eq!(ClassicTickers::get(&zeta), None);
        assert_ok!(create(charlie, "ZETA", 0 * fee));
        assert_eq!(ClassicTickers::get(&zeta), None);
    });
}

fn generate_uid(entity_name: String) -> InvestorUid {
    InvestorUid::from(format!("uid_{}", entity_name).as_bytes())
}

// Test for the validating the code for unique investors and aggregation of balances.
#[test]
fn check_unique_investor_count() {
    let cdd_provider = AccountKeyring::Charlie.to_account_id();
    ExtBuilder::default()
        .set_max_tms_allowed(5)
        .cdd_providers(vec![cdd_provider.clone()])
        .build()
        .execute_with(|| {
            let user = |key: AccountKeyring| {
                let _ = make_account_without_cdd(key.to_account_id()).unwrap();
                User::existing(key)
            };
            let alice = user(AccountKeyring::Alice);
            // Bob entity as investor in given ticker.
            let bob_1 = user(AccountKeyring::Bob);
            // Eve also comes under the `Bob` entity.
            let bob_2 = user(AccountKeyring::Eve);

            let total_supply = 1_000_000_000;

            let (ticker, mut token) = a_token(alice.did);
            token.total_supply = total_supply;
            assert_ok!(basic_asset(alice, ticker, &token));

            // Verify the asset creation
            assert_eq!(Asset::token_details(&ticker), token);

            // Verify the balance of the alice and the investor count for the asset.
            assert_eq!(Asset::balance_of(&ticker, alice.did), total_supply); // It should be equal to total supply.
                                                                             // Alice act as the unique investor but not on the basis of ScopeId as alice doesn't posses the claim yet.
            assert_eq!(Statistics::investor_count(&ticker), 1);
            assert!(!ScopeIdOf::contains_key(&ticker, alice.did));

            // 1. Transfer some funds to bob_1.did.

            // 1a). Add empty compliance requirement.
            allow_all_transfers(ticker, alice);

            // 1b). Should fail when transferring funds to bob_1.did because it doesn't posses scope_claim.
            // portfolio Id -
            assert_noop!(
                transfer(ticker, alice, bob_1, 1000),
                AssetError::InvalidTransfer
            );

            // 1c). Provide the valid scope claim.
            // Create Investor unique Id, In an ideal scenario it will be generated from the PUIS system.
            let bob_uid = generate_uid("BOB_ENTITY".to_string());
            provide_scope_claim(bob_1.did, ticker, bob_uid, cdd_provider.clone(), None).0;

            let alice_uid = generate_uid("ALICE_ENTITY".to_string());
            provide_scope_claim(alice.did, ticker, alice_uid, cdd_provider.clone(), None).0;
            let alice_scope_id = Asset::scope_id_of(&ticker, &alice.did);

            // 1d). Validate the storage changes.
            let bob_scope_id = Asset::scope_id_of(&ticker, &bob_1.did);
            assert_eq!(Asset::aggregate_balance_of(&ticker, bob_scope_id), 0);
            assert_eq!(Asset::balance_of_at_scope(bob_scope_id, bob_1.did), 0);

            assert_eq!(
                Asset::aggregate_balance_of(&ticker, alice_scope_id),
                total_supply
            );
            assert_eq!(
                Asset::balance_of_at_scope(alice_scope_id, alice.did),
                total_supply
            );

            // 1e). successfully transfer funds.
            assert_ok!(transfer(ticker, alice, bob_1, 1000));

            // validate the storage changes for Bob.
            assert_eq!(Asset::aggregate_balance_of(&ticker, &bob_scope_id), 1000);
            assert_eq!(Asset::balance_of_at_scope(&bob_scope_id, &bob_1.did), 1000);
            assert_eq!(Asset::balance_of(&ticker, &bob_1.did), 1000);
            assert_eq!(Statistics::investor_count(&ticker), 2);

            // validate the storage changes for Alice.
            assert_eq!(
                Asset::aggregate_balance_of(&ticker, &alice_scope_id),
                total_supply - 1000
            );
            assert_eq!(
                Asset::balance_of_at_scope(&alice_scope_id, &alice.did),
                total_supply - 1000
            );
            assert_eq!(Asset::balance_of(&ticker, &alice.did), total_supply - 1000);
            assert_eq!(Statistics::investor_count(&ticker), 2);

            // Provide scope claim to bob_2.did
            provide_scope_claim(bob_2.did, ticker, bob_uid, cdd_provider, None).0;

            // 1f). successfully transfer funds.
            assert_ok!(transfer(ticker, alice, bob_2, 1000));

            // validate the storage changes for Bob.
            assert_eq!(Asset::aggregate_balance_of(&ticker, &bob_scope_id), 2000);
            assert_eq!(Asset::balance_of_at_scope(&bob_scope_id, &bob_2.did), 1000);
            assert_eq!(Asset::balance_of(&ticker, &bob_2.did), 1000);
            assert_eq!(Statistics::investor_count(&ticker), 2);
        });
}

#[test]
fn next_checkpoint_is_updated() {
    ExtBuilder::default()
        .build()
        .execute_with(next_checkpoint_is_updated_we);
}

fn next_checkpoint_is_updated_we() {
    // 14 November 2023, 22:13 UTC (millisecs)
    let start: u64 = 1_700_000_000_000;
    // A period of 42 hours.
    let period = CalendarPeriod {
        unit: CalendarUnit::Hour,
        amount: 42,
    };
    // The increment in seconds.
    let period_secs = match period.to_recurring().unwrap().as_fixed_or_variable() {
        FixedOrVariableCalendarUnit::Fixed(secs) => secs,
        _ => panic!("period should be fixed"),
    };
    let period_ms = period_secs * 1000;
    Timestamp::set_timestamp(start);
    assert_eq!(start, <TestStorage as asset::Config>::UnixTime::now());

    let owner = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    let (ticker, token) = a_token(owner.did);
    assert_ok!(basic_asset(owner, ticker, &token));

    assert_eq!(Checkpoint::schedules(ticker), Vec::new());
    let schedule = ScheduleSpec {
        start: Some(start),
        period,
        remaining: 0,
    };
    assert_ok!(Checkpoint::set_schedules_max_complexity(
        root(),
        period.complexity()
    ));
    assert_ok!(Checkpoint::create_schedule(
        owner.origin(),
        ticker,
        schedule
    ));
    let id = CheckpointId(1);
    assert_eq!(id, Checkpoint::checkpoint_id_sequence(&ticker));
    assert_eq!(start, Checkpoint::timestamps(ticker, id));
    let total_supply = token.total_supply;
    assert_eq!(total_supply, Checkpoint::total_supply_at(ticker, id));
    assert_eq!(total_supply, Asset::get_balance_at(ticker, owner.did, id));
    assert_eq!(0, Asset::get_balance_at(ticker, bob.did, id));
    let checkpoint2 = start + period_ms;
    assert_eq!(vec![Some(checkpoint2)], next_checkpoints(ticker, start),);
    assert_eq!(vec![checkpoint2], checkpoint_ats(ticker));

    let transfer = |at| {
        Timestamp::set_timestamp(at);
        default_transfer(owner, bob, ticker, total_supply / 2);
    };

    // Make a transaction before the next timestamp.
    transfer(checkpoint2 - 1000);
    // Make another transaction at the checkpoint.
    // The updates are applied after the checkpoint is recorded.
    // After this transfer Alice's balance is 0.
    transfer(checkpoint2);
    // The balance after checkpoint 2.
    assert_eq!(0, Asset::balance_of(&ticker, owner.did));
    // Balances at checkpoint 2.
    let id = CheckpointId(2);
    assert_eq!(vec![start + 2 * period_ms], checkpoint_ats(ticker));
    assert_eq!(id, Checkpoint::checkpoint_id_sequence(&ticker));
    assert_eq!(start + period_ms, Checkpoint::timestamps(ticker, id));
    assert_eq!(
        total_supply / 2,
        Asset::get_balance_at(ticker, owner.did, id)
    );
    assert_eq!(total_supply / 2, Asset::get_balance_at(ticker, bob.did, id));
}

#[test]
fn non_recurring_schedule_works() {
    ExtBuilder::default()
        .build()
        .execute_with(non_recurring_schedule_works_we);
}

fn non_recurring_schedule_works_we() {
    // 14 November 2023, 22:13 UTC (millisecs)
    let start: u64 = 1_700_000_000_000;
    // Non-recuring schedule.
    let period = CalendarPeriod::default();
    Timestamp::set_timestamp(start);
    assert_eq!(start, <TestStorage as asset::Config>::UnixTime::now());

    let owner = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    let (ticker, token) = a_token(owner.did);
    assert_ok!(basic_asset(owner, ticker, &token));

    assert_eq!(Checkpoint::schedules(ticker), Vec::new());
    let schedule = ScheduleSpec {
        start: Some(start),
        period,
        remaining: 0,
    };
    assert_ok!(Checkpoint::set_schedules_max_complexity(
        root(),
        period.complexity()
    ));
    assert_ok!(Checkpoint::create_schedule(
        owner.origin(),
        ticker,
        schedule
    ));
    let id = CheckpointId(1);
    assert_eq!(id, Checkpoint::checkpoint_id_sequence(&ticker));
    assert_eq!(start, Checkpoint::timestamps(ticker, id));
    let total_supply = token.total_supply;
    assert_eq!(total_supply, Checkpoint::total_supply_at(ticker, id));
    assert_eq!(total_supply, Asset::get_balance_at(ticker, owner.did, id));
    assert_eq!(0, Asset::get_balance_at(ticker, bob.did, id));
    // The schedule will not recur.
    assert_eq!(Checkpoint::schedules(ticker), Vec::new());
}

fn checkpoint_ats(ticker: Ticker) -> Vec<u64> {
    Checkpoint::schedules(ticker)
        .into_iter()
        .map(|s| s.at)
        .collect()
}

fn next_checkpoints(ticker: Ticker, start: u64) -> Vec<Option<u64>> {
    Checkpoint::schedules(ticker)
        .into_iter()
        .map(|s| s.schedule.next_checkpoint(start))
        .collect()
}

#[test]
fn schedule_remaining_works() {
    ExtBuilder::default().build().execute_with(|| {
        let start = 1_000;
        Timestamp::set_timestamp(start);

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create the asset.
        let (ticker, token) = a_token(owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        let transfer = |at: Moment| {
            Timestamp::set_timestamp(at * 1_000);
            default_transfer(owner, bob, ticker, 1);
        };
        let collect_ts = |sh_id| {
            Checkpoint::schedule_points(ticker, sh_id)
                .into_iter()
                .map(|cp| Checkpoint::timestamps(ticker, cp))
                .collect::<Vec<_>>()
        };

        // No schedules yet.
        assert_eq!(Checkpoint::schedules(ticker), vec![]);

        // For simplicity, we use 1s = 1_000ms periods.
        let period = CalendarPeriod {
            unit: CalendarUnit::Second,
            amount: 1,
        };

        // Allow such a schedule to be added on-chain. Otherwise, we'll have errors.
        assert_ok!(Checkpoint::set_schedules_max_complexity(
            root(),
            period.complexity()
        ));

        // Create a schedule with one remaining and where `start == now`.
        let mut spec = ScheduleSpec {
            start: Some(start),
            period,
            remaining: 1,
        };
        let schedule = CheckpointSchedule { start, period };
        assert_ok!(Checkpoint::create_schedule(owner.origin(), ticker, spec));

        // We had `remaining == 1` and `start == now`,
        // so since a CP was created, hence `remaining => 0`,
        // the schedule was immediately evicted.
        assert_eq!(Checkpoint::schedules(ticker), vec![]);
        assert_eq!(collect_ts(ScheduleId(1)), vec![start]);

        // This time, we set `remaining == 5`, but we still have `start == now`,
        // thus one CP is immediately created, so `remaining => 4`.
        spec.remaining = 5;
        let id2 = ScheduleId(2);
        let assert_ts = |ticks| {
            assert_eq!(
                collect_ts(id2),
                (1..=ticks).map(|x| x * 1_000).collect::<Vec<_>>()
            );
        };
        let assert_sh = |at: Moment, remaining| {
            assert_eq!(
                Checkpoint::schedules(ticker),
                vec![StoredSchedule {
                    id: id2,
                    schedule,
                    at: 1_000 * at,
                    remaining,
                }]
            );
        };
        assert_ok!(Checkpoint::create_schedule(owner.origin(), ticker, spec));
        assert_sh(2, 4);
        assert_ts(1);

        // Transfer and move through the 2nd to 4th recurrences.
        for i in 2..5 {
            transfer(i);
            assert_sh(i + 1, spec.remaining - i as u32);
            assert_ts(i);
        }

        // Transfer and move to the 5th (last) recurrence.
        // We've to the point where there are no ticks left.
        transfer(5);
        assert_eq!(Checkpoint::schedules(ticker), vec![]);
        assert_ts(5);
    });
}

#[test]
fn mesh_1531_ts_collission_regression_test() {
    ExtBuilder::default().build().execute_with(|| {
        // Create the assets.
        let owner = User::new(AccountKeyring::Alice);
        let asset = |name: &[u8]| {
            let (ticker, token) = token(name, owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));
            ticker
        };
        let alpha = asset(b"ALPHA");
        let beta = asset(b"BETA");

        // First CP is made at 1s.
        let cp = CheckpointId(1);
        Timestamp::set_timestamp(1_000);
        let create = |ticker| Checkpoint::create_checkpoint(owner.origin(), ticker);
        assert_ok!(create(alpha));
        assert_eq!(Checkpoint::timestamps(alpha, cp), 1_000);

        // Second CP is for beta, using same ID.
        Timestamp::set_timestamp(2_000);
        assert_ok!(create(beta));
        assert_eq!(Checkpoint::timestamps(alpha, cp), 1_000);
        assert_eq!(Checkpoint::timestamps(beta, cp), 2_000);
    });
}

#[test]
fn secondary_key_not_authorized_for_asset_test() {
    let users @ [owner, all, not] = [
        AccountKeyring::Alice,
        AccountKeyring::Bob,
        AccountKeyring::Charlie,
    ];
    let invalid_names = [b"WPUSD1\0", &b"WPUSC\0\0", &b"WPUSD\01"];
    let invalid_tickers = invalid_names
        .iter()
        .filter_map(|name| Ticker::try_from(name.as_ref()).ok());

    let secondary_keys = vec![
        SecondaryKey {
            signer: Signatory::Account(not.to_account_id()),
            permissions: Permissions {
                asset: AssetPermissions::elems(invalid_tickers),
                ..Default::default()
            },
        },
        SecondaryKey {
            signer: Signatory::Account(all.to_account_id()),
            permissions: Permissions::default(),
        },
    ];

    let owner = polymesh_primitives::Identity {
        primary_key: owner.to_account_id(),
        secondary_keys,
    };

    ExtBuilder::default()
        .add_regular_users(&[owner])
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| {
            // NB `sk_not_permsissions` does not have enought asset permissions to issue `ticker`.
            let [owner, all, not] = users.map(User::existing);
            let (ticker, token) = token(b"WPUSD", owner.did);
            assert_ok!(basic_asset(owner, ticker, &token));

            let minted_value = 50_000u128.into();
            StoreCallMetadata::set_call_metadata(b"pallet_asset".into(), b"issuer".into());
            assert_noop!(
                Asset::issue(not.origin(), ticker, minted_value),
                pallet_external_agents::Error::<TestStorage>::SecondaryKeyNotAuthorizedForAsset
            );

            assert_ok!(Asset::issue(all.origin(), ticker, minted_value));
            assert_eq!(Asset::total_supply(ticker), TOTAL_SUPPLY + minted_value);
        });
}

#[test]
fn invalid_ticker_registry_test() {
    test_with_owner(|owner| {
        let (ticker, token) = token(b"MYUSD", owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        // Generate a data set for testing: (input, expected result)
        [
            (&b"MYUSD"[..], true),
            (&b"MYUSD\01"[..], false),
            (&b"YOUR"[..], false),
        ]
        .iter()
        .map(|(name, exp)| (Ticker::try_from(name.as_ref()).unwrap(), exp))
        .for_each(|(ticker, exp)| {
            let valid = Asset::is_ticker_registry_valid(&ticker, owner.did);
            assert_eq!(*exp, valid)
        });
    });
}

#[test]
fn sender_same_as_receiver_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, true);

        // Create new portfolio
        let eu_portfolio = PortfolioId::default_portfolio(owner.did);
        let uk_portfolio = new_portfolio(owner.acc(), "UK");

        // Enforce an unsafe tranfer.
        assert_noop!(
            Asset::unsafe_transfer(eu_portfolio, uk_portfolio, &ticker, 1_000),
            AssetError::SenderSameAsReceiver
        );
    });
}

#[test]
fn invalid_granularity_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, false);
        assert_noop!(
            Asset::issue(owner.origin(), ticker, 10_000),
            AssetError::InvalidGranularity
        );
    })
}

#[test]
fn create_asset_errors_test() {
    let owner = AccountKeyring::Alice.to_account_id();
    let other = AccountKeyring::Bob.to_account_id();
    let cdd = AccountKeyring::Eve.to_account_id();

    ExtBuilder::default()
        .add_regular_users_from_accounts(&[owner.clone(), other.clone()])
        .cdd_providers(vec![cdd])
        .build()
        .execute_with(|| create_asset_errors(owner, other))
}

fn bytes_of_len<R: From<Vec<u8>>>(e: u8, len: usize) -> R {
    iter::repeat(e).take(len).collect::<Vec<_>>().into()
}

fn create_asset_errors(owner: AccountId, other: AccountId) {
    let o = Origin::signed(owner);
    let o2 = Origin::signed(other);

    let ticker = Ticker::try_from(&b"MYUSD"[..]).unwrap();
    let name: AssetName = ticker.as_ref().into();
    let atype = AssetType::default();

    let name_max_length = <TestStorage as AssetConfig>::AssetNameMaxLength::get() + 1;
    let input_expected = vec![
        (
            bytes_of_len(b'A', name_max_length as usize),
            TOTAL_SUPPLY,
            true,
            None,
            AssetError::MaxLengthOfAssetNameExceeded,
        ),
        (
            name.clone(),
            TOTAL_SUPPLY,
            true,
            Some(exceeded_funding_round_name()),
            AssetError::FundingRoundNameMaxLengthExceeded,
        ),
        (
            name.clone(),
            1_000,
            false,
            None,
            AssetError::InvalidGranularity,
        ),
        (
            name.clone(),
            u128::MAX,
            true,
            None,
            AssetError::TotalSupplyAboveLimit,
        ),
    ];

    for (name, total_supply, is_divisible, funding_name, expected_err) in input_expected.into_iter()
    {
        assert_noop!(
            Asset::base_create_asset_and_mint(
                o.clone(),
                name,
                ticker,
                total_supply,
                is_divisible,
                atype.clone(),
                vec![],
                funding_name
            ),
            expected_err
        );
    }

    assert_ok!(Asset::register_ticker(o2.clone(), ticker));
    assert_noop!(
        Asset::base_create_asset_and_mint(
            o.clone(),
            name.clone(),
            ticker,
            TOTAL_SUPPLY,
            true,
            atype.clone(),
            vec![],
            None
        ),
        AssetError::TickerAlreadyRegistered
    );
}

#[test]
fn asset_type_custom_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let (ticker, mut token) = a_token(owner.did);
        let mut case = |add| {
            token.asset_type = AssetType::Custom(max_len_bytes(add));
            basic_asset(owner, ticker, &token)
        };
        assert_too_long!(case(1));
        assert_ok!(case(0));
    });
}

#[test]
fn asset_doc_field_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let ticker = an_asset(owner, true);
        let add_doc = |doc| Asset::add_documents(owner.origin(), vec![doc], ticker);
        assert_too_long!(add_doc(Document {
            uri: max_len_bytes(1),
            ..<_>::default()
        }));
        assert_too_long!(add_doc(Document {
            name: max_len_bytes(1),
            ..<_>::default()
        }));
        assert_too_long!(add_doc(Document {
            doc_type: Some(max_len_bytes(1)),
            ..<_>::default()
        }));
        assert_ok!(add_doc(Document {
            uri: max_len_bytes(0),
            name: max_len_bytes(0),
            doc_type: Some(max_len_bytes(0)),
            ..<_>::default()
        }));
    });
}

#[track_caller]
fn test_with_owner(test: impl FnOnce(User)) {
    let owner = AccountKeyring::Alice;
    ExtBuilder::default()
        .add_regular_users_from_accounts(&[owner.to_account_id()])
        .cdd_providers(vec![AccountKeyring::Eve.to_account_id()])
        .build()
        .execute_with(|| test(User::existing(owner)));
}

#[test]
fn unsafe_can_transfer_all_status_codes_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, false);

        let uk_portfolio = new_portfolio(owner.acc(), "UK");
        let default_portfolio = PortfolioId::default_portfolio(owner.did);
        let do_unsafe_can_transfer = || {
            Asset::unsafe_can_transfer(None, default_portfolio, None, uk_portfolio, &ticker, 100)
                .unwrap()
        };

        // INVALID_GRANULARITY
        let code = do_unsafe_can_transfer();
        assert_eq!(code, INVALID_GRANULARITY);

        // Update indivisible.
        assert_ok!(Asset::make_divisible(owner.origin(), ticker));

        // INVALID_RECEIVER_DID
        let code = do_unsafe_can_transfer();
        assert_eq!(code, INVALID_RECEIVER_DID);

        // INVALID_SENDER_DID
        let no_cdd_portfolio_did = PortfolioId::default();
        let code = Asset::unsafe_can_transfer(
            None,
            no_cdd_portfolio_did,
            None,
            default_portfolio,
            &ticker,
            100,
        )
        .unwrap();
        assert_eq!(code, INVALID_SENDER_DID);
    });
}

#[test]
fn set_funding_round_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, true);
        assert_noop!(
            Asset::set_funding_round(owner.origin(), ticker, exceeded_funding_round_name()),
            AssetError::FundingRoundNameMaxLengthExceeded
        );
        assert_ok!(Asset::set_funding_round(
            owner.origin(),
            ticker,
            FundingRoundName(b"VIP round".to_vec())
        ));
    })
}

#[test]
fn update_identifiers_errors_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, true);
        let invalid_asset_ids = vec![
            AssetIdentifier::CUSIP(*b"037833108"),   // Invalid checksum.
            AssetIdentifier::CINS(*b"S08000AA7"),    // Invalid checksum.
            AssetIdentifier::ISIN(*b"US0378331004"), // Invalid checksum.
            AssetIdentifier::LEI(*b"549300GFX6WN7JDUSN37"), // Invalid checksum.
        ];

        invalid_asset_ids.into_iter().for_each(|asset_id| {
            assert_noop!(
                Asset::update_identifiers(owner.origin(), ticker, vec![asset_id]),
                AssetError::InvalidAssetIdentifier
            );
        });

        let valid_asset_ids = vec![AssetIdentifier::CUSIP(*b"037833100")];
        assert_ok!(Asset::update_identifiers(
            owner.origin(),
            ticker,
            valid_asset_ids
        ));
    })
}
