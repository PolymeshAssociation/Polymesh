use frame_support::StorageMap;
use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring;

use pallet_asset::{AssetIdTicker, TickerAssetId};
use polymesh_primitives::Ticker;

use crate::asset_test::{now, set_timestamp};
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type ExternalAgentsError = pallet_external_agents::Error<TestStorage>;

#[test]
fn link_ticker_to_asset_id_successfully() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_eq!(TickerAssetId::get(ticker), Some(asset_id));
        assert_eq!(AssetIdTicker::get(asset_id), Some(ticker));
    });
}

#[test]
fn link_ticker_to_asset_id_ticker_not_registered_to_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(dave.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        assert_ok!(Asset::create_asset(
            dave.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::link_ticker_to_asset_id(dave.origin(), ticker, asset_id),
            AssetError::TickerNotRegisteredToCaller
        );
    });
}

#[test]
fn link_ticker_to_asset_id_ticker_unauthorized_agent() {
    ExtBuilder::default().build().execute_with(|| {
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(dave.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        assert_ok!(Asset::create_asset(
            dave.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::link_ticker_to_asset_id(alice.origin(), ticker, asset_id),
            ExternalAgentsError::UnauthorizedAgent
        );
    });
}

#[test]
fn link_ticker_to_asset_id_expired_ticker() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        set_timestamp(now() + 10001);
        assert_noop!(
            Asset::link_ticker_to_asset_id(alice.origin(), ticker, asset_id),
            AssetError::TickerRegistrationExpired
        );
    });
}

#[test]
fn link_ticker_to_asset_id_ticker_already_linked() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_noop!(
            Asset::link_ticker_to_asset_id(alice.origin(), ticker, asset_id),
            AssetError::TickerIsAlreadyLinkedToAnAsset
        );
    });
}

#[test]
fn link_ticker_to_asset_asset_already_linked() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let ticker_1: Ticker = Ticker::from_slice_truncated(b"TICKER1");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker_1,));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_noop!(
            Asset::link_ticker_to_asset_id(alice.origin(), ticker_1, asset_id),
            AssetError::AssetIsAlreadyLinkedToATicker
        );
    });
}

#[test]
fn link_ticker_to_asset_id_after_unlink() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let ticker_1: Ticker = Ticker::from_slice_truncated(b"TICKER1");

        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker,));
        assert_ok!(Asset::register_unique_ticker(alice.origin(), ticker_1,));

        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_ok!(Asset::unlink_ticker_from_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_ok!(Asset::link_ticker_to_asset_id(
            alice.origin(),
            ticker_1,
            asset_id
        ));

        assert_eq!(TickerAssetId::get(ticker_1), Some(asset_id));
        assert_eq!(AssetIdTicker::get(asset_id), Some(ticker_1));
    });
}
