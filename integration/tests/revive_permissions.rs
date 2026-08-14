//! Secondary key permissions are enforced for the runtime calls made through `pallet_revive`,
//! both by the precompiles and by the `RUNTIME_PALLETS_ADDR` interface for Ethereum wallets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use polymesh_api::types::polymesh_primitives::{
    authorization::AuthorizationData,
    secondary_key::Signatory,
    settlement::{VenueDetails, VenueType},
};

use integration::*;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

/// A secondary key needs permission for the extrinsic the precompile calls, not just for `Revive`.
#[tokio::test]
#[test_log::test]
async fn erc20_mint_checks_secondary_key_permissions() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users_with_secondary_keys(&[("Erc20SkPerms", 1)])
        .await?;
    let api = tester.api.clone();
    let issuer = &mut users[0];

    let (_asset, erc20) = create_erc20_asset(&api, &node, issuer, "ERC20 Perms", MINT).await?;

    // The secondary key may only use the `Revive` pallet.
    let mut perms = PermissionsBuilder::whole();
    perms.clear_extrinsic();
    perms.allow_pallet("Revive");
    issuer.set_all_keys_permissions(&perms).await?;

    {
        let sk = issuer.get_sk_mut(0)?;
        let mut caller = SubstrateCaller::new(&api, sk).await?;
        assert!(
            erc20.mint(&mut caller, 500).await.is_err(),
            "mint() should be rejected without `Asset` permissions"
        );
    }
    assert_eq!(erc20.total_supply().await?, MINT);

    // Allow the `Asset` pallet as well.
    perms.allow_pallet("Asset");
    issuer.set_all_keys_permissions(&perms).await?;

    {
        let sk = issuer.get_sk_mut(0)?;
        let mut caller = SubstrateCaller::new(&api, sk).await?;
        erc20.mint(&mut caller, 500).await?;
    }
    assert_eq!(erc20.total_supply().await?, MINT + 500);

    Ok(())
}

/// The same check applies to runtime calls an Ethereum wallet makes through `RUNTIME_PALLETS_ADDR`.
///
/// `pallet_revive` dispatches those from `eth_substrate_call`, so without the runtime's dispatch
/// hook the permission check would see `Revive.eth_substrate_call` instead of the inner call.
#[tokio::test]
#[test_log::test]
async fn substrate_call_checks_secondary_key_permissions() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester.users(&["EthSubstrateCallPerms"]).await?;
    let api = tester.api.clone();

    // An eth wallet always acts as its fallback account, so it joins the identity through the
    // very interface under test rather than by signing a substrate extrinsic.
    let wallet = node.new_wallet();
    wallet.fund(&mut tester, REVIVE_INIT_POLYX).await?;

    let mut res = api
        .call()
        .identity()
        .add_authorization(
            Signatory::Account(wallet.account()),
            AuthorizationData::JoinIdentity(PermissionsBuilder::whole().build()),
            None,
        )?
        .execute(&mut users[0])
        .await?;
    let auth_id = get_auth_id(&mut res)
        .await?
        .expect("Missing JoinIdentity auth id");

    let join = api.call().identity().join_identity_as_key(auth_id)?;
    wallet.send_runtime_call(&join).await?;

    let create_venue = api.call().settlement().create_venue(
        VenueDetails(vec![]),
        Default::default(),
        VenueType::Other,
    )?;

    // Measured while the key still has the `whole` permissions it joined with: a gas limit the
    // dry run refuses to produce later is rejected by the pool, so the call would never reach a
    // block and the execution path below would go untested.
    let gas = wallet.estimate_runtime_call(&create_venue).await? * wallet.gas_multiplier;

    // The key may only use the `Revive` pallet.
    let mut perms = PermissionsBuilder::whole();
    perms.clear_extrinsic();
    perms.allow_pallet("Revive");
    let set_perms = |perms: &PermissionsBuilder| {
        api.call()
            .identity()
            .set_secondary_key_permissions(wallet.account(), perms.build())
    };
    set_perms(&perms)?
        .execute(&mut users[0])
        .await?
        .ok()
        .await?;

    assert!(
        wallet.estimate_runtime_call(&create_venue).await.is_err(),
        "the dry run should reject create_venue without `Settlement` permissions"
    );
    let err = wallet
        .send_runtime_call_with_gas(&create_venue, gas)
        .await
        .expect_err("execution should reject create_venue without `Settlement` permissions");
    // A pool rejection is an error too, but then the call never reached the dispatch hook.
    assert!(
        err.to_string().contains("reverted"),
        "create_venue should have been included in a block and reverted, got: {err}"
    );

    // Allow the `Settlement` pallet as well.
    perms.allow_pallet("Settlement");
    set_perms(&perms)?
        .execute(&mut users[0])
        .await?
        .ok()
        .await?;

    wallet.estimate_runtime_call(&create_venue).await?;
    wallet.send_runtime_call(&create_venue).await?;

    Ok(())
}
