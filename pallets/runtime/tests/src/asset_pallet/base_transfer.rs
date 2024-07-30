use frame_support::{assert_noop, assert_ok, StorageDoubleMap};
use sp_keyring::AccountKeyring;

use pallet_asset::BalanceOf;
use pallet_portfolio::PortfolioAssetBalances;
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::settlement::{Leg, SettlementType, VenueDetails, VenueId, VenueType};
use polymesh_primitives::{
    Claim, ClaimType, Condition, ConditionType, CountryCode, PortfolioId, PortfolioKind,
    PortfolioName, PortfolioNumber, Scope, TrustedFor, TrustedIssuer, WeightMeter,
};

use super::setup::{create_and_issue_sample_asset, create_and_issue_sample_nft, ISSUE_AMOUNT};
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
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(ComplianceManager::pause_asset_compliance(
            alice.origin(),
            asset_id
        ),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_ok!(Asset::base_transfer(
            alice_default_portfolio,
            bob_user_portfolio,
            asset_id,
            ISSUE_AMOUNT,
            None,
            None,
            alice.did,
            &mut weight_meter
        ),);

        assert_eq!(BalanceOf::get(&asset_id, &alice_default_portfolio.did), 0);
        assert_eq!(
            PortfolioAssetBalances::get(&alice_default_portfolio, &asset_id),
            0
        );
        assert_eq!(
            BalanceOf::get(&asset_id, &bob_user_portfolio.did),
            ISSUE_AMOUNT
        );
        assert_eq!(
            PortfolioAssetBalances::get(&bob_user_portfolio, &asset_id),
            ISSUE_AMOUNT
        );
    })
}

#[test]
fn base_transfer_invalid_token_type() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let alice = User::new(AccountKeyring::Alice);
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = create_and_issue_sample_nft(&alice);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                asset_id,
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
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = Asset::generate_asset_id(alice.did, false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            false,
            AssetType::default(),
            Vec::new(),
            None,
        ));

        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            ISSUE_AMOUNT,
            PortfolioKind::Default
        ));

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                asset_id,
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
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = Asset::generate_asset_id(alice.did, false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                asset_id,
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
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = create_and_issue_sample_asset(&alice);
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
                asset_id,
                amount: ISSUE_AMOUNT,
            }],
            vec![alice_default_portfolio],
            None,
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_default_portfolio,
                asset_id,
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
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };
        let bob_user_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        let asset_id = create_and_issue_sample_asset(&alice);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                asset_id,
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
        let dave: User = User::new(AccountKeyring::Dave);
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
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(ComplianceManager::add_compliance_requirement(
            alice.origin(),
            asset_id,
            Vec::new(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(alice.did)
                )),
                issuers: vec![TrustedIssuer {
                    issuer: dave.did,
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction])
                }]
            }],
        ));
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                asset_id,
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
        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::freeze(alice.origin(), asset_id,),);
        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        assert_noop!(
            Asset::base_transfer(
                alice_default_portfolio,
                bob_user_portfolio,
                asset_id,
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
