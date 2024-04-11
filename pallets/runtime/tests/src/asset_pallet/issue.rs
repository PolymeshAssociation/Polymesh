use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap};
use sp_keyring::AccountKeyring;

use pallet_asset::{BalanceOf, Tokens};
use pallet_portfolio::{PortfolioAssetBalances, PortfolioAssetCount, PortfolioLockedAssets};
use polymesh_primitives::asset::{AssetType, NonFungibleType};
use polymesh_primitives::{
    AuthorizationData, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, Signatory,
    Ticker,
};

use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type ExternalAgentsError = pallet_external_agents::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;

#[test]
fn issue_tokens_default_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

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
            1_000,
            PortfolioKind::Default
        ));

        assert_eq!(
            PortfolioAssetBalances::get(&alice_default_portfolio, &ticker),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_default_portfolio, &ticker),
            0
        );
        assert_eq!(BalanceOf::get(&ticker, &alice.did), 1_000);
        assert_eq!(Tokens::get(&ticker).unwrap().total_supply, 1_000);
        assert_eq!(PortfolioAssetCount::get(alice_default_portfolio), 1);
    });
}

#[test]
fn issue_tokens_user_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let alice_user_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AliceUserPortfolio".to_vec())
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
            1_000,
            PortfolioKind::User(PortfolioNumber(1))
        ));

        assert_eq!(
            PortfolioAssetBalances::get(&alice_user_portfolio, &ticker),
            1_000
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_user_portfolio, &ticker),
            0
        );
        assert_eq!(BalanceOf::get(&ticker, &alice.did), 1_000);
        assert_eq!(Tokens::get(&ticker).unwrap().total_supply, 1_000);
    });
}

#[test]
fn issue_tokens_invalid_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
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
            Asset::issue(alice.origin(), ticker, 1_000, alice_user_portfolio),
            PortfolioError::PortfolioDoesNotExist
        );
    })
}

#[test]
fn issue_tokens_assigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
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
            1_000,
            PortfolioKind::Default
        ));
        assert_eq!(BalanceOf::get(ticker, alice.did), 1_000);
        assert_eq!(PortfolioAssetBalances::get(&portfolio_id, &ticker), 1_000);
        assert_eq!(PortfolioAssetBalances::get(&portfolio_id, &ticker), 1_000);
    })
}

#[test]
fn issue_tokens_no_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_noop!(
            Asset::issue(alice.origin(), ticker, 1_000, PortfolioKind::Default),
            ExternalAgentsError::UnauthorizedAgent
        );
    })
}

#[test]
fn issue_tokens_no_auth() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
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
        assert_noop!(
            Asset::issue(bob.origin(), ticker, 1_000, PortfolioKind::Default),
            ExternalAgentsError::UnauthorizedAgent
        );
    })
}

#[test]
fn issue_tokens_not_granular() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            false,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_noop!(
            Asset::issue(alice.origin(), ticker, 1_000, PortfolioKind::Default),
            AssetError::InvalidGranularity
        );
    })
}

#[test]
fn issue_tokens_invalid_token_type() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::NonFungible(NonFungibleType::Invoice),
            Vec::new(),
            None,
        ));
        assert_noop!(
            Asset::issue(alice.origin(), ticker, 1_000, PortfolioKind::Default),
            AssetError::UnexpectedNonFungibleToken
        );
    })
}
