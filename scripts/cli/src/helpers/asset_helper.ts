import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, Document } from "../types";
import { sendTx, ApiSingleton } from "../util/init";
import { assert } from "chai";
import type { IdentityId } from "../interfaces";

/**
 * @description Adds Documents for a given token
 * @param {Ticker} ticker - Ticker
 * @param {Document[]} docs - An array of Documents
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<boolean>}
 */
export async function addDocuments(ticker: Ticker, docs: Document[], signer: KeyringPair): Promise<boolean> {
	try {
		const api = await ApiSingleton.getInstance();
		const transaction = api.tx.asset.addDocuments(docs, ticker);
		await sendTx(signer, transaction);
		return true;
	} catch (err) {
		console.log(`Error: ${err.message}`);
		return false;
	}
}

/**
 * @description Issues a token to an Identity
 * @param {KeyringPair} account - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @param {number} amount - Token amount
 * @param {string} fundingRound - Funding Round
 * @return {Promise<void>}
 */
export async function issueTokenToDid(
	account: KeyringPair,
	ticker: Ticker,
	amount: number,
	fundingRound: string | null
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	assert(ticker.length <= 12, "Ticker cannot be longer than 12 characters");
	const tickerData = await api.query.asset.tickers(ticker);

	if (tickerData.owner) {
		const transaction = api.tx.asset.createAsset(
			ticker,
			ticker,
			amount,
			true,
			{ EquityCommon: "" },
			[],
			fundingRound
		);
		await sendTx(account, transaction).catch((err) => console.log(`Error: ${err.message}`));
	} else {
		console.log("ticker exists already");
	}
}

/**
 * @description Mints an Asset
 * @param {KeyringPair} minter - KeyringPair
 * @param {Ticker} ticker - Ticker
 * @return {Promise<void>}
 */
export async function mintingAsset(minter: KeyringPair, ticker: Ticker): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.asset.issue(ticker, 100);
	await sendTx(minter, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Gets the Asset balance
 * @param {Ticker} ticker - Ticker
 * @param {IdentityId} did - Token amount
 * @return {Promise<number>}
 */
export async function assetBalance(ticker: Ticker, did: IdentityId): Promise<number> {
	const api = await ApiSingleton.getInstance();
	const balance = (await api.query.asset.balanceOf(ticker, did)).toNumber();
	return balance;
}
