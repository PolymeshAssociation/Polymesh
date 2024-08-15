use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap};
use sp_keyring::AccountKeyring;

use pallet_asset::{Assets, BalanceOf};
use pallet_portfolio::{PortfolioAssetBalances, PortfolioAssetCount, PortfolioLockedAssets};
use polymesh_primitives::asset::{AssetType, NonFungibleType};
use polymesh_primitives::{
    AuthorizationData, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, Signatory,
};

use super::setup::{create_and_issue_sample_asset, ISSUE_AMOUNT};
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

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_eq!(
            PortfolioAssetBalances::get(&alice_default_portfolio, &asset_id),
            ISSUE_AMOUNT
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_default_portfolio, &asset_id),
            0
        );
        assert_eq!(BalanceOf::get(&asset_id, &alice.did), ISSUE_AMOUNT);
        assert_eq!(Assets::get(&asset_id).unwrap().total_supply, ISSUE_AMOUNT);
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

        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"AliceUserPortfolio".to_vec())
        ));
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            ISSUE_AMOUNT,
            PortfolioKind::User(PortfolioNumber(1))
        ));

        assert_eq!(
            PortfolioAssetBalances::get(&alice_user_portfolio, asset_id),
            ISSUE_AMOUNT
        );
        assert_eq!(
            PortfolioLockedAssets::get(&alice_user_portfolio, asset_id),
            0
        );
        assert_eq!(BalanceOf::get(asset_id, &alice.did), ISSUE_AMOUNT);
        assert_eq!(Assets::get(asset_id).unwrap().total_supply, ISSUE_AMOUNT);
    });
}

#[test]
fn issue_tokens_invalid_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let alice_user_portfolio = PortfolioKind::User(PortfolioNumber(1));

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::issue(alice.origin(), asset_id, 1_000, alice_user_portfolio),
            PortfolioError::PortfolioDoesNotExist
        );
    })
}

#[test]
fn issue_tokens_assigned_custody() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let portfolio_id = PortfolioId::new(alice.did, PortfolioKind::Default);

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
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
        )
        .unwrap();
        assert_ok!(Portfolio::accept_portfolio_custody(
            bob.origin(),
            authorization_id
        ));

        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            1_000,
            PortfolioKind::Default
        ));
        assert_eq!(BalanceOf::get(asset_id, alice.did), 1_000);
        assert_eq!(PortfolioAssetBalances::get(&portfolio_id, asset_id), 1_000);
        assert_eq!(PortfolioAssetBalances::get(&portfolio_id, asset_id), 1_000);
    })
}

#[test]
fn issue_tokens_no_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        assert_noop!(
            Asset::issue(
                alice.origin(),
                [0; 16].into(),
                1_000,
                PortfolioKind::Default
            ),
            ExternalAgentsError::UnauthorizedAgent
        );
    })
}

#[test]
fn issue_tokens_no_auth() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_noop!(
            Asset::issue(bob.origin(), asset_id, 1_000, PortfolioKind::Default),
            ExternalAgentsError::UnauthorizedAgent
        );
    })
}

#[test]
fn issue_tokens_not_granular() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            false,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_noop!(
            Asset::issue(alice.origin(), asset_id, 1_000, PortfolioKind::Default),
            AssetError::InvalidGranularity
        );
    })
}

#[test]
fn issue_tokens_invalid_token_type() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::NonFungible(NonFungibleType::Invoice),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::issue(alice.origin(), asset_id, 1_000, PortfolioKind::Default),
            AssetError::UnexpectedNonFungibleToken
        );
    })
}
