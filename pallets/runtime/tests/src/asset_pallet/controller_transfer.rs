use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_std::collections::btree_set::BTreeSet;

use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::settlement::{
    InstructionId, Leg, SettlementType, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{
    AssetHolder, AssetHolderKind, AuthorizationData, PortfolioId, PortfolioKind, Signatory,
};

use super::setup::{create_and_issue_sample_asset, ISSUE_AMOUNT};
use crate::storage::{default_asset_holder_set, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type ExternalAgents = pallet_external_agents::Pallet<TestStorage>;
type Identity = pallet_identity::Pallet<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

fn make_full_agent(owner: &User, agent: &User, asset_id: polymesh_primitives::asset::AssetId) {
    let authorization_id = Identity::add_auth(
        owner.did,
        Signatory::from(agent.did),
        AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
        None,
    )
    .unwrap();
    assert_ok!(ExternalAgents::accept_become_agent(
        agent.origin(),
        authorization_id
    ));
}

#[test]
fn controller_transfer_locked_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let bob_default_portfolio = PortfolioId {
            did: bob.did,
            kind: PortfolioKind::Default,
        };
        let alice_default_portfolio = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::Default,
        };

        let asset_id = create_and_issue_sample_asset(&alice);
        let authorization_id = Identity::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();
        assert_ok!(ExternalAgents::accept_become_agent(
            bob.origin(),
            authorization_id
        ));
        // Lock the asset by creating and affirming an instruction
        assert_ok!(Settlement::create_venue(
            alice.origin(),
            VenueDetails::default(),
            BTreeSet::from([alice.acc()]),
            VenueType::Other
        ));
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(VenueId(0)),
            SettlementType::SettleManual(System::block_number() + 1),
            None,
            None,
            vec![Leg::Fungible {
                sender: alice_default_portfolio.clone().into(),
                receiver: bob_default_portfolio.clone().into(),
                asset_id,
                amount: ISSUE_AMOUNT,
            }],
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            InstructionId(0),
            default_asset_holder_set(alice.did),
        ),);

        // Controller transfer should fail since the tokens are locked
        assert_noop!(
            Asset::controller_transfer(
                bob.origin(),
                asset_id,
                200,
                alice_default_portfolio.into(),
                AssetHolderKind::DefaultPortfolio
            ),
            AssetError::InsufficientBalance
        );
    });
}

#[test]
fn controller_self_transfer_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        assert_noop!(
            Asset::controller_transfer(
                alice.origin(),
                asset_id,
                1000,
                alice_default_portfolio.into(),
                AssetHolderKind::DefaultPortfolio
            ),
            AssetError::SenderSameAsReceiver
        );
    });
}

#[test]
fn controller_transfer_within_free_balance_does_not_unfreeze() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let alice_holder = AssetHolder::from(alice_default_portfolio.clone());

        let asset_id = create_and_issue_sample_asset(&alice);
        make_full_agent(&alice, &bob, asset_id);

        let frozen_amount = ISSUE_AMOUNT / 2;
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            alice_holder.clone(),
            frozen_amount,
        ));

        let free_balance = ISSUE_AMOUNT - frozen_amount;
        assert_ok!(Asset::controller_transfer(
            bob.origin(),
            asset_id,
            free_balance,
            alice_default_portfolio.into(),
            AssetHolderKind::DefaultPortfolio,
        ));

        assert_eq!(
            Asset::get_holders_frozen_balance(&alice_holder, &asset_id),
            frozen_amount
        );
    });
}

#[test]
fn controller_transfer_exceeding_free_balance_unfreezes_shortfall() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);
        let alice_holder = AssetHolder::from(alice_default_portfolio.clone());

        let asset_id = create_and_issue_sample_asset(&alice);
        make_full_agent(&alice, &bob, asset_id);

        let frozen_amount = ISSUE_AMOUNT / 2;
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            alice_holder.clone(),
            frozen_amount,
        ));

        let free_balance = ISSUE_AMOUNT - frozen_amount;
        let shortfall = 100;
        let transfer_value = free_balance + shortfall;
        assert_ok!(Asset::controller_transfer(
            bob.origin(),
            asset_id,
            transfer_value,
            alice_default_portfolio.into(),
            AssetHolderKind::DefaultPortfolio,
        ));

        assert_eq!(
            Asset::get_holders_frozen_balance(&alice_holder, &asset_id),
            frozen_amount - shortfall
        );
    });
}
