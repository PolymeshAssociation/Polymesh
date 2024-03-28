use frame_support::{assert_noop, assert_ok, StorageDoubleMap};
use sp_keyring::AccountKeyring;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioAssetBalances;
use polymesh_primitives::asset::{AssetType, NonFungibleType};
use polymesh_primitives::settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{
    PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber, Ticker, WeightMeter,
};

use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Module<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Module<TestStorage>;
type ExternalAgents = pallet_external_agents::Module<TestStorage>;
type Identity = pallet_identity::Module<TestStorage>;
type Portfolio = pallet_portfolio::Module<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Settlement = pallet_settlement::Module<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

#[test]
fn base_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_user_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        assert_ok!(Portfolio::create_portfolio(
            bob.origin(),
            PortfolioName(b"BobUserPortfolio".to_vec())
        ));
        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::Default
        ),);
        assert_ok!(ComplianceManager::pause_asset_compliance(
            alice.origin(),
            ticker
        ),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_ok!(Asset::base_transfer(
            alice_default_portfolio,
            bob_user_portfolio,
            &ticker,
            1_000,
            None,
            None,
            alice.did,
            &mut weight_meter
        ),);

        assert_eq!(BalanceOf::get(&ticker, &alice_default_portfolio.did), 0);
        assert_eq!(
            PortfolioAssetBalances::get(&alice_default_portfolio, &ticker),
            0
        );
        assert_eq!(BalanceOf::get(&ticker, &bob_user_portfolio.did), 1_000);
        assert_eq!(
            PortfolioAssetBalances::get(&bob_user_portfolio, &ticker),
            1_000
        );
    })
}

#[test]
fn base_transfer_invalid_token_type() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new(),
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                &ticker,
                1,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            AssetError::UnexpectedNonFungibleToken
        );
    })
}

#[test]
fn base_transfer_invalid_granularity() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            false,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                &ticker,
                1,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            AssetError::InvalidGranularity
        );
    })
}

#[test]
fn base_transfer_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                &ticker,
                1,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            AssetError::InsufficientBalance
        );
    })
}

#[test]
fn base_transfer_locked_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::Default
        ),);
        // Lock the asset by creating and affirming an instruction
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            vec![alice.acc()],
            VenueType::Other
        ));
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            VenueId(0),
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: alice_default_portfolio,
                receiver: bob_default_portfolio,
                ticker: ticker,
                amount: 1_000,
            }],
            vec![alice_default_portfolio],
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                &ticker,
                1,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            PortfolioError::InsufficientPortfolioBalance
        );
    })
}

#[test]
fn base_transfer_invalid_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_user_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::Default
        ),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                &ticker,
                1_000,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            PortfolioError::PortfolioDoesNotExist
        );
    })
}

#[test]
fn base_transfer_invalid_compliance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_user_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        assert_ok!(Portfolio::create_portfolio(
            bob.origin(),
            PortfolioName(b"BobUserPortfolio".to_vec())
        ));
        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::Default
        ),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                &ticker,
                1_000,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            AssetError::InvalidTransferComplianceFailure
        );
    })
}

#[test]
fn base_transfer_frozen_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let ticker = Ticker::from_slice_truncated(b"TICKER");
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_user_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        assert_ok!(Portfolio::create_portfolio(
            bob.origin(),
            PortfolioName(b"BobUserPortfolio".to_vec())
        ));
        assert_ok!(Asset::create_asset(
            alice.origin(),
            ticker.as_ref().into(),
            ticker,
            true,
            AssetType::Derivative,
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            ticker,
            1_000,
            PortfolioKind::Default
        ),);
        assert_ok!(Asset::freeze(alice.origin(), ticker,),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                &ticker,
                1_000,
                None,
                None,
                alice.did,
                &mut weight_meter
            ),
            AssetError::InvalidTransferFrozenAsset
        );
    })
}
