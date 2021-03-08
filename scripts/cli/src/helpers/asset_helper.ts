import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, Document, TickerRegistration, IdentityId } from "../types";
import { sendTx, handle } from "../util/init";
import { assert } from "chai";

/**
 * @description Adds Documents for a given token
 * @param {ApiPromise}  api - ApiPromise
 * @param {Ticker} ticker - Ticker
 * @param {Document[]} docs - An array of Documents
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<boolean>}
 */
export async function addDocuments(
	api: ApiPromise,
	ticker: Ticker,
	docs: Document[],
	signer: KeyringPair
): Promise<boolean> {
	try {
		const transaction = api.tx.asset.addDocuments(docs, ticker);
		await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
		return true;
	} catch (err) {
		return false;
	}
}

/**
 * @description Issues a token to an Identity
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} account - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @param {number} amount - Token amount
 * @param {string} fundingRound - Funding Round
 * @return {Promise<void>}
 */
export async function issueTokenToDid(
	api: ApiPromise,
	account: KeyringPair,
	ticker: Ticker,
	amount: number,
	fundingRound?: string
): Promise<void> {
	assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");
	const [tickerDataErr, tickerData] = await handle(api.query.asset.tickers(ticker));
	if (tickerDataErr) throw new Error("Retrieving ticker failed");

	let tickerExist = (tickerData as unknown) as TickerRegistration;

	if (tickerExist.owner == 0) {
		const transaction = api.tx.asset.createAsset(ticker, ticker, amount, true, 0, [], fundingRound);
		await sendTx(account, transaction).catch((err) => console.log(`Error: ${err.message}`));
	} else {
		console.log("ticker exists already");
	}
}

/**
 * @description Mints an Asset
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} minter - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function mintingAsset(api: ApiPromise, minter: KeyringPair, ticker: Ticker): Promise<void> {
	const transaction = api.tx.asset.issue(ticker, 100);
	await sendTx(minter, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Gets the Asset balance
 * @param {ApiPromise}  api - ApiPromise
 * @param {Ticker} ticker - Ticker
 * @param {IdentityId} did - Token amount
 * @return {Promise<number>}
 */
export async function assetBalance(api: ApiPromise, ticker: Ticker, did: IdentityId): Promise<number> {
	const [balanceErr, balance] = await handle(api.query.asset.balanceOf(ticker, did));
	if (balanceErr) throw new Error("assetBalance failed");
	return (balance as unknown) as number;
}
