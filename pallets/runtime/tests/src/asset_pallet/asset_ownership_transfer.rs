use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_asset::SecurityTokensOwnedByUser;
use pallet_external_agents::GroupOfAgent;
use polymesh_primitives::agent::AgentGroup;
use polymesh_primitives::{AuthorizationData, Signatory};

use crate::asset_pallet::setup::create_and_issue_sample_asset;
use crate::ext_builder::ExtBuilder;
use crate::storage::{TestStorage, User};

#[test]
fn transfer_token_ownership_agents_check() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);

        let agent_auth_id = pallet_identity::Pallet::<TestStorage>::add_auth(
            alice.did,
            Signatory::from(dave.did),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
            None,
        )
        .unwrap();

        assert_ok!(
            pallet_external_agents::Pallet::<TestStorage>::accept_become_agent(
                dave.origin(),
                agent_auth_id
            )
        );

        let asset_transfer_auth_id = pallet_identity::Pallet::<TestStorage>::add_auth(
            alice.did,
            Signatory::from(bob.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();

        assert_ok!(
            pallet_asset::Pallet::<TestStorage>::accept_asset_ownership_transfer(
                bob.origin(),
                asset_transfer_auth_id
            )
        );

        // Asset owner is updated
        assert_eq!(
            SecurityTokensOwnedByUser::<TestStorage>::get(alice.did, asset_id),
            false
        );
        assert_eq!(
            SecurityTokensOwnedByUser::<TestStorage>::get(bob.did, asset_id),
            true
        );
        // Old asset owner is removed from external agents
        assert_eq!(GroupOfAgent::<TestStorage>::get(asset_id, alice.did), None);
        // New asset owner is added from external agents
        assert_eq!(
            GroupOfAgent::<TestStorage>::get(asset_id, bob.did),
            Some(AgentGroup::Full)
        );
        // Previous permissioned agent remains
        assert_eq!(
            GroupOfAgent::<TestStorage>::get(asset_id, dave.did),
            Some(AgentGroup::Full)
        );
    })
}

#[test]
fn block_self_ownership_transfers() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);

        let asset_transfer_auth_id = pallet_identity::Pallet::<TestStorage>::add_auth(
            alice.did,
            Signatory::from(alice.did),
            AuthorizationData::TransferAssetOwnership(asset_id),
            None,
        )
        .unwrap();

        assert_noop!(
            pallet_asset::Pallet::<TestStorage>::accept_asset_ownership_transfer(
                alice.origin(),
                asset_transfer_auth_id
            ),
            pallet_asset::Error::<TestStorage>::SelfOwnershipTransferNotAllowed
        );
    })
}
