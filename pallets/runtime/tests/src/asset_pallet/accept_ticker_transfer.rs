use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap, StorageValue};
use sp_keyring::AccountKeyring;

use pallet_asset::{
    AssetOwnershipRelation, AssetOwnershipRelations, TickerConfig, TickerRegistration, Tickers,
};
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::{AuthorizationData, Signatory, Ticker};

use crate::asset_test::{now, set_time_to_now};
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;

#[test]
fn accept_ticker_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        let auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        )
        .unwrap();
        assert_ok!(Asset::accept_ticker_transfer(bob.origin(), auth_id,),);

        let ticker_registration_config = TickerConfig::<TestStorage>::get();
        assert_eq!(
            AssetOwnershipRelations::get(bob.did, ticker),
            AssetOwnershipRelation::TickerOwned
        );
        assert_eq!(
            AssetOwnershipRelations::get(alice.did, ticker),
            AssetOwnershipRelation::NotOwned
        );
        assert_eq!(
            Tickers::<TestStorage>::get(ticker).unwrap(),
            TickerRegistration {
                owner: bob.did,
                expiry: ticker_registration_config.registration_length
            }
        );
    });
}

#[test]
fn accept_ticker_transfer_missing_auth() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), 1,),
            "Authorization does not exist"
        );
    });
}

#[test]
fn accept_ticker_transfer_asset_exists() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        let auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        )
        .unwrap();
        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), auth_id,),
            AssetError::AssetAlreadyCreated
        );
    });
}

#[test]
fn accept_ticker_transfer_auth_expired() {
    ExtBuilder::default().build().execute_with(|| {
        set_time_to_now();
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        let bob_auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            Some(now() - 1000),
        )
        .unwrap();
        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), bob_auth_id,),
            "Authorization expired"
        );
    });
}

//#[test]
//fn accept_ticker_transfer_registration_expired() {
//    ExtBuilder::default().build().execute_with(|| {
//        set_time_to_now();
//        let bob = User::new(AccountKeyring::Bob);
//        let alice = User::new(AccountKeyring::Alice);
//        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
//
//        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
//        let auth_id = Identity::add_auth(
//            alice.did,
//            Signatory::from(bob.did),
//            AuthorizationData::TransferTicker(ticker),
//            None,
//        );
//        set_timestamp(now() + 10001);
//        assert_noop!(
//            Asset::accept_ticker_transfer(bob.origin(), auth_id,),
//            AssetError::TickerRegistrationExpired
//        );
//    });
//}

#[test]
fn accept_ticker_transfer_illegal_auth() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        let bob_auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        )
        .unwrap();
        let dave_auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(dave.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        )
        .unwrap();
        assert_ok!(Asset::accept_ticker_transfer(bob.origin(), bob_auth_id,),);
        assert_noop!(
            Asset::accept_ticker_transfer(dave.origin(), dave_auth_id,),
            "Illegal use of Authorization"
        );
    });
}

#[test]
fn accept_ticker_transfer_bad_type() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_ticker(alice.origin(), ticker,));
        let bob_auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::RotatePrimaryKey,
            None,
        )
        .unwrap();
        Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferTicker(ticker),
            None,
        )
        .unwrap();
        assert_noop!(
            Asset::accept_ticker_transfer(bob.origin(), bob_auth_id,),
            "Authorization type is wrong"
        );
    });
}
