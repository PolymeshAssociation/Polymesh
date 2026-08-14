//! Secondary key permissions are enforced for the runtime calls made by the precompiles.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

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
