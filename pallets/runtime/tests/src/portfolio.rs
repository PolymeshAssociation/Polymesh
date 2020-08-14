use super::{
    storage::{register_keyring_account, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok};
use pallet_asset::{AssetType, SecurityToken};
use pallet_portfolio::MovePortfolioItem;
use polymesh_primitives::{PortfolioName, Ticker};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Asset = pallet_asset::Module<TestStorage>;
type Error = pallet_portfolio::Error<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Portfolio = pallet_portfolio::Module<TestStorage>;

#[test]
fn can_create_rename_delete_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let owner_signed = Origin::signed(AccountKeyring::Alice.public());
        let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
        let name = PortfolioName::from([42u8].to_vec());
        let num = Portfolio::next_portfolio_number(&owner_did);
        assert_ok!(Portfolio::create_portfolio(
            owner_signed.clone(),
            name.clone()
        ));
        assert_eq!(Portfolio::portfolios(&owner_did, num), Some(name));
        let new_name = PortfolioName::from([55u8].to_vec());
        assert_ok!(Portfolio::rename_portfolio(
            owner_signed.clone(),
            num,
            new_name.clone()
        ));
        assert_eq!(Portfolio::portfolios(&owner_did, num), Some(new_name));
        assert_ok!(Portfolio::delete_portfolio(owner_signed.clone(), num));
        assert!(Portfolio::portfolios(&owner_did, num).is_none());
    });
}

#[test]
fn can_move_asset_from_portfolio() {
    ExtBuilder::default()
        .build()
        .execute_with(do_move_asset_from_portfolio);
}

fn do_move_asset_from_portfolio() {
    let owner_signed = Origin::signed(AccountKeyring::Alice.public());
    let owner_did = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob_signed = Origin::signed(AccountKeyring::Bob.public());
    let _ = register_keyring_account(AccountKeyring::Bob).unwrap();
    let total_supply = 1_000_000;
    let token = SecurityToken {
        name: vec![0x01].into(),
        owner_did,
        total_supply,
        divisible: true,
        asset_type: AssetType::default(),
        ..Default::default()
    };
    let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
    assert_ok!(Asset::create_asset(
        owner_signed.clone(),
        token.name.clone(),
        ticker,
        token.total_supply,
        token.divisible,
        token.asset_type.clone(),
        vec![],
        None,
    ));
    assert_eq!(
        Portfolio::default_portfolio_balance(owner_did, &ticker),
        total_supply,
    );
    let name = PortfolioName::from([42u8].to_vec());
    let num = Portfolio::next_portfolio_number(&owner_did);
    assert_ok!(Portfolio::create_portfolio(
        owner_signed.clone(),
        name.clone(),
    ));
    assert_eq!(Portfolio::portfolios(&owner_did, num), Some(name));
    assert_eq!(
        Portfolio::user_portfolio_balance(owner_did, num, &ticker),
        0,
    );
    // Attempt to move more than the total supply.
    assert_err!(
        Portfolio::move_portfolio(
            owner_signed.clone(),
            None,
            Some(num),
            vec![MovePortfolioItem {
                ticker,
                amount: total_supply * 2,
            }]
        ),
        Error::InsufficientPortfolioBalance
    );
    // Attempt to move to the same portfolio.
    assert_err!(
        Portfolio::move_portfolio(
            owner_signed.clone(),
            None,
            None,
            vec![MovePortfolioItem { ticker, amount: 1 }]
        ),
        Error::DestinationIsSamePortfolio
    );
    // Attempt to move to a non-existent portfolio.
    assert_err!(
        Portfolio::move_portfolio(
            owner_signed.clone(),
            None,
            Some(num + 777),
            vec![MovePortfolioItem { ticker, amount: 1 }]
        ),
        Error::PortfolioDoesNotExist
    );
    // Attempt to move by another identity.
    assert_err!(
        Portfolio::move_portfolio(
            bob_signed.clone(),
            None,
            Some(num),
            vec![MovePortfolioItem { ticker, amount: 1 }]
        ),
        Error::PortfolioDoesNotExist
    );
    // Move an amount within bounds.
    let move_amount = total_supply / 2;
    assert_ok!(Portfolio::move_portfolio(
        owner_signed.clone(),
        None,
        Some(num),
        vec![MovePortfolioItem {
            ticker,
            amount: move_amount,
        }]
    ));
    assert_eq!(
        Portfolio::default_portfolio_balance(owner_did, &ticker),
        total_supply - move_amount,
    );
    assert_eq!(
        Portfolio::user_portfolio_balance(owner_did, num, &ticker),
        move_amount,
    );
}
