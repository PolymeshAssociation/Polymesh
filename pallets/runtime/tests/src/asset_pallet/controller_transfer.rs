use frame_support::{assert_noop, assert_ok};
use sp_keyring::AccountKeyring;

use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::settlement::{
    InstructionId, Leg, SettlementType, VenueDetails, VenueId, VenueType,
};
use polymesh_primitives::{AuthorizationData, PortfolioId, PortfolioKind, Signatory};

use super::setup::{create_and_issue_sample_asset, ISSUE_AMOUNT};
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
fn controller_transfer_locked_asset() {
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
            vec![alice.acc()],
            VenueType::Other
        ));
        assert_ok!(Settlement::add_instruction(
            alice.origin(),
            Some(VenueId(0)),
            SettlementType::SettleManual(System::block_number() + 1),
            None,
            None,
            vec![Leg::Fungible {
                sender: alice_default_portfolio,
                receiver: bob_default_portfolio,
                asset_id,
                amount: ISSUE_AMOUNT,
            }],
            None,
        ));
        assert_ok!(Settlement::affirm_instruction(
            alice.origin(),
            InstructionId(0),
            vec![alice_default_portfolio]
        ),);

        // Controller transfer should fail since the tokens are locked
        assert_noop!(
            Asset::controller_transfer(bob.origin(), asset_id, 200, alice_default_portfolio),
            PortfolioError::InsufficientPortfolioBalance
        );
    });
}
