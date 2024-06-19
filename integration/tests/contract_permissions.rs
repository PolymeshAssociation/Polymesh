use anyhow::Result;

use polymesh_api::{
    types::polymesh_primitives::{
        authorization::AuthorizationData,
        secondary_key::Signatory,
        settlement::{VenueDetails, VenueType},
    },
    types::polymesh_contracts::chain_extension::ExtrinsicId,
    TransactionResults,
};
use sp_weights::Weight;
use sp_core::Encode;

use integration::*;

async fn get_contract_address(res: &mut TransactionResults) -> Result<Option<AccountId>> {
    if let Some(events) = res.events().await? {
        for rec in &events.0 {
            match &rec.event {
                RuntimeEvent::Contracts(ContractsEvent::Instantiated { contract, .. }) => {
                    return Ok(Some(*contract));
                }
                _ => (),
            }
        }
    }
    Ok(None)
}

#[tokio::test]
async fn contract_as_secondary_key_change_identity() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester
        .users(&["ContractChangeDID1", "ContractChangeDID2"])
        .await?;

    // Use `sudo` to update call runtime whitelist for contracts.
    let mut sudo = tester.sudo.clone().expect("No Sudo user");
    let mut res_whitelist = tester
      .api
      .call()
      .sudo()
      .sudo(tester
        .api
        .call()
        .polymesh_contracts()
        .update_call_runtime_whitelist(vec![
          // Allow `utility.force_batch`.
          (ExtrinsicId(0x29, 0x04), true)
        ])?.into()
      )?
      .submit_and_watch(&mut sudo)
      .await?;

    // Upload and deploy `call_runtime_tester` contract as a secondary key of DID1.
    let call_runtime_bytes = include_bytes!("call_runtime_tester.wasm");
    let perms = PermissionsBuilder::whole();
    let mut res = tester
        .api
        .call()
        .polymesh_contracts()
        .instantiate_with_code_perms(
            0,
            Weight::from_parts(10_000_000_000, 0),
            None,
            call_runtime_bytes.to_vec(),
            vec![0x9b, 0xae, 0x9d, 0x5e], // Selector for `new` constructor.
            vec![0x42],                   // salt.
            perms.build(),
        )?
        .execute(&mut users[0])
        .await?;

    // Wait for the contract to be deployed and get it's address.
    let contract = get_contract_address(&mut res).await?
      .expect("Failed to deploy contract");

    // Wait for whitelist to update.
    res_whitelist.ok().await?;

    // Create JoinIdentity auth for the contract to join DID2 with no-permissions.
    let mut res = tester
        .api
        .call()
        .identity()
        .add_authorization(
            Signatory::Account(contract),
            AuthorizationData::JoinIdentity(PermissionsBuilder::empty().build()),
            None,
        )?
        .execute(&mut users[1])
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing JoinIdentity auth id");

    // Prepare `system.remark` call.
    let remark_call = tester.api.call().system().remark(vec![])?;
    // Prepare `settlement.create_venue` call.
    let create_venue_call = tester.api.call().settlement().create_venue(
        VenueDetails(vec![]),
        vec![],
        VenueType::Other,
    )?;
    // Prepare `identity.leave_identity_as_key` call.
    let leave_did1_call = tester.api.call().identity().leave_identity_as_key()?;
    // Prepare `identity.join_identity_as_key` call.
    let join_did2_call = tester.api.call().identity().join_identity_as_key(auth_id)?;

    let expected = vec![
      true,  // remark.
      true,  // create venue.
      true,  // leave did1.
      true,  // remark.
      false, // create venue.
      true,  // join did2.
      true,  // remark.
      false, // create venue.
    ];
    let batch_call = tester
      .api
      .call()
      .utility()
      .force_batch(vec![
        remark_call.runtime_call().clone(),
        create_venue_call.runtime_call().clone(),
        leave_did1_call.into(), // The secondary key should have no identity here.
        remark_call.runtime_call().clone(),
        // The key shouldn't be allowed to create venues.  Has no identity.
        create_venue_call.runtime_call().clone(),
        // The key should be allowed to join DID2.
        join_did2_call.into(), // The key is now a secondar key of DID2 with no permissions.
        remark_call.runtime_call().clone(),
        // The secondary key shouldn't be allowed to create venues.  Has no call permissions.
        create_venue_call.runtime_call().clone(),
      ])?.runtime_call().encode();
    let encoded_call = (0x6bu8, 0x1eu8, 0x9fu8, 0xe6u8, batch_call).encode();

    let mut res = tester
        .api
        .call()
        .contracts()
        .call(
            contract.into(),
            0,
            Weight::from_parts(10_000_000_000, 0),
            None,
            encoded_call,
        )?
        .execute(&mut users[0])
        .await?;
    res.ok().await?;
    let calls_ok = get_batch_results(&mut res).await?;
    assert_eq!(calls_ok, expected);

    Ok(())
}
