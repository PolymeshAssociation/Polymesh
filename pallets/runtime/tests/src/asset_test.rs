use chrono::prelude::Utc;
use frame_support::dispatch::{DispatchError, DispatchResult};
use frame_support::{assert_noop, assert_ok};
use frame_support::{IterableStorageDoubleMap, StorageDoubleMap, StorageMap, StorageValue};
use hex_literal::hex;
use ink_primitives::hash as FunctionSelectorHasher;
use rand::Rng;
use sp_consensus_babe::Slot;
use sp_io::hashing::keccak_256;
use sp_runtime::AnySignature;
use sp_std::convert::{From, TryFrom, TryInto};
use sp_std::iter;

use pallet_asset::{
    AssetDocuments, AssetMetadataLocalKeyToName, AssetMetadataLocalNameToKey,
    AssetMetadataLocalSpecs, AssetMetadataValues, AssetOwnershipRelation, BalanceOf,
    Config as AssetConfig, CustomTypeIdSequence, CustomTypes, CustomTypesInverse,
    PreApprovedTicker, ScopeIdOf, SecurityToken, TickerRegistrationConfig,
    TickersExemptFromAffirmation,
};
use pallet_portfolio::{NextPortfolioNumber, PortfolioAssetBalances};
use polymesh_common_utilities::asset::AssetFnTrait;
use polymesh_common_utilities::constants::currency::ONE_UNIT;
use polymesh_common_utilities::constants::*;
use polymesh_common_utilities::traits::checkpoint::{
    NextCheckpoints, ScheduleCheckpoints, ScheduleId,
};
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::asset::{
    AssetName, AssetType, CheckpointId, CustomAssetTypeId, FundingRoundName, NonFungibleType,
};
use polymesh_primitives::asset_metadata::{
    AssetMetadataKey, AssetMetadataLocalKey, AssetMetadataLockStatus, AssetMetadataName,
    AssetMetadataSpec, AssetMetadataValue, AssetMetadataValueDetail,
};
use polymesh_primitives::calendar::{CalendarPeriod, CalendarUnit, FixedOrVariableCalendarUnit};
use polymesh_primitives::statistics::StatType;
use polymesh_primitives::{
    AccountId, AssetIdentifier, AssetPermissions, AuthorizationData, AuthorizationError, Document,
    DocumentId, Fund, FundDescription, IdentityId, InvestorUid, Memo, Moment, NFTCollectionKeys,
    Permissions, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, SecondaryKey,
    Signatory, Ticker, WeightMeter,
};
use test_client::AccountKeyring;

use crate::ext_builder::{ExtBuilder, IdentityRecord};
use crate::nft::create_nft_collection;
use crate::storage::{
    add_secondary_key, make_account_without_cdd, provide_scope_claim,
    provide_scope_claim_to_multiple_parties, register_keyring_account, root, Checkpoint,
    TestStorage, User,
};

type BaseError = pallet_base::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type Asset = pallet_asset::Module<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type OffChainSignature = AnySignature;
type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;
type DidRecords = pallet_identity::DidRecords<TestStorage>;
type Statistics = pallet_statistics::Module<TestStorage>;
type AssetGenesis = pallet_asset::GenesisConfig<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type FeeError = pallet_protocol_fee::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type StoreCallMetadata = pallet_permissions::StoreCallMetadata<TestStorage>;

fn now() -> u64 {
    Utc::now().timestamp() as _
}

fn set_time_to_now() {
    set_timestamp(now());
}

fn slot_duration() -> u64 {
    pallet_babe::Pallet::<TestStorage>::slot_duration()
}

pub(crate) fn set_timestamp(n: u64) {
    pallet_babe::CurrentSlot::<TestStorage>::set(Slot::from(n / slot_duration()));
    Timestamp::set_timestamp(n);
}

pub(crate) fn max_len() -> u32 {
    <TestStorage as pallet_base::Config>::MaxLen::get()
}

pub(crate) fn max_len_bytes<R: From<Vec<u8>>>(add: u32) -> R {
    bytes_of_len(b'A', (max_len() + add) as usize)
}

macro_rules! assert_too_long {
    ($e:expr) => {
        let e_result = $e;
        assert_noop!(e_result, pallet_base::Error::<TestStorage>::TooLong);
    };
}

pub(crate) fn token_details(ticker: &Ticker) -> SecurityToken {
    Asset::token_details(ticker).unwrap_or_default()
}

pub(crate) fn token(name: &[u8], owner_did: IdentityId) -> (Ticker, SecurityToken) {
    let ticker = Ticker::from_slice_truncated(name);
    let token = SecurityToken {
        owner_did,
        total_supply: TOTAL_SUPPLY,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    (ticker, token)
}

pub(crate) fn a_token(owner_did: IdentityId) -> (Ticker, SecurityToken) {
    token(b"A", owner_did)
}

pub(crate) fn an_asset(owner: User, divisible: bool) -> Ticker {
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
    Asset::create_asset(
        owner.origin(),
        ticker.as_ref().into(),
        ticker,
        token.divisible,
        token.asset_type.clone(),
        ids,
        None,
    )?;
    enable_investor_count(ticker, owner);
    Asset::issue(
        owner.origin(),
        ticker,
        token.total_supply,
        PortfolioKind::Default,
    )?;
    assert_eq!(Asset::balance_of(&ticker, owner.did), token.total_supply);
    Ok(())
}

pub(crate) fn basic_asset(owner: User, ticker: Ticker, token: &SecurityToken) -> DispatchResult {
    asset_with_ids(owner, ticker, token, vec![])
}

pub(crate) fn create_token(owner: User) -> (Ticker, SecurityToken) {
    let r = a_token(owner.did);
    assert_ok!(basic_asset(owner, r.0, &r.1));
    r
}

pub(crate) fn allow_all_transfers(ticker: Ticker, owner: User) {
    assert_ok!(ComplianceManager::add_compliance_requirement(
        owner.origin(),
        ticker,
        vec![],
        vec![]
    ));
}

fn enable_investor_count(ticker: Ticker, owner: User) {
    assert_ok!(Statistics::set_active_asset_stats(
        owner.origin(),
        ticker.into(),
        [StatType::investor_count()].iter().cloned().collect(),
    ));
}

pub(crate) fn transfer(ticker: Ticker, from: User, to: User, amount: u128) -> DispatchResult {
    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    Asset::base_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        &ticker,
        amount,
        None,
        None,
        IdentityId::default(),
        &mut weight_meter,
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
    let did = Identity::get_identity(&owner).expect("User missing identity");

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

pub fn next_schedule_id(ticker: Ticker) -> ScheduleId {
    let ScheduleId(id) = Checkpoint::schedule_id_sequence(ticker);
    ScheduleId(id + 1)
}

#[track_caller]
pub fn check_schedules(ticker: Ticker, schedules: &[(ScheduleId, ScheduleCheckpoints)]) {
    let mut cached = NextCheckpoints::default();
    for (id, schedule) in schedules {
        assert_eq!(
            Checkpoint::scheduled_checkpoints(ticker, id).as_ref(),
            Some(schedule)
        );
        cached.add_schedule_next(*id, schedule.next().unwrap());
        cached.inc_total_pending(schedule.len() as u64);
    }
    if cached.is_empty() {
        assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
    } else {
        assert_eq!(Checkpoint::cached_next_checkpoints(ticker), Some(cached));
    }
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

        // Create asset.
        let funding_round_name: FundingRoundName = b"round1".into();
        assert_ok!(Asset::create_asset(
            owner.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            token.asset_type.clone(),
            Vec::new(),
            Some(funding_round_name.clone()),
        ));
        enable_investor_count(ticker, owner);

        let issue = |supply| Asset::issue(owner.origin(), ticker, supply, PortfolioKind::Default);
        assert_noop!(
            issue(1_000_000_000_000_000_000_000_000),
            AssetError::TotalSupplyAboveLimit
        );

        // Sucessfully issue. Investor count should now be 1.
        assert_ok!(issue(token.total_supply));
        assert_eq!(Statistics::investor_count(ticker), 1);

        // A correct entry is added
        assert_eq!(token_details(&ticker), token);
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
        assert_eq!(token_details(&ticker), token);
        // Rename the token and check storage has been updated.
        let new: AssetName = [0x42].into();
        assert_ok!(Asset::rename_asset(owner.origin(), ticker, new.clone()));
        assert_eq!(Asset::asset_names(ticker), Some(new));
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
            assert_eq!(token_details(&ticker).total_supply, 0);

            assert_noop!(
                Asset::redeem(owner.origin(), ticker, 1),
                PortfolioError::InsufficientPortfolioBalance
            );
        })
}

fn default_transfer(from: User, to: User, ticker: Ticker, val: u128) {
    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    assert_ok!(Asset::unsafe_transfer(
        PortfolioId::default_portfolio(from.did),
        PortfolioId::default_portfolio(to.did),
        &ticker,
        val,
        None,
        None,
        IdentityId::default(),
        &mut weight_meter
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
        let stored_token = token_details(&ticker);
        assert_eq!(stored_token.asset_type, token.asset_type);
        assert_eq!(Asset::identifiers(ticker), identifiers);
        assert_noop!(
            register(Ticker::from_slice_truncated(&[b'A'][..])),
            AssetError::AssetAlreadyCreated
        );

        assert_noop!(
            register(Ticker::from_slice_truncated(
                &[b'A', b'A', b'A', b'A', b'A', b'A', b'A', b'A', b'A'][..]
            )),
            AssetError::TickerTooLong
        );

        let ticker = Ticker::from_slice_truncated(&[b'A', b'A'][..]);

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

        set_timestamp(now() + 10001);

        assert_eq!(Asset::is_ticker_registry_valid(&ticker, owner.did), false);
        assert_eq!(Asset::is_ticker_available(&ticker), true);

        for bs in &[
            [b'A', 31, b'B'].as_ref(),
            [127, b'A'].as_ref(),
            [b'A', 0, 0, 0, b'A'].as_ref(),
        ] {
            assert_noop!(
                register(Ticker::from_slice_truncated(&bs[..])),
                AssetError::TickerNotAlphanumeric
            );
        }
    })
}

#[test]
fn transfer_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        let ticker = Ticker::from_slice_truncated(&[b'A', b'A'][..]);

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

        assert_eq!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id_bob),
            Err("Illegal use of Authorization".into()),
        );

        let add_auth = |auth, expiry| {
            Identity::add_auth(alice.did, Signatory::from(bob.did), auth, Some(expiry))
        };

        let auth_id = add_auth(AuthorizationData::TransferTicker(ticker), now() - 100);

        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id),
            "Authorization expired"
        );

        // Try accepting the wrong authorization type.
        let auth_id = add_auth(AuthorizationData::RotatePrimaryKey, now() + 100);

        assert_eq!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id),
            Err(AuthorizationError::BadType.into()),
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

        assert_eq!(token_details(&ticker).owner_did, owner.did);

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
        assert_eq!(token_details(&ticker).owner_did, alice.did);
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
        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id_bob),
            Err(EAError::UnauthorizedAgent.into())
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

        // Try accepting the wrong authorization type.
        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::RotatePrimaryKey,
            Some(now() + 100),
        );

        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            Err(AuthorizationError::BadType.into())
        );

        auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(Ticker::from_slice_truncated(&[0x50][..])),
            Some(now() + 100),
        );

        assert_eq!(
            Asset::accept_asset_ownership_transfer(bob.origin(), auth_id),
            Err(AssetError::NoSuchAsset.into())
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
        assert_eq!(token_details(&ticker).owner_did, bob.did);
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
        assert_eq!(token_details(&ticker), token);
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
            assert_eq!(
                Some(doc),
                Asset::asset_documents(ticker, DocumentId(idx as u32))
            );
        }

        assert_ok!(Asset::remove_documents(
            owner.origin(),
            (0..=1).map(DocumentId).collect(),
            ticker
        ));

        assert_eq!(AssetDocuments::iter_prefix_values(ticker).count(), 0);
    });
}

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
    let alice = User::new(AccountKeyring::Alice);
    let bob = User::new_with(alice.did, AccountKeyring::Bob);
    let _charlie_id = register_keyring_account(AccountKeyring::Charlie).unwrap();

    // 1. Add Bob as signatory to Alice ID.
    add_secondary_key(alice.did, bob.acc());

    assert_ok!(Balances::transfer_with_memo(
        alice.origin(),
        bob.acc().into(),
        1_000,
        Some(Memo::from("Bob funding"))
    ));

    // 2. Bob can create token
    let (ticker_1, token_1) = a_token(alice.did);
    assert_ok!(Asset::create_asset(
        bob.origin(),
        ticker_1.as_ref().into(),
        ticker_1,
        true,
        token_1.asset_type.clone(),
        vec![],
        None,
    ));
    assert_ok!(Asset::issue(
        bob.origin(),
        ticker_1,
        token_1.total_supply,
        PortfolioKind::Default
    ));
    assert_eq!(token_details(&ticker_1), token_1);

    // 3. Alice freezes her secondary keys.
    assert_ok!(Identity::freeze_secondary_keys(alice.origin()));

    // 4. Bob cannot create a token.
    let (_ticker_2, _token_2) = a_token(alice.did);
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
                    &mut WeightMeter::max_limit_no_minimum(),
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

fn ticker(name: &str) -> Ticker {
    Ticker::from_slice_truncated(name.as_bytes())
}

fn default_reg_config() -> TickerRegistrationConfig<u64> {
    TickerRegistrationConfig {
        max_ticker_length: 8,
        registration_length: Some(10000),
    }
}

fn alice_secret_key() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Alice")).unwrap()
}

fn bob_secret_key() -> libsecp256k1::SecretKey {
    libsecp256k1::SecretKey::parse(&keccak_256(b"Bob")).unwrap()
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

fn generate_uid(entity_name: String) -> InvestorUid {
    InvestorUid::from(format!("uid_{}", entity_name).as_bytes())
}

// Test for the validating the code for unique investors and aggregation of balances.
#[ignore]
#[test]
fn check_unique_investor_count() {
    let cdd_provider = AccountKeyring::Charlie.to_account_id();
    ExtBuilder::default()
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
            assert_eq!(token_details(&ticker), token);

            // Verify the balance of the alice and the investor count for the asset.
            assert_eq!(Asset::balance_of(&ticker, alice.did), total_supply); // It should be equal to total supply.
                                                                             // Alice act as the unique investor but not on the basis of ScopeId as alice doesn't posses the claim yet.
            assert_eq!(Statistics::investor_count(ticker), 1);
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
            assert_eq!(Statistics::investor_count(ticker), 2);

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
            assert_eq!(Statistics::investor_count(ticker), 2);

            // Provide scope claim to bob_2.did
            provide_scope_claim(bob_2.did, ticker, bob_uid, cdd_provider, None).0;

            // 1f). successfully transfer funds.
            assert_ok!(transfer(ticker, alice, bob_2, 1000));

            // validate the storage changes for Bob.
            assert_eq!(Asset::aggregate_balance_of(&ticker, &bob_scope_id), 2000);
            assert_eq!(Asset::balance_of_at_scope(&bob_scope_id, &bob_2.did), 1000);
            assert_eq!(Asset::balance_of(&ticker, &bob_2.did), 1000);
            assert_eq!(Statistics::investor_count(ticker), 2);
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
    set_timestamp(start);
    assert_eq!(start, <TestStorage as AssetConfig>::UnixTime::now());

    let owner = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    let (ticker, token) = a_token(owner.did);
    assert_ok!(basic_asset(owner, ticker, &token));

    assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
    let schedule = ScheduleCheckpoints::from_period(start, period, 5);
    assert_ok!(Checkpoint::set_schedules_max_complexity(
        root(),
        period.complexity()
    ));
    assert_ok!(Checkpoint::create_schedule(
        owner.origin(),
        ticker,
        schedule
    ));
    assert_ok!(Checkpoint::advance_update_balances(&ticker, &[]));
    let id = CheckpointId(1);
    assert_eq!(id, Checkpoint::checkpoint_id_sequence(&ticker));
    assert_eq!(start, Checkpoint::timestamps(ticker, id));
    let total_supply = token.total_supply;
    assert_eq!(total_supply, Checkpoint::total_supply_at(ticker, id));
    assert_eq!(total_supply, Asset::get_balance_at(ticker, owner.did, id));
    assert_eq!(0, Asset::get_balance_at(ticker, bob.did, id));
    let checkpoint2 = start + period_ms;
    assert_eq!(vec![Some(checkpoint2)], next_checkpoints(ticker));
    assert_eq!(vec![checkpoint2], checkpoint_ats(ticker));

    let transfer = |at| {
        set_timestamp(at);
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
    set_timestamp(start);
    assert_eq!(start, <TestStorage as AssetConfig>::UnixTime::now());

    let owner = User::new(AccountKeyring::Alice);
    let bob = User::new(AccountKeyring::Bob);

    let (ticker, token) = a_token(owner.did);
    assert_ok!(basic_asset(owner, ticker, &token));

    assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
    let schedule = ScheduleCheckpoints::from_period(start, period, 10);
    assert_ok!(Checkpoint::set_schedules_max_complexity(
        root(),
        period.complexity()
    ));
    assert_ok!(Checkpoint::create_schedule(
        owner.origin(),
        ticker,
        schedule
    ));
    assert_ok!(Checkpoint::advance_update_balances(&ticker, &[]));
    let id = CheckpointId(1);
    assert_eq!(id, Checkpoint::checkpoint_id_sequence(&ticker));
    assert_eq!(start, Checkpoint::timestamps(ticker, id));
    let total_supply = token.total_supply;
    assert_eq!(total_supply, Checkpoint::total_supply_at(ticker, id));
    assert_eq!(total_supply, Asset::get_balance_at(ticker, owner.did, id));
    assert_eq!(0, Asset::get_balance_at(ticker, bob.did, id));
    // The schedule will not recur.
    assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
}

fn checkpoint_ats(ticker: Ticker) -> Vec<u64> {
    let cached = Checkpoint::cached_next_checkpoints(ticker).unwrap_or_default();
    cached.schedules.values().copied().collect()
}

fn next_checkpoints(ticker: Ticker) -> Vec<Option<u64>> {
    let ScheduleId(id) = Checkpoint::schedule_id_sequence(ticker);
    (1..=id)
        .into_iter()
        .map(|id| Checkpoint::scheduled_checkpoints(ticker, ScheduleId(id)).and_then(|s| s.next()))
        .collect()
}

#[test]
fn schedule_remaining_works() {
    ExtBuilder::default().build().execute_with(|| {
        let start = 1_000;
        set_timestamp(start);

        let owner = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        // Create the asset.
        let (ticker, token) = a_token(owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        let transfer = |at: Moment| {
            set_timestamp(at * 1_000);
            default_transfer(owner, bob, ticker, 1);
        };
        let collect_ts = |sh_id| {
            Checkpoint::schedule_points(ticker, sh_id)
                .into_iter()
                .map(|cp| Checkpoint::timestamps(ticker, cp))
                .collect::<Vec<_>>()
        };

        // No schedules yet.
        assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);

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
        let schedule = ScheduleCheckpoints::from_period(start, period, 1);
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            ticker,
            schedule
        ));
        assert_ok!(Checkpoint::advance_update_balances(&ticker, &[]));

        // We had `remaining == 1` and `start == now`,
        // so since a CP was created, hence `remaining => 0`,
        // the schedule was immediately evicted.
        assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
        assert_eq!(collect_ts(ScheduleId(1)), vec![start]);

        // This time, we set `remaining == 5`, but we still have `start == now`,
        // thus one CP is immediately created, so `remaining => 4`.
        let schedule = ScheduleCheckpoints::from_period(start, period, 5);
        let remaining = schedule.len();
        let id2 = ScheduleId(2);
        let assert_ts = |ticks| {
            assert_eq!(
                collect_ts(id2),
                (1..=ticks).map(|x| x * 1_000).collect::<Vec<_>>()
            );
        };
        let assert_sh = |at: Moment, remaining| {
            let now = (at - 1) * 1_000;
            let mut schedule = schedule.clone();
            schedule.remove_expired(now);
            let chain = Checkpoint::scheduled_checkpoints(ticker, id2).unwrap_or_default();
            assert_eq!(chain, schedule);
            assert_eq!(chain.len(), remaining);
        };
        assert_ok!(Checkpoint::create_schedule(
            owner.origin(),
            ticker,
            schedule.clone()
        ));
        assert_ok!(Checkpoint::advance_update_balances(&ticker, &[]));
        assert_sh(2, 4);
        assert_ts(1);

        // Transfer and move through the 2nd to 4th recurrences.
        for i in 2..5 {
            transfer(i);
            assert_sh(i + 1, remaining - i as usize);
            assert_ts(i);
        }

        // Transfer and move to the 5th (last) recurrence.
        // We've to the point where there are no ticks left.
        transfer(5);
        assert_eq!(Checkpoint::cached_next_checkpoints(ticker), None);
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
        set_timestamp(1_000);
        let create = |ticker| Checkpoint::create_checkpoint(owner.origin(), ticker);
        assert_ok!(create(alpha));
        assert_eq!(Checkpoint::timestamps(alpha, cp), 1_000);

        // Second CP is for beta, using same ID.
        set_timestamp(2_000);
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
        .filter_map(|name| Some(Ticker::from_slice_truncated(name.as_ref())));

    let secondary_keys = vec![
        SecondaryKey {
            key: not.to_account_id(),
            permissions: Permissions {
                asset: AssetPermissions::elems(invalid_tickers),
                ..Default::default()
            },
        },
        SecondaryKey {
            key: all.to_account_id(),
            permissions: Permissions::default(),
        },
    ];

    let owner = IdentityRecord::new(owner.to_account_id(), secondary_keys);

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
                Asset::issue(not.origin(), ticker, minted_value, PortfolioKind::Default),
                pallet_external_agents::Error::<TestStorage>::SecondaryKeyNotAuthorizedForAsset
            );

            assert_ok!(Asset::issue(
                all.origin(),
                ticker,
                minted_value,
                PortfolioKind::Default
            ));
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
        .map(|(name, exp)| (Ticker::from_slice_truncated(*name), exp))
        .for_each(|(ticker, exp)| {
            assert_eq!(*exp, Asset::is_ticker_registry_valid(&ticker, owner.did))
        });
    });
}

#[test]
fn sender_same_as_receiver_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, true);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();

        // Create new portfolio
        let eu_portfolio = PortfolioId::default_portfolio(owner.did);
        let uk_portfolio = new_portfolio(owner.acc(), "UK");

        // Enforce an unsafe tranfer.
        assert_noop!(
            Asset::unsafe_transfer(
                eu_portfolio,
                uk_portfolio,
                &ticker,
                1_000,
                None,
                None,
                IdentityId::default(),
                &mut weight_meter
            ),
            AssetError::SenderSameAsReceiver
        );
    });
}

#[test]
fn invalid_granularity_test() {
    test_with_owner(|owner| {
        let ticker = an_asset(owner, false);
        assert_noop!(
            Asset::issue(owner.origin(), ticker, 10_000, PortfolioKind::Default),
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
    let create = |ticker, name, is_divisible, funding_name| {
        Asset::create_asset(
            o.clone(),
            name,
            ticker,
            is_divisible,
            AssetType::default(),
            vec![],
            funding_name,
        )
    };

    let ta = Ticker::from_slice_truncated(&b"A"[..]);
    let max_length = <TestStorage as AssetConfig>::AssetNameMaxLength::get() + 1;
    assert_noop!(
        create(ta, bytes_of_len(b'A', max_length as usize), true, None),
        AssetError::MaxLengthOfAssetNameExceeded
    );

    let name: AssetName = ta.as_ref().into();
    assert_noop!(
        create(ta, name.clone(), true, Some(exceeded_funding_round_name()),),
        AssetError::FundingRoundNameMaxLengthExceeded,
    );

    assert_ok!(create(ta, name.clone(), false, None));
    assert_noop!(
        Asset::issue(o.clone(), ta, 1_000, PortfolioKind::Default),
        AssetError::InvalidGranularity,
    );

    let tb = Ticker::from_slice_truncated(&b"B"[..]);
    assert_ok!(create(tb, name.clone(), true, None));
    assert_noop!(
        Asset::issue(o.clone(), tb, u128::MAX, PortfolioKind::Default),
        AssetError::TotalSupplyAboveLimit,
    );

    let o2 = Origin::signed(other);
    let tc = Ticker::from_slice_truncated(&b"C"[..]);
    assert_ok!(Asset::register_ticker(o2.clone(), tc));
    assert_noop!(
        create(tc, name, true, None),
        AssetError::TickerAlreadyRegistered
    );
}

#[test]
fn asset_type_custom_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let case = |add| Asset::register_custom_asset_type(user.origin(), max_len_bytes(add));
        assert_too_long!(case(1));
        assert_ok!(case(0));
    });
}

#[test]
fn asset_type_custom_works() {
    ExtBuilder::default().build().execute_with(|| {
        let user = User::new(AccountKeyring::Alice);
        let register = |ty: &str| Asset::register_custom_asset_type(user.origin(), ty.into());
        let seq_is = |num| {
            assert_eq!(CustomTypeIdSequence::get().0, num);
        };
        let slot_has = |id, data: &str| {
            seq_is(id);
            let id = CustomAssetTypeId(id);
            let data = data.as_bytes();
            assert_eq!(CustomTypes::get(id), data);
            assert_eq!(CustomTypesInverse::get(data), Some(id));
        };

        // Nothing so far. Generator (G) at 0.
        seq_is(0);

        // Register first type. G -> 1.
        assert_ok!(register("foo"));
        slot_has(1, "foo");

        // Register same type. G unmoved.
        assert_ok!(register("foo"));
        slot_has(1, "foo");

        // Register different type. G -> 2.
        assert_ok!(register("bar"));
        slot_has(2, "bar");

        // Register same type. G unmoved.
        assert_ok!(register("bar"));
        slot_has(2, "bar");

        // Register different type. G -> 3.
        assert_ok!(register("foobar"));
        slot_has(3, "foobar");

        // Set G to max. Next registration fails.
        CustomTypeIdSequence::put(CustomAssetTypeId(u32::MAX));
        assert_noop!(register("qux"), BaseError::CounterOverflow);
    });
}

#[test]
fn invalid_custom_asset_type_check() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Dave);

        // Create ticker.
        let (ticker, mut token) = a_token(owner.did);

        let invalid_id = CustomAssetTypeId(1_000_000);
        token.asset_type = AssetType::Custom(invalid_id);
        assert_noop!(
            basic_asset(owner, ticker, &token),
            AssetError::InvalidCustomAssetTypeId
        );
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
            Asset::unsafe_can_transfer(
                None,
                default_portfolio,
                None,
                uk_portfolio,
                &ticker,
                100,
                &mut WeightMeter::max_limit_no_minimum(),
            )
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
            &mut WeightMeter::max_limit_no_minimum(),
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

#[test]
fn issuers_can_redeem_tokens_from_portfolio() {
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

            let portfolio_name = PortfolioName(vec![65u8; 5]);
            let next_portfolio_num = NextPortfolioNumber::get(&owner.did);
            let portfolio = PortfolioId::default_portfolio(owner.did);
            let user_portfolio = PortfolioId::user_portfolio(owner.did, next_portfolio_num.clone());
            Portfolio::create_portfolio(owner.origin(), portfolio_name.clone()).unwrap();

            Portfolio::move_portfolio_funds(
                owner.origin(),
                portfolio,
                user_portfolio,
                vec![Fund {
                    description: FundDescription::Fungible {
                        ticker,
                        amount: token.total_supply,
                    },
                    memo: None,
                }],
            )
            .unwrap();

            assert_eq!(
                PortfolioAssetBalances::get(&portfolio, &ticker),
                0u32.into()
            );
            assert_eq!(
                PortfolioAssetBalances::get(&user_portfolio, &ticker),
                token.total_supply
            );

            assert_noop!(
                Asset::redeem_from_portfolio(
                    bob.origin(),
                    ticker,
                    token.total_supply,
                    PortfolioKind::User(next_portfolio_num)
                ),
                EAError::UnauthorizedAgent
            );

            assert_noop!(
                Asset::redeem_from_portfolio(
                    owner.origin(),
                    ticker,
                    token.total_supply + 1,
                    PortfolioKind::User(next_portfolio_num)
                ),
                PortfolioError::InsufficientPortfolioBalance
            );

            assert_ok!(Asset::redeem_from_portfolio(
                owner.origin(),
                ticker,
                token.total_supply / 2,
                PortfolioKind::User(next_portfolio_num)
            ));

            assert_eq!(
                Asset::balance_of(&ticker, owner.did),
                token.total_supply / 2
            );
            assert_eq!(token_details(&ticker).total_supply, token.total_supply / 2);

            // Add auth for custody to be moved to bob
            let auth_id = Identity::add_auth(
                owner.did,
                Signatory::from(bob.did),
                AuthorizationData::PortfolioCustody(user_portfolio),
                None,
            );

            // Check that bob accepts auth
            assert_ok!(Portfolio::accept_portfolio_custody(bob.origin(), auth_id));

            assert_eq!(
                Portfolio::portfolio_custodian(user_portfolio),
                Some(bob.did)
            );

            // Check error is given when unauthorized custodian tries to redeem from portfolio
            assert_noop!(
                Asset::redeem_from_portfolio(
                    owner.origin(),
                    ticker,
                    token.total_supply,
                    PortfolioKind::User(next_portfolio_num)
                ),
                PortfolioError::UnauthorizedCustodian
            );

            // Remove bob as custodian
            assert_ok!(Portfolio::quit_portfolio_custody(
                bob.origin(),
                user_portfolio
            ));

            assert_ok!(Asset::redeem_from_portfolio(
                owner.origin(),
                ticker,
                token.total_supply / 2,
                PortfolioKind::User(next_portfolio_num)
            ));

            // Adds Bob as an external agent for the asset
            assert_ok!(ExternalAgents::unchecked_add_agent(
                ticker,
                bob.did,
                AgentGroup::Full
            ));

            // Remove owner as agent
            assert_ok!(ExternalAgents::remove_agent(
                owner.origin(),
                ticker,
                owner.did
            ));

            // Check error is given when unauthorized agent tries to redeem from portfolio
            assert_noop!(
                Asset::redeem_from_portfolio(
                    owner.origin(),
                    ticker,
                    1,
                    PortfolioKind::User(next_portfolio_num)
                ),
                EAError::UnauthorizedAgent
            );
        })
}

#[test]
fn issuers_can_change_asset_type() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let owner = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        // Create an asset
        let (ticker, token) = a_token(owner.did);
        assert_ok!(basic_asset(owner, ticker, &token));

        // Only the asset issuer is allowed to update the asset type
        assert_noop!(
            Asset::update_asset_type(alice.origin(), ticker, AssetType::EquityPreferred),
            EAError::UnauthorizedAgent
        );
        // Invalid Custom type id must be rejected
        assert_noop!(
            Asset::update_asset_type(
                owner.origin(),
                ticker,
                AssetType::Custom(CustomAssetTypeId(1))
            ),
            AssetError::InvalidCustomAssetTypeId
        );
        // Owner of the asset must be able to change the asset type, as long as it's not an invalid custom type
        assert_ok!(Asset::update_asset_type(
            owner.origin(),
            ticker,
            AssetType::EquityPreferred
        ));
        assert_eq!(
            token_details(&ticker).asset_type,
            AssetType::EquityPreferred
        );
    })
}

/// Only metadata keys that already have a value set can be locked.
#[test]
fn prevent_locking_an_empty_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_noop!(
            Asset::set_asset_metadata_details(
                alice.origin(),
                ticker,
                asset_metada_key,
                asset_metadata_detail
            ),
            AssetError::AssetMetadataValueIsEmpty
        );
    })
}

/// Only metadata keys that already exist can be deleted.
#[test]
fn remove_local_metadata_key_missing_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let local_key = AssetMetadataLocalKey(1);
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), ticker, local_key),
            AssetError::AssetMetadataKeyIsMissing
        );
    })
}

/// Only metadata keys that are not locked can be deleted.
#[test]
fn remove_local_metadata_key_locked_value() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            ticker,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::set_asset_metadata_details(
            alice.origin(),
            ticker,
            asset_metada_key,
            asset_metadata_detail
        ));
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), ticker, AssetMetadataLocalKey(1)),
            AssetError::AssetMetadataValueIsLocked
        );
    })
}

/// Only metadata keys that don't belong to NFT collections can be deleted.
#[test]
fn remove_nft_collection_metada_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER".as_ref());
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        let collection_keys: NFTCollectionKeys = vec![asset_metada_key.clone()].into();
        create_nft_collection(
            alice,
            ticker,
            AssetType::NonFungible(NonFungibleType::Derivative),
            collection_keys,
        );
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            ticker,
            asset_metada_key,
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_noop!(
            Asset::remove_local_metadata_key(alice.origin(), ticker, AssetMetadataLocalKey(1)),
            AssetError::AssetMetadataKeyBelongsToNFTCollection
        );
    })
}

/// Successfully deletes a local metadata key.
#[test]
fn remove_local_metadata_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            ticker,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::remove_local_metadata_key(
            alice.origin(),
            ticker,
            AssetMetadataLocalKey(1)
        ),);
        assert_eq!(
            AssetMetadataLocalKeyToName::get(&ticker, AssetMetadataLocalKey(1)),
            None
        );
        assert_eq!(
            AssetMetadataLocalNameToKey::get(&ticker, &asset_metadata_name),
            None
        );
        assert_eq!(
            AssetMetadataLocalSpecs::get(&ticker, &AssetMetadataLocalKey(1)),
            None
        );
        assert_eq!(AssetMetadataValues::get(&ticker, &asset_metada_key), None);
    })
}

/// Only metadata keys that already exist can have their value removed.
#[test]
fn remove_local_metadata_value_missing_key() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        assert_noop!(
            Asset::remove_metadata_value(
                alice.origin(),
                ticker,
                AssetMetadataKey::Local(AssetMetadataLocalKey(1))
            ),
            AssetError::AssetMetadataKeyIsMissing
        );
    })
}

/// Only metadata keys that are no locked can have their value removed.
#[test]
fn remove_local_metadata_value_locked_value() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec
        ));
        let asset_metadata_detail = AssetMetadataValueDetail {
            expire: None,
            lock_status: AssetMetadataLockStatus::Locked,
        };
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            ticker,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::set_asset_metadata_details(
            alice.origin(),
            ticker,
            asset_metada_key,
            asset_metadata_detail
        ));
        assert_noop!(
            Asset::remove_metadata_value(alice.origin(), ticker, asset_metada_key),
            AssetError::AssetMetadataValueIsLocked
        );
    })
}

/// Successfully removes a metadata value.
#[test]
fn remove_metadata_value() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();

        let alice = User::new(AccountKeyring::Alice);
        let ticker = an_asset(alice, true);
        let asset_metadata_name = AssetMetadataName(b"mylocalkey".to_vec());
        let asset_metadata_spec = AssetMetadataSpec {
            url: None,
            description: None,
            type_def: None,
        };
        assert_ok!(Asset::register_asset_metadata_local_type(
            alice.origin(),
            ticker,
            asset_metadata_name.clone(),
            asset_metadata_spec.clone()
        ));
        let asset_metada_key = AssetMetadataKey::Local(AssetMetadataLocalKey(1));
        assert_ok!(Asset::set_asset_metadata(
            alice.origin(),
            ticker,
            asset_metada_key.clone(),
            AssetMetadataValue(b"randomvalue".to_vec()),
            None,
        ));
        assert_ok!(Asset::remove_metadata_value(
            alice.origin(),
            ticker,
            asset_metada_key.clone(),
        ),);
        assert_eq!(
            AssetMetadataLocalKeyToName::get(&ticker, AssetMetadataLocalKey(1)),
            Some(asset_metadata_name.clone())
        );
        assert_eq!(
            AssetMetadataLocalNameToKey::get(&ticker, &asset_metadata_name),
            Some(AssetMetadataLocalKey(1))
        );
        assert_eq!(
            AssetMetadataLocalSpecs::get(&ticker, &AssetMetadataLocalKey(1)),
            Some(asset_metadata_spec)
        );
        assert_eq!(AssetMetadataValues::get(&ticker, &asset_metada_key), None);
    })
}

#[test]
fn issue_token_invalid_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_user_portfolio = PortfolioKind::User(PortfolioNumber(1));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::issue(alice.origin(), ticker, issued_amount, alice_user_portfolio),
            PortfolioError::PortfolioDoesNotExist
        );
    })
}

#[test]
fn issue_token_unassigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_user_portfolio = PortfolioKind::User(PortfolioNumber(1));

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AlicePortfolio".to_vec())
        ));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            issued_amount,
            alice_user_portfolio
        ));
        assert_eq!(BalanceOf::get(ticker, alice.did), issued_amount);
    })
}

#[test]
fn issue_token_assigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let portfolio_id = PortfolioId::new(alice.did, PortfolioKind::Default);

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        // Change custody of the default portfolio
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(portfolio_id),
            None,
        );
        assert_ok!(Portfolio::accept_portfolio_custody(
            bob.origin(),
            authorization_id
        ));

        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            issued_amount,
            PortfolioKind::Default
        ));
        assert_eq!(BalanceOf::get(ticker, alice.did), issued_amount);
    })
}

#[test]
fn redeem_token_unassigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            issued_amount,
            PortfolioKind::Default
        ));
        assert_ok!(Asset::redeem_from_portfolio(
            alice.origin(),
            ticker,
            issued_amount,
            PortfolioKind::Default
        ));
        assert_eq!(BalanceOf::get(ticker, alice.did), 0);
    })
}

#[test]
fn redeem_token_assigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let issued_amount = ONE_UNIT;
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let portfolio_id = PortfolioId::new(alice.did, PortfolioKind::Default);

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        // Change custody of the default portfolio
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(portfolio_id),
            None,
        );
        assert_ok!(Portfolio::accept_portfolio_custody(
            bob.origin(),
            authorization_id
        ));

        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            issued_amount,
            PortfolioKind::Default
        ));
        assert_noop!(
            Asset::redeem_from_portfolio(
                alice.origin(),
                ticker,
                issued_amount,
                PortfolioKind::Default
            ),
            PortfolioError::UnauthorizedCustodian
        );
    })
}
fn pre_approve_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = ticker("TICKER");
        let alice = User::new(AccountKeyring::Alice);
        Asset::pre_approve_ticker(alice.origin(), ticker).unwrap();

        assert!(PreApprovedTicker::get(alice.did, ticker));
        assert!(!TickersExemptFromAffirmation::get(ticker));
        assert!(Asset::skip_ticker_affirmation(&alice.did, &ticker));
    });
}

#[test]
fn remove_ticker_pre_approval() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = ticker("TICKER");
        let alice = User::new(AccountKeyring::Alice);
        Asset::pre_approve_ticker(alice.origin(), ticker).unwrap();
        Asset::remove_ticker_pre_approval(alice.origin(), ticker).unwrap();

        assert!(!PreApprovedTicker::get(alice.did, ticker));
        assert!(!TickersExemptFromAffirmation::get(ticker));
        assert!(!Asset::skip_ticker_affirmation(&alice.did, &ticker));
    });
}

#[test]
fn unauthorized_custodian_ticker_exemption() {
    ExtBuilder::default().build().execute_with(|| {
        let ticker: Ticker = ticker("TICKER");
        let alice = User::new(AccountKeyring::Alice);

        assert_noop!(
            Asset::exempt_ticker_affirmation(alice.origin(), ticker),
            DispatchError::BadOrigin
        );
        assert_ok!(Asset::exempt_ticker_affirmation(root(), ticker,),);

        assert!(!PreApprovedTicker::get(alice.did, ticker));
        assert!(TickersExemptFromAffirmation::get(ticker));
        assert!(Asset::skip_ticker_affirmation(&alice.did, &ticker));

        assert_noop!(
            Asset::remove_ticker_affirmation_exemption(alice.origin(), ticker),
            DispatchError::BadOrigin
        );
        assert_ok!(Asset::remove_ticker_affirmation_exemption(root(), ticker,),);

        assert!(!PreApprovedTicker::get(alice.did, ticker));
        assert!(!TickersExemptFromAffirmation::get(ticker));
        assert!(!Asset::skip_ticker_affirmation(&alice.did, &ticker));
    });
}
