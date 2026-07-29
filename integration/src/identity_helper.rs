//! Identity onboarding helpers.
//!
//! `PolymeshTester` creates identities for its own sr25519 users, but the Revive
//! tests also need identities for accounts that can't sign Substrate extrinsics:
//! Ethereum wallets (which sign secp256k1 transactions) and contract accounts.
//! Those have to be onboarded by a DID registrar.
//!
//! Polymesh v8 removed CDD claims, so onboarding uses `identity.register_did`
//! instead of the deprecated `identity.cdd_register_did*` calls.

use anyhow::Result;

use crate::*;

/// Returns the identity linked to `account`, if any.
pub async fn get_did(api: &Api, account: AccountId) -> Result<Option<IdentityId>> {
    let did = match api.query().identity().key_records(account).await? {
        Some(KeyRecord::PrimaryKey(did)) | Some(KeyRecord::SecondaryKey(did)) => Some(did),
        _ => None,
    };
    Ok(did)
}

/// Registers a new identity with `account` as its primary key.
///
/// This is a no-op if `account` is already linked to an identity, so it is safe
/// to call for accounts that may have been onboarded by an earlier test.
///
/// The call is signed by the tester's DID registrar (Alice on a `--dev` chain).
pub async fn register_did(tester: &mut PolymeshTester, account: AccountId) -> Result<IdentityId> {
    if let Some(did) = get_did(&tester.api, account).await? {
        return Ok(did);
    }

    let api = tester.api.clone();
    let mut res = api
        .call()
        .identity()
        .register_did(account)?
        .execute(&mut tester.cdd)
        .await?;

    get_identity_id(&mut res)
        .await?
        .ok_or_else(|| anyhow::anyhow!("no identity created for {account:?}"))
}

/// Gives `account` some POLYX so it can pay transaction fees and storage deposits.
///
/// Uses `sudo` when the chain has it (the `--dev` chain), and falls back to a
/// plain transfer from the registrar account otherwise (the CI chain).
pub async fn fund_polyx(
    tester: &mut PolymeshTester,
    account: AccountId,
    polyx: u128,
) -> Result<()> {
    let api = tester.api.clone();
    let amount = polyx * ONE_POLYX;

    let mut res = match tester.sudo.as_mut() {
        Some(sudo) => {
            let call = api
                .call()
                .balances()
                .force_set_balance(account.into(), amount)?;
            api.call().sudo().sudo(call.into())?.execute(sudo).await?
        }
        None => {
            api.call()
                .balances()
                .transfer_with_memo(account.into(), amount, None)?
                .execute(&mut tester.cdd)
                .await?
        }
    };
    res.ok().await?;
    Ok(())
}

/// Registers an identity for `account` and funds it with `polyx` POLYX.
///
/// Both steps are idempotent, so this can be called for accounts that are
/// already onboarded.
pub async fn onboard_account(
    tester: &mut PolymeshTester,
    account: AccountId,
    polyx: u128,
) -> Result<IdentityId> {
    let did = register_did(tester, account).await?;
    fund_polyx(tester, account, polyx).await?;
    Ok(did)
}
