use chrono::{prelude::*, Duration};
use core::result::Result as StdResult;
use frame_support::traits::Currency;
use frame_support::{assert_err, assert_ok};
use frame_system::ensure_signed;
use polymesh_runtime_common::{constants::currency::MAX_SUPPLY, traits::CommonTrait};
use test_client::{self, AccountKeyring};

use polymesh_primitives::Ticker;

use crate::{
    simple_token::{self, SimpleTokenRecord},
    test::{
        storage::{make_account, TestStorage},
        ExtBuilder,
    },
};

use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

type SimpleToken = simple_token::Module<TestStorage>;
type Error = simple_token::Error<TestStorage>;

#[test]
fn create_token_works() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();

        let ticker = Ticker::from(&[0x01][..]);
        let total_supply = 1_000_000;

        // Issuance is successful
        assert_ok!(SimpleToken::create_token(
            owner_signed.clone(),
            ticker,
            total_supply
        ));

        assert_eq!(
            SimpleToken::tokens(ticker),
            SimpleTokenRecord {
                ticker,
                total_supply,
                owner_did
            }
        );

        assert_err!(
            SimpleToken::create_token(owner_signed.clone(), ticker, total_supply),
            Error::TickerAlreadyExists
        );

        assert_ok!(SimpleToken::create_token(
            owner_signed.clone(),
            Ticker::from("1234567890123456789012345678901234567890".as_bytes()),
            total_supply,
        ));
        assert_eq!(
            SimpleToken::tokens(Ticker::from(
                "1234567890123456789012345678901234567890".as_bytes()
            )),
            SimpleTokenRecord {
                ticker: Ticker::from("123456789012".as_bytes()),
                total_supply,
                owner_did
            }
        );

        assert_err!(
            SimpleToken::create_token(
                owner_signed.clone(),
                Ticker::from(&[0x02][..]),
                MAX_SUPPLY + 1
            ),
            Error::TotalSupplyAboveLimit
        );
    });
}

#[test]
fn transfer_works() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (spender_signed, spender_did) = make_account(AccountKeyring::Bob.public()).unwrap();

        let ticker = Ticker::from(&[0x01][..]);
        let total_supply = 1_000_000;

        // Issuance is successful
        assert_ok!(SimpleToken::create_token(
            owner_signed.clone(),
            ticker,
            total_supply
        ));

        let gift = 1000u128;
        assert_err!(
            SimpleToken::transfer(spender_signed.clone(), ticker, owner_did, gift),
            Error::NotAnOwner
        );

        assert_ok!(SimpleToken::transfer(
            owner_signed.clone(),
            ticker,
            spender_did,
            gift
        ));
        assert_eq!(
            SimpleToken::balance_of((ticker, owner_did)),
            total_supply - gift
        );
        assert_eq!(SimpleToken::balance_of((ticker, spender_did)), gift);
    });
}

#[test]
fn approve_transfer_works() {
    ExtBuilder::default().build().execute_with(|| {
        let (owner_signed, owner_did) = make_account(AccountKeyring::Alice.public()).unwrap();
        let (spender_signed, spender_did) = make_account(AccountKeyring::Bob.public()).unwrap();
        let (agent_signed, _) = make_account(AccountKeyring::Dave.public()).unwrap();

        let ticker = Ticker::from(&[0x01][..]);
        let total_supply = 1_000_000;

        // Issuance is successful
        assert_ok!(SimpleToken::create_token(
            owner_signed.clone(),
            ticker,
            total_supply
        ));

        let allowance = 1000u128;

        assert_err!(
            SimpleToken::approve(spender_signed.clone(), ticker, spender_did, allowance),
            Error::NotAnOwner
        );

        assert_ok!(SimpleToken::approve(
            owner_signed.clone(),
            ticker,
            spender_did,
            allowance
        ));
        assert_eq!(
            SimpleToken::allowance((ticker, owner_did, spender_did)),
            allowance
        );

        assert_err!(
            SimpleToken::approve(owner_signed.clone(), ticker, spender_did, std::u128::MAX),
            Error::AllowanceOverflow
        );

        assert_err!(
            SimpleToken::transfer_from(
                spender_signed.clone(),
                ticker,
                owner_did,
                spender_did,
                allowance + 1u128
            ),
            Error::InsufficientAllowance
        );

        assert_ok!(SimpleToken::transfer_from(
            spender_signed.clone(),
            ticker,
            owner_did,
            spender_did,
            allowance
        ));
        assert_eq!(
            SimpleToken::balance_of((ticker, owner_did)),
            total_supply - allowance
        );
        assert_eq!(SimpleToken::balance_of((ticker, spender_did)), allowance);
    });
}
