import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, Document } from "../types";
import type { IdentityId } from "../interfaces";
import { sendTx, ApiSingleton } from "../util/init";
import { assert } from "chai";

/**
 * @description Adds Documents for a given token
 */
export async function addDocuments(
  signer: KeyringPair,
  ticker: Ticker,
  docs: Document[]
): Promise<boolean> {
  try {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.asset.addDocuments(docs, ticker);
    await sendTx(signer, transaction);
    return true;
  } catch (err: unknown) {
    if (err instanceof Error) {
      console.log(`Error: ${err.message}`);
    }
    return false;
  }
}

/**
 * @description Issues a token to an Identity
 */
export async function issueTokenToDid(
  signer: KeyringPair,
  ticker: Ticker,
  amount: number,
  fundingRound: string | null
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");
  const tickerData = await api.query.asset.tickers(ticker);

  if (tickerData.isEmpty) {
    const createTx = api.tx.asset.createAsset(
      ticker,
      ticker,
      true,
      { EquityCommon: "" },
      [],
      fundingRound,
    );
    await sendTx(signer, createTx);
    const issueTx = api.tx.asset.issue(ticker, amount, { Default: "" });
    await sendTx(signer, issueTx);
  } else {
    console.log("ticker already reserved");
  }
}

/**
 * @description Mints an Asset
 */
export async function mintingAsset(
  signer: KeyringPair,
  ticker: Ticker
): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.asset.issue(ticker, 100, { Default: "" });
  await sendTx(signer, transaction);
}

/**
 * @description Gets the Asset balance
 */
export async function assetBalance(
  ticker: Ticker,
  did: IdentityId
): Promise<number> {
  const api = await ApiSingleton.getInstance();
  const balance = (await api.query.asset.balanceOf(ticker, did)).toNumber();
  return balance;
}
