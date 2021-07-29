import type { KeyringPair } from "@polkadot/keyring/types";
import type {
  Ticker,
  ExtrinsicPermissions,
} from "../types";
import { sendTx, ApiSingleton } from "../util/init";
import type { IdentityId } from "../interfaces";

/**
 * @description Creates Group
 */
 export async function createGroup(
    signer: KeyringPair,
    ticker: Ticker,
    perms: ExtrinsicPermissions
  ): Promise<void> {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.externalAgents.createGroup(ticker, perms);
    await sendTx(signer, transaction);
  }

  export async function setGroupPermissions(
      signer: KeyringPair,
      ticker: Ticker,
      id: number,
      perms: ExtrinsicPermissions
  ) {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.externalAgents.setGroupPermissions(ticker, id, perms);
    await sendTx(signer, transaction);
  }

  // TODO: Abdicate function 

  // TODO: Remove agent function 

  // TODO: change group function 

  // TODO: add Agent function 