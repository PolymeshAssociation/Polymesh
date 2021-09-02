import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces";
import type { AnyNumber } from "@polkadot/types/types";
import type {
  Permissions,
  Expiry,
  ExtrinsicPermissions,
  PortfolioPermissions,
  AssetPermissions,
  Signatory,
  AuthorizationData,
  AgentGroup,
  CddId,
} from "../types";
import { sendTx, keyToIdentityIds, ApiSingleton } from "../util/init";
import type { IdentityId, Moment } from "../interfaces";

/**
 * @description Adds a Claim to an Identity
 */
export async function addClaimsToDids(
  signer: KeyringPair,
  did: IdentityId,
  claimType: "Exempted",
  claimValue: any,
  expiry: Expiry
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  // Receieving Conditions Claim
  let claim = { [claimType]: claimValue };
  const transaction = api.tx.identity.addClaim(did, claim, expiry);
  await sendTx(signer, transaction);
}

/**
 * @description Sets permission to signer key
 */
export async function setPermissionToSigner(
  signers: KeyringPair[],
  receivers: KeyringPair[],
  extrinsic: ExtrinsicPermissions,
  portfolio: PortfolioPermissions,
  asset: AssetPermissions
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const permissions: Permissions = {
    asset,
    extrinsic,
    portfolio,
  };

  for (let i in signers) {
    let signer = {
      Account: receivers[i].publicKey as AccountId,
    };
    let transaction = api.tx.identity.setPermissionToSigner(
      signer,
      permissions
    );
    await sendTx(signers[i], transaction);
  }
}

/**
 * @description Authorizes the joining of secondary keys to a DID
 */
export async function authorizeJoinToIdentities(
  signers: KeyringPair[],
  receivers: KeyringPair[]
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  for (let i in receivers) {
    // 1. Authorize
    const auths = await api.query.identity.authorizations.entries({
      Account: signers[i].publicKey,
    });

    let last_auth_id: AnyNumber = 0;
    for (let j in auths) {
      if (auths[j][1].auth_id > last_auth_id) {
        last_auth_id = auths[j][1].auth_id;
      }
    }
    const transaction = api.tx.identity.joinIdentityAsKey(last_auth_id);
    await sendTx(signers[i], transaction);
  }
}

/**
 * @description Creates an Identity for KeyringPairs.
 */
export async function createIdentities(
  signer: KeyringPair,
  receivers: KeyringPair[]
): Promise<IdentityId[]> {
  return createIdentitiesWithExpiry(signer, receivers, []);
}

export async function createIdentitiesWithExpiry(
  signer: KeyringPair,
  receivers: KeyringPair[],
  expiries: Uint8Array[]
): Promise<IdentityId[]> {
  const api = await ApiSingleton.getInstance();
  let dids: IdentityId[] = [];

  for (let account of receivers) {
    let account_did = (await keyToIdentityIds(account.publicKey)).toString();

    if (parseInt(account_did) == 0) {
      console.log(`>>>> [Register CDD Claim] acc: ${account.address}`);
      const transaction = api.tx.identity.cddRegisterDid(account.address, []);
      await sendTx(signer, transaction);
    } else {
      console.log("Identity Already Linked.");
    }
  }
  await setDidsArray(dids, receivers);
  await addCddClaim(signer, dids, expiries);
  return dids;
}

async function setDidsArray(dids: IdentityId[], receivers: KeyringPair[]) {
  for (let i in receivers) {
    const did = await keyToIdentityIds(receivers[i].publicKey);
    dids.push(did);
    console.log(`>>>> [Get DID ] acc: ${receivers[i].address} did: ${dids[i]}`);
  }
}

async function addCddClaim(
  signer: KeyringPair,
  dids: IdentityId[],
  expiries: Uint8Array[]
) {
  const api = await ApiSingleton.getInstance();
  // Add CDD Claim with CDD_ID
  for (let i in dids) {
    const cdd_id_byte = (parseInt(i) + 1).toString(16).padStart(2, "0");
    const claim = {
      CustomerDueDiligence: `0x00000000000000000000000000000000000000000000000000000000000000${cdd_id_byte}`,
    };
    const expiry = expiries.length == 0 ? null : expiries[i];

    console.log(
      `>>>> [add CDD Claim] did: ${dids[i]}, claim: ${JSON.stringify(claim)}`
    );
    const transaction = api.tx.identity.addClaim(dids[i], claim, expiry);
    await sendTx(signer, transaction);
  }
}

export async function invalidateCddClaims(
  signer: KeyringPair,
  cdd: CddId,
  disable_from: Date,
  expiry: Expiry
) {
  const api = await ApiSingleton.getInstance();

  const transaction = api.tx.identity.invalidateCddClaims(
    cdd,
    Number(disable_from),
    expiry
  );
  await sendTx(signer, transaction);
}

export async function addAuthorization(
  signer: KeyringPair,
  receiver: Signatory,
  auth_data: any,
  expiry: Expiry
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.identity.addAuthorization(
    receiver,
    auth_data,
    expiry
  );
  await sendTx(signer, transaction);
}

export async function getAuthId() {
  const api = await ApiSingleton.getInstance();
  return api.query.identity.multiPurposeNonce();
}
