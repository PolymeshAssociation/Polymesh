import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces";
import type { Expiry, Permissions } from "../types";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Attaches a secondary key to each DID
 */
export async function addSecondaryKeys(
  signers: KeyringPair[],
  receivers: KeyringPair[]
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  let totalPermissions: Permissions = {
    asset: { These: [] },
    extrinsic: { These: new Map() },
    portfolio: { These: [] },
  };

  for (let i in signers) {
    let target = {
      Account: receivers[i].publicKey as AccountId,
    };
    let authData = {
      JoinIdentity: totalPermissions,
    };
    let expiry: Expiry = null;
    // 1. Add Secondary Item to identity.
    const transaction = api.tx.identity.addAuthorization(
      target,
      authData,
      expiry
    );
    await sendTx(signers[i], transaction);
  }
}

/**
 * @description Attaches a secondary key to each DID
 */
export async function createMultiSig(
  signer: KeyringPair,
  signatories: AccountId[],
  numOfSigners: number
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.multiSig.createMultisig(signatories, numOfSigners, null);
  await sendTx(signer, transaction);
}
