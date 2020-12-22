use super::{
    storage::{register_keyring_account, Checkpoint, TestStorage},
    ExtBuilder,
};

use polymesh_common_utilities::{asset::AssetType, traits::CommonTrait};
use polymesh_primitives::{calendar::CheckpointId, PortfolioId, Ticker};
use polymesh_runtime_common::dividend::{self, Dividend};

use pallet_asset::{self as asset, SecurityToken};
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;

use frame_support::{assert_ok, traits::Currency};
use frame_system::ensure_signed;

use chrono::{prelude::*, Duration};
use lazy_static::lazy_static;
use test_client::{self, AccountKeyring};

use std::{
    collections::HashMap,
    convert::TryFrom,
    sync::{Arc, Mutex},
};

lazy_static! {
    static ref TOKEN_MAP: Arc<
        Mutex<
        HashMap<
        Ticker,
        SecurityToken<
            <TestStorage as CommonTrait>::Balance,
            >,
            >,
            >,
            > = Arc::new(Mutex::new(HashMap::new()));
    /// Because Rust's Mutex is not recursive a second symbolic lock is necessary
    static ref TOKEN_MAP_OUTER_LOCK: Arc<Mutex<()>> = Arc::new(Mutex::new(()));
}

type Timestamp = pallet_timestamp::Module<TestStorage>;
type DividendModule = dividend::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;

#[test]
fn correct_dividend_must_work() {
    ExtBuilder::default().build().execute_with(|| {
        let token_owner_acc = Origin::signed(AccountKeyring::Alice.public());
        let token_owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let payout_owner_acc = Origin::signed(AccountKeyring::Bob.public());
        let payout_owner_did = register_keyring_account(AccountKeyring::Bob).unwrap();

        // A token representing 1M shares
        let token = SecurityToken {
            name: vec![b'A'].into(),
            owner_did: token_owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let payout_token = SecurityToken {
            name: vec![b'B'].into(),
            owner_did: payout_owner_did,
            total_supply: 200_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        let payout_ticker = Ticker::try_from(payout_token.name.as_slice()).unwrap();
        let token_owner_account = ensure_signed(token_owner_acc.clone()).ok().unwrap();
        Balances::make_free_balance_be(&token_owner_account, 1_000_000);
        let payout_owner_account = ensure_signed(payout_owner_acc.clone()).ok().unwrap();
        Balances::make_free_balance_be(&payout_owner_account, 1_000_000);
        // Share issuance is successful
        assert_ok!(Asset::create_asset(
            token_owner_acc.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            token.asset_type.clone(),
            vec![],
            None,
        ));

        assert_ok!(Asset::create_asset(
            payout_owner_acc.clone(),
            payout_token.name.clone(),
            payout_ticker,
            payout_token.total_supply,
            true,
            payout_token.asset_type.clone(),
            vec![],
            None,
        ));

        // Prepare an exempted investor
        let investor_acc = Origin::signed(AccountKeyring::Charlie.public());
        let investor_did = register_keyring_account(AccountKeyring::Charlie).unwrap();
        let investor_account_id = ensure_signed(investor_acc.clone()).ok().unwrap();
        Balances::make_free_balance_be(&investor_account_id, 1_000_000);

        let amount_invested = 50_000;

        let now = Utc::now();
        Timestamp::set_timestamp(now.timestamp() as u64);

        // We need a lock to exist till assertions are done
        let outer = TOKEN_MAP_OUTER_LOCK.lock().unwrap();
        *TOKEN_MAP.lock().unwrap() = {
            let mut map = HashMap::new();
            map.insert(ticker, token.clone());
            map
        };

        drop(outer);

        // Allow all transfers
        assert_ok!(ComplianceManager::add_compliance_requirement(
            token_owner_acc.clone(),
            ticker,
            vec![],
            vec![]
        ));

        assert_ok!(ComplianceManager::add_compliance_requirement(
            payout_owner_acc.clone(),
            payout_ticker,
            vec![],
            vec![]
        ));

        // Transfer tokens to investor
        assert_ok!(Asset::unsafe_transfer(
            PortfolioId::default_portfolio(token_owner_did),
            PortfolioId::default_portfolio(investor_did),
            &ticker,
            amount_invested
        ));

        // Create checkpoint for token
        assert_ok!(Checkpoint::create_checkpoint(
            token_owner_acc.clone(),
            ticker
        ));

        // Checkpoints are 1-indexed
        let checkpoint_id = CheckpointId(1);

        let dividend = Dividend {
            amount: 500_000,
            amount_left: 500_000,
            remaining_claimed: false,
            matures_at: Some((now - Duration::hours(1)).timestamp() as u64),
            expires_at: Some((now + Duration::hours(1)).timestamp() as u64),
            payout_currency: payout_ticker,
            checkpoint_id,
        };

        // Transfer payout tokens to asset owner
        assert_ok!(Asset::unsafe_transfer(
            PortfolioId::default_portfolio(payout_owner_did),
            PortfolioId::default_portfolio(token_owner_did),
            &payout_ticker,
            dividend.amount
        ));

        // Create the dividend for asset
        assert_ok!(DividendModule::new(
            token_owner_acc.clone(),
            dividend.amount,
            ticker,
            dividend.matures_at.clone().unwrap(),
            dividend.expires_at.clone().unwrap(),
            dividend.payout_currency.clone(),
            dividend.checkpoint_id
        ));

        // Compare created dividend with the expected structure
        assert_eq!(
            DividendModule::get_dividend(&ticker, 0),
            Some(dividend.clone())
        );

        // Claim investor's share
        assert_ok!(DividendModule::claim(investor_acc.clone(), ticker, 0,));

        // Check if the correct amount was added to investor balance
        let share = dividend.amount * amount_invested / token.total_supply;
        assert_eq!(Asset::balance_of(payout_ticker, investor_did), share);

        // Check if amount_left was adjusted correctly
        let current_entry =
            DividendModule::get_dividend(&ticker, 0).expect("Could not retrieve dividend");
        assert_eq!(current_entry.amount_left, current_entry.amount - share);
    });
}
