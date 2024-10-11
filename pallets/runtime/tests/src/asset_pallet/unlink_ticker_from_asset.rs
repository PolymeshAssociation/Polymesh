use frame_support::{assert_noop, assert_ok};
use frame_support::{StorageDoubleMap, StorageMap};
use sp_keyring::AccountKeyring;

use pallet_asset::{AssetIdTicker, TickerAssetId, TickersOwnedByUser, UniqueTickerRegistration};
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::{AuthorizationData, Signatory, Ticker};

use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type ExternalAgentsError = pallet_external_agents::Error<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;

#[test]
fn unlink_ticker_from_asset_id_successfully() {
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

        assert_ok!(Asset::unlink_ticker_from_asset_id(
            alice.origin(),
            ticker,
            asset_id
        ));

        assert_eq!(TickerAssetId::get(ticker), None);
        assert_eq!(AssetIdTicker::get(asset_id), None);
        assert_eq!(UniqueTickerRegistration::<TestStorage>::get(ticker), None);
        assert_eq!(TickersOwnedByUser::get(alice.did, ticker), false);
    });
}

#[test]
fn unlink_ticker_from_asset_id_unauthorized_agent() {
    ExtBuilder::default().build().execute_with(|| {
        let dave = User::new(AccountKeyring::Dave);
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

        assert_noop!(
            Asset::unlink_ticker_from_asset_id(dave.origin(), ticker, asset_id),
            ExternalAgentsError::UnauthorizedAgent
        );
    });
}

#[test]
fn unlink_ticker_from_asset_id_ticker_registration_not_found() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        let ticker: Ticker = Ticker::from_slice_truncated(b"TICKER");
        let ticker_1: Ticker = Ticker::from_slice_truncated(b"TICKER1");

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

        assert_noop!(
            Asset::unlink_ticker_from_asset_id(alice.origin(), ticker_1, asset_id),
            AssetError::TickerRegistrationNotFound
        );
    });
}

#[test]
fn unlink_ticker_from_asset_id_ticker_not_registered_to_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
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

        let auth_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        assert_ok!(ExternalAgents::accept_become_agent(bob.origin(), auth_id));

        assert_noop!(
            Asset::unlink_ticker_from_asset_id(bob.origin(), ticker, asset_id),
            AssetError::TickerNotRegisteredToCaller
        );
    });
}

#[test]
fn unlink_ticker_from_asset_id_ticker_not_linked() {
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
            Asset::unlink_ticker_from_asset_id(alice.origin(), ticker_1, asset_id),
            AssetError::TickerIsNotLinkedToTheAsset
        );
    });
}
