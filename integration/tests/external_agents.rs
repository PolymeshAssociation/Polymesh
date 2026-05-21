#[cfg(feature = "current_release")]
mod external_agents_tests {
    use std::collections::BTreeSet;

    use anyhow::Result;

    use integration::*;
    use polymesh_api::types::polymesh_primitives::{
        agent::AgentGroup,
        authorization::AuthorizationData,
        secondary_key::{ExtrinsicPermissions, Signatory},
    };

    /// Test the full external agents lifecycle: creating custom groups, adding
    /// agents, changing groups, and removing agents.
    ///
    /// Ported from `08_external_agents.ts`.
    #[tokio::test]
    #[test_log::test]
    async fn external_agents() -> Result<()> {
        let mut tester = PolymeshTester::new().await?;
        let mut users = tester
            .users(&["EAOwner", "EAAgent1", "EAAgent2"])
            .await?
            .into_iter();
        let mut owner = users.next().expect("EAOwner");
        let mut agent1 = users.next().expect("EAAgent1");
        let mut agent2 = users.next().expect("EAAgent2");

        let agent1_did = agent1.did.expect("EAAgent1 DID");
        let agent2_did = agent2.did.expect("EAAgent2 DID");

        let asset_helper = AssetHelper::new(
            &tester.api,
            &mut owner,
            "EATestAsset",
            1_000_000,
            BTreeSet::new(),
        )
        .await?;
        let asset_id = asset_helper.asset_id;

        // Create a custom agent group with whole extrinsic permissions.
        let mut res = tester
            .api
            .call()
            .external_agents()
            .create_group(asset_id, ExtrinsicPermissions::Whole)?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let ag_id = integration::get_ag_id(&mut res)
            .await?
            .expect("AGId from GroupCreated event");

        // Invite agent1 as a Full agent.
        let expiry: Option<u64> = None;
        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Identity(agent1_did),
                AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
                expiry,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let auth_id = get_auth_id(&mut res).await?.expect("auth id");

        // agent1 accepts.
        tester
            .api
            .call()
            .external_agents()
            .accept_become_agent(auth_id)?
            .submit_and_watch(&mut agent1)
            .await?
            .ok()
            .await?;

        // Invite agent2 to the custom group.
        let mut res = tester
            .api
            .call()
            .identity()
            .add_authorization(
                Signatory::Identity(agent2_did),
                AuthorizationData::BecomeAgent(asset_id, AgentGroup::Custom(ag_id.clone())),
                expiry,
            )?
            .submit_and_watch(&mut owner)
            .await?;
        res.ok().await?;
        let auth_id2 = get_auth_id(&mut res).await?.expect("auth id2");

        // agent2 accepts.
        tester
            .api
            .call()
            .external_agents()
            .accept_become_agent(auth_id2)?
            .submit_and_watch(&mut agent2)
            .await?
            .ok()
            .await?;

        // Change agent1's group to the custom group.
        tester
            .api
            .call()
            .external_agents()
            .change_group(asset_id, agent1_did, AgentGroup::Custom(ag_id.clone()))?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Update the custom group's permissions.
        tester
            .api
            .call()
            .external_agents()
            .set_group_permissions(asset_id, ag_id, ExtrinsicPermissions::Whole)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // Remove agent2.
        tester
            .api
            .call()
            .external_agents()
            .remove_agent(asset_id, agent2_did)?
            .submit_and_watch(&mut owner)
            .await?
            .ok()
            .await?;

        // agent1 abdicates (owner is still a Full agent so this is safe).
        tester
            .api
            .call()
            .external_agents()
            .abdicate(asset_id)?
            .submit_and_watch(&mut agent1)
            .await?
            .ok()
            .await?;

        Ok(())
    }
}
