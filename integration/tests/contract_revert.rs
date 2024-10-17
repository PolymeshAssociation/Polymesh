use anyhow::Result;

use sp_weights::Weight;

use integration::*;

#[tokio::test]
async fn contract_constructor_reverted() -> Result<()> {
    let mut tester = PolymeshTester::new().await?;
    let mut users = tester.users(&["ContractRevertedDID"]).await?;

    // Upload and deploy `contract_revert` contract as a secondary key of DID1.
    let contract_revert_bytes = include_bytes!("contract_revert.wasm");
    let perms = PermissionsBuilder::whole();
    let mut res = tester
        .api
        .call()
        .polymesh_contracts()
        .instantiate_with_code_perms(
            0,
            Weight::from_parts(10_000_000_000, 0),
            None,
            contract_revert_bytes.to_vec(),
            vec![0x9b, 0xae, 0x9d, 0x5e, 0x01], // Selector for `new(true)` constructor.
            vec![0x42],                         // salt.
            perms.build(),
        )?
        .submit_and_watch(&mut users[0])
        .await?;

    let tx_res = res.extrinsic_result().await?.expect("should have results");
    assert!(tx_res.is_failed(), "The transaction should have failed.");
    Ok(())
}
