use super::{
    asset_test::{a_token, basic_asset, max_len_bytes},
    storage::{TestStorage, User},
    ExtBuilder,
};
use frame_support::{assert_noop, assert_ok, StorageMap};
use pallet_asset::SecurityToken;
use pallet_portfolio::MovePortfolioItem;
use polymesh_common_utilities::portfolio::PortfolioSubTrait;
use polymesh_primitives::{
    AuthorizationData, AuthorizationError, PortfolioId, PortfolioName, PortfolioNumber, Signatory,
    Ticker,
};
use test_client::AccountKeyring;

type Asset = pallet_asset::Module<TestStorage>;
type Error = pallet_portfolio::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Origin = <TestStorage as frame_system::Trait>::Origin;
type Portfolio = pallet_portfolio::Module<TestStorage>;

fn create_token() -> (Ticker, SecurityToken<u128>) {
    let owner = User::existing(AccountKeyring::Alice);
    let r = a_token(owner.did);
    assert_ok!(basic_asset(owner, r.0, &r.1));
    r
}

fn create_portfolio() -> (User, PortfolioNumber) {
    let owner = User::new(AccountKeyring::Alice);
    let name = PortfolioName::from([42u8].to_vec());
    let num = Portfolio::next_portfolio_number(&owner.did);
    assert_eq!(num, PortfolioNumber(1));
    assert_ok!(Portfolio::create_portfolio(owner.origin(), name.clone()));
    assert_eq!(Portfolio::portfolios(&owner.did, num), name);
    (owner, num)
}

#[test]
fn portfolio_name_too_long() {
    ExtBuilder::default().build().execute_with(|| {
        let owner = User::new(AccountKeyring::Alice);
        let id = Portfolio::next_portfolio_number(owner.did);
        let create = |name| Portfolio::create_portfolio(owner.origin(), name);
        let rename = |name| Portfolio::rename_portfolio(owner.origin(), id, name);
        assert_too_long!(create(max_len_bytes(1)));
        assert_ok!(create(max_len_bytes(0)));
        assert_too_long!(rename(max_len_bytes(1)));
        assert_ok!(rename(b"".into()));
        assert_ok!(rename(max_len_bytes(0)));
    });
}

#[test]
fn can_create_rename_delete_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner, num) = create_portfolio();
        let new_name = PortfolioName::from([55u8].to_vec());
        assert_ok!(Portfolio::rename_portfolio(
            owner.origin(),
            num,
            new_name.clone()
        ));
        assert_eq!(
            Portfolio::next_portfolio_number(&owner.did),
            PortfolioNumber(2)
        );
        assert_eq!(Portfolio::portfolios(&owner.did, num), new_name);
        assert_ok!(Portfolio::delete_portfolio(owner.origin(), num));
    });
}

#[test]
fn can_recover_funds_from_deleted_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner, num) = create_portfolio();
        let (ticker, token) = create_token();
        let owner_default_portfolio = PortfolioId::default_portfolio(owner.did);
        let owner_user_portfolio = PortfolioId::user_portfolio(owner.did, num);

        // Move funds to new portfolio
        let move_amount = token.total_supply / 2;
        assert_ok!(Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_default_portfolio,
            owner_user_portfolio,
            vec![MovePortfolioItem {
                ticker,
                amount: move_amount,
            }]
        ));
        let ensure_balances = |default_portfolio_balance, user_portfolio_balance| {
            assert_eq!(
                Portfolio::default_portfolio_balance(owner.did, &ticker),
                default_portfolio_balance
            );
            assert_eq!(
                Portfolio::user_portfolio_balance(owner.did, num, &ticker),
                user_portfolio_balance
            );
        };
        ensure_balances(token.total_supply - move_amount, move_amount);

        // Delete portfolio
        assert_ok!(Portfolio::delete_portfolio(owner.origin(), num));
        ensure_balances(token.total_supply - move_amount, move_amount);

        // Recover funds
        assert_ok!(Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_user_portfolio,
            owner_default_portfolio,
            vec![MovePortfolioItem {
                ticker,
                amount: move_amount,
            }]
        ));
        ensure_balances(token.total_supply, 0);
    });
}

#[test]
fn can_move_asset_from_portfolio() {
    ExtBuilder::default()
        .build()
        .execute_with(do_move_asset_from_portfolio);
}

fn do_move_asset_from_portfolio() {
    let (owner, num) = create_portfolio();
    let bob = User::new(AccountKeyring::Bob);
    let (ticker, token) = create_token();
    assert_eq!(
        Portfolio::default_portfolio_balance(owner.did, &ticker),
        token.total_supply,
    );
    assert_eq!(
        Portfolio::user_portfolio_balance(owner.did, num, &ticker),
        0,
    );

    let owner_default_portfolio = PortfolioId::default_portfolio(owner.did);
    let owner_user_portfolio = PortfolioId::user_portfolio(owner.did, num);

    // Attempt to move more than the total supply.
    assert_noop!(
        Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_default_portfolio,
            owner_user_portfolio,
            vec![MovePortfolioItem {
                ticker,
                amount: token.total_supply * 2,
            }]
        ),
        Error::InsufficientPortfolioBalance
    );
    assert_noop!(
        Portfolio::ensure_portfolio_transfer_validity(
            &owner_default_portfolio,
            &owner_user_portfolio,
            &ticker,
            &(token.total_supply * 2),
        ),
        Error::InsufficientPortfolioBalance
    );

    // Attempt to move to the same portfolio.
    assert_noop!(
        Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_default_portfolio,
            owner_default_portfolio,
            vec![MovePortfolioItem { ticker, amount: 1 }]
        ),
        Error::DestinationIsSamePortfolio
    );
    assert_noop!(
        Portfolio::ensure_portfolio_transfer_validity(
            &owner_default_portfolio,
            &owner_default_portfolio,
            &ticker,
            &1,
        ),
        Error::DestinationIsSamePortfolio
    );

    // Attempt to move to a non-existent portfolio.
    assert_noop!(
        Portfolio::ensure_portfolio_transfer_validity(
            &owner_default_portfolio,
            &PortfolioId::user_portfolio(owner.did, PortfolioNumber(666)),
            &ticker,
            &1,
        ),
        Error::PortfolioDoesNotExist
    );

    // Attempt to move by another identity.
    assert_noop!(
        Portfolio::move_portfolio_funds(
            bob.origin(),
            owner_default_portfolio,
            owner_user_portfolio,
            vec![MovePortfolioItem { ticker, amount: 1 }]
        ),
        Error::UnauthorizedCustodian
    );

    // Move an amount within bounds.
    let move_amount = token.total_supply / 2;
    assert_ok!(Portfolio::move_portfolio_funds(
        owner.origin(),
        owner_default_portfolio,
        owner_user_portfolio,
        vec![MovePortfolioItem {
            ticker,
            amount: move_amount,
        }]
    ));
    assert_ok!(Portfolio::ensure_portfolio_transfer_validity(
        &owner_default_portfolio,
        &owner_user_portfolio,
        &ticker,
        &move_amount,
    ));
    assert_eq!(
        Portfolio::default_portfolio_balance(owner.did, &ticker),
        token.total_supply - move_amount,
    );
    assert_eq!(
        Portfolio::user_portfolio_balance(owner.did, num, &ticker),
        move_amount,
    );
}

#[test]
fn can_lock_unlock_assets() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner, num) = create_portfolio();
        let (ticker, token) = create_token();
        assert_eq!(
            Portfolio::default_portfolio_balance(owner.did, &ticker),
            token.total_supply,
        );

        let owner_default_portfolio = PortfolioId::default_portfolio(owner.did);
        let owner_user_portfolio = PortfolioId::user_portfolio(owner.did, num);

        // Lock half of the tokens
        let lock_amount = token.total_supply / 2;
        assert_ok!(Portfolio::lock_tokens(
            &owner_default_portfolio,
            &ticker,
            &lock_amount
        ));

        assert_eq!(
            Portfolio::default_portfolio_balance(owner.did, &ticker),
            token.total_supply,
        );
        assert_eq!(
            Portfolio::locked_assets(owner_default_portfolio, &ticker),
            lock_amount,
        );

        assert_noop!(
            Portfolio::move_portfolio_funds(
                owner.origin(),
                owner_default_portfolio,
                owner_user_portfolio,
                vec![MovePortfolioItem {
                    ticker,
                    amount: token.total_supply,
                }]
            ),
            Error::InsufficientPortfolioBalance
        );

        // Transfer for unlocked tokens succeeds
        assert_ok!(Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_default_portfolio,
            owner_user_portfolio,
            vec![MovePortfolioItem {
                ticker,
                amount: lock_amount,
            }]
        ));
        assert_eq!(
            Portfolio::default_portfolio_balance(owner.did, &ticker),
            token.total_supply - lock_amount,
        );
        assert_eq!(
            Portfolio::user_portfolio_balance(owner.did, num, &ticker),
            lock_amount,
        );
        assert_eq!(
            Portfolio::locked_assets(owner_default_portfolio, &ticker),
            lock_amount,
        );

        // Transfer of any more tokens fails
        assert_noop!(
            Portfolio::move_portfolio_funds(
                owner.origin(),
                owner_default_portfolio,
                owner_user_portfolio,
                vec![MovePortfolioItem { ticker, amount: 1 }]
            ),
            Error::InsufficientPortfolioBalance
        );

        // Unlock tokens
        assert_ok!(Portfolio::unlock_tokens(
            &owner_default_portfolio,
            &ticker,
            &lock_amount
        ));

        assert_eq!(
            Portfolio::default_portfolio_balance(owner.did, &ticker),
            token.total_supply - lock_amount,
        );
        assert_eq!(
            Portfolio::user_portfolio_balance(owner.did, num, &ticker),
            lock_amount,
        );
        assert_eq!(
            Portfolio::locked_assets(owner_default_portfolio, &ticker),
            0,
        );

        // Transfer of all tokens succeeds since there is no lock anymore
        assert_ok!(Portfolio::move_portfolio_funds(
            owner.origin(),
            owner_default_portfolio,
            owner_user_portfolio,
            vec![MovePortfolioItem {
                ticker,
                amount: token.total_supply - lock_amount,
            }]
        ));
        assert_eq!(Portfolio::default_portfolio_balance(owner.did, &ticker), 0,);
        assert_eq!(
            Portfolio::user_portfolio_balance(owner.did, num, &ticker),
            token.total_supply,
        );
        assert_eq!(
            Portfolio::locked_assets(owner_default_portfolio, &ticker),
            0,
        );
    });
}

#[test]
fn can_take_custody_of_portfolios() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner, num) = create_portfolio();
        let bob = User::new(AccountKeyring::Bob);

        let owner_default_portfolio = PortfolioId::default_portfolio(owner.did);
        let owner_user_portfolio = PortfolioId::user_portfolio(owner.did, num);

        // Custody of all portfolios is with the owner identity by default
        assert_ok!(Portfolio::ensure_portfolio_custody(
            owner_default_portfolio,
            owner.did
        ));
        assert_ok!(Portfolio::ensure_portfolio_custody(
            owner_user_portfolio,
            owner.did
        ));
        assert_eq!(
            Portfolio::portfolio_custodian(owner_default_portfolio),
            None
        );
        assert_eq!(Portfolio::portfolio_custodian(owner_user_portfolio), None);
        assert_eq!(
            Portfolio::portfolios_in_custody(bob.did, owner_user_portfolio),
            false
        );

        // Bob can not issue authorization for custody transfer of a portfolio they don't have custody of
        let mut auth_id = Identity::add_auth(
            bob.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(owner_user_portfolio),
            None,
        );
        assert_noop!(
            Identity::accept_authorization(bob.origin(), auth_id),
            AuthorizationError::Unauthorized
        );

        // Can not accept an invalid auth
        assert_noop!(
            Identity::accept_authorization(bob.origin(), auth_id + 1),
            AuthorizationError::Invalid
        );

        // Can accept a valid custody transfer auth
        auth_id = Identity::add_auth(
            owner.did,
            Signatory::from(bob.did),
            AuthorizationData::PortfolioCustody(owner_user_portfolio),
            None,
        );
        assert_ok!(Identity::accept_authorization(bob.origin(), auth_id));

        assert_ok!(Portfolio::ensure_portfolio_custody(
            owner_default_portfolio,
            owner.did
        ));
        assert_ok!(Portfolio::ensure_portfolio_custody(
            owner_user_portfolio,
            bob.did
        ));
        assert_noop!(
            Portfolio::ensure_portfolio_custody(owner_user_portfolio, owner.did),
            Error::UnauthorizedCustodian
        );
        assert_eq!(
            Portfolio::portfolio_custodian(owner_default_portfolio),
            None
        );
        assert_eq!(
            Portfolio::portfolio_custodian(owner_user_portfolio),
            Some(bob.did)
        );
        assert_eq!(
            Portfolio::portfolios_in_custody(bob.did, owner_user_portfolio),
            true
        );

        // Owner can not issue authorization for custody transfer of a portfolio they don't have custody of
        auth_id = Identity::add_auth(
            owner.did,
            Signatory::from(owner.did),
            AuthorizationData::PortfolioCustody(owner_user_portfolio),
            None,
        );
        assert_noop!(
            Identity::accept_authorization(owner.origin(), auth_id),
            AuthorizationError::Unauthorized
        );

        // Bob transfers portfolio custody back to Alice.
        auth_id = Identity::add_auth(
            bob_did,
            Signatory::from(owner_did),
            AuthorizationData::PortfolioCustody(owner_user_portfolio),
            None,
        );
        assert_ok!(Identity::accept_authorization(
            owner_signed.clone(),
            auth_id
        ));
        assert_eq!(
            Portfolio::portfolios_in_custody(owner_did, owner_user_portfolio),
            true
        );
        // The mapping is removed which means the owner is the custodian.
        assert_eq!(
            pallet_portfolio::PortfolioCustodian::contains_key(&owner_user_portfolio),
            false
        );
    });
}
