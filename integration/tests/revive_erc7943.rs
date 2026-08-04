//! The ERC-20 precompile interface for Polymesh native assets.
//!
//! Requires a running `eth-rpc` node, see `ETH_RPC_URL`.

// The Revive pallet was added in v8.
#![cfg(feature = "current_release")]

use anyhow::Result;

use integration::*;
use polymesh_api::types::polymesh_primitives::condition::{Condition, ConditionType};
use polymesh_api::types::polymesh_primitives::condition::{TrustedFor, TrustedIssuer};
use polymesh_api::types::polymesh_primitives::identity_claim::{Claim, ClaimType, Scope};
use polymesh_api::types::polymesh_primitives::jurisdiction::CountryCode;

/// Initial supply issued to the asset owner's account.
const MINT: u128 = 1_000_000;

/// Can transfer returns false after receiver fails compliance check.
#[tokio::test]
#[test_log::test]
async fn erc7943_can_transfer_receiver_fails_compliance() -> Result<()> {
    let (mut tester, node) = revive_tester().await?;
    let mut users = tester
        .users(&["Erc7943SubIssuer", "Erc7943SubHolder"])
        .await?;
    let api = tester.api.clone();
    let (issuers, holders) = users.split_at_mut(1);
    let issuer = &mut issuers[0];
    let holder = &mut holders[0];

    let (asset_helper, erc7943) =
        create_erc20_asset(&api, &node, issuer, "ERC7943 Sub", MINT).await?;
    erc7943
        .api
        .call()
        .compliance_manager()
        .add_compliance_requirement(
            asset_helper.asset_id,
            Default::default(),
            vec![Condition {
                condition_type: ConditionType::IsPresent(Claim::Jurisdiction(
                    CountryCode::BR,
                    Scope::Identity(issuer.did.unwrap()),
                )),
                issuers: vec![TrustedIssuer {
                    issuer: issuer.did.unwrap(),
                    trusted_for: TrustedFor::Specific(vec![ClaimType::Jurisdiction]),
                }],
            }],
        )?
        .execute(issuer)
        .await?;

    let issuer_address = eth_address_of(&api, issuer).await?;
    let holder_address = eth_address_of(&api, holder).await?;

    assert!(!erc7943
        .can_transfer(issuer_address, holder_address, 1_000)
        .await
        .unwrap());

    Ok(())
}
