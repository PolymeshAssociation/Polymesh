import type { KeyringPair } from "@polkadot/keyring/types";
import type { PortfolioId, Ticker, MovePortfolioItem } from "../types";
import type { IdentityId } from "../interfaces";
import { sendTx, keyToIdentityIds, ApiSingleton } from "../util/init";

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

/**
 * @description Moves portfolio funds
 */
export async function movePortfolioFunds(
  signer: KeyringPair,
  primaryKey: KeyringPair,
  ticker: Ticker,
  amount: number
): Promise<boolean> {
  const api = await ApiSingleton.getInstance();
  try {
    const primaryKeyDid = await keyToIdentityIds(primaryKey.publicKey);
    const signerDid = await keyToIdentityIds(signer.publicKey);
    const portfolioNum = (await nextPortfolioNumber(signerDid)) - 1;

    const from: PortfolioId = {
      did: primaryKeyDid,
      kind: { Default: "" },
    };
    const to: PortfolioId = {
      did: signerDid,
      kind: { User: portfolioNum },
    };
    const items: MovePortfolioItem[] = [
      {
        ticker,
        amount,
      },
    ];

    const transaction = api.tx.portfolio.movePortfolioFunds(from, to, items);
    await sendTx(signer, transaction);
    return true;
  } catch (err: unknown) {
    if (err instanceof Error) {
      console.log(`Error: ${err.message}`);
    }
    return false;
  }
}
