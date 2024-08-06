import type { KeyringPair } from "@polkadot/keyring/types";
import type { IdentityId } from "../interfaces";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Returns the next portfolio number
 */
export async function nextPortfolioNumber(did: IdentityId): Promise<number> {
  const api = await ApiSingleton.getInstance();
  return (await api.query.portfolio.nextPortfolioNumber(did)).toNumber();
}

/**
 * @description Creates a portfolio
 */
export async function createPortfolio(
  signer: KeyringPair,
  name: string
): Promise<boolean> {
  const api = await ApiSingleton.getInstance();
  try {
    const transaction = api.tx.portfolio.createPortfolio(name);
    const r = await sendTx(signer, transaction);
    return true;
  } catch (err: unknown) {
    console.log(`Error: ${err}`);
    return false;
  }
}
