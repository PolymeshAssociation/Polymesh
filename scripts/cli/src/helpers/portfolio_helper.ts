import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { IdentityId, PortfolioId, Ticker, MovePortfolioItem, PortfolioNumber } from "../types";
import { sendTx, keyToIdentityIds } from "../util/init";

/**
 * @description Returns the next portfolio number
 * @param {ApiPromise}  api - ApiPromise
 * @param {IdentityId} did - IdentityId
 * @return {Promise<number>}
 */
export async function nextPortfolioNumber(api: ApiPromise, did: IdentityId): Promise<number> {
	return ((await api.query.portfolio
		.nextPortfolioNumber(did)
		.catch((err) => console.log(`Error: ${err.message}`))) as unknown) as number;
}

/**
 * @description Creates a portfolio
 * @param {ApiPromise}  api - ApiPromise
 * @param {string} name - Name of portfolio
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<boolean>}
 */
export async function createPortfolio(api: ApiPromise, name: string, signer: KeyringPair): Promise<boolean> {
	try {
		const transaction = api.tx.portfolio.createPortfolio(name);
		await sendTx(signer, transaction);
		return true;
	} catch (err) {
		console.log(`Error: ${err.message}`)
		return false;
	}
}

/**
 * @description Moves portfolio funds
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} primaryKey - KeyringPair
 * @param {KeyringPair} secondaryKey - KeyringPair
 * @param {Ticker} ticker - Ticker to be moved
 * @param {number} amount - Amount to move
 * @return {Promise<boolean>}
 */
export async function movePortfolioFunds(
	api: ApiPromise,
	primary_key: KeyringPair,
	secondary_key: KeyringPair,
	ticker: Ticker,
	amount: number
): Promise<boolean> {
	try {
		const primaryKeyDid = await keyToIdentityIds(api, primary_key.publicKey);
		const secondaryKeyDid = await keyToIdentityIds(api, secondary_key.publicKey);
		const portfolioNum: PortfolioNumber = (await nextPortfolioNumber(api, secondaryKeyDid)) - 1;

		const from: PortfolioId = {
			did: primaryKeyDid,
			kind: { Default: "" },
		};
		const to: PortfolioId = {
			did: secondaryKeyDid,
			kind: { User: portfolioNum },
		};
		const items: MovePortfolioItem[] = [
			{
				ticker,
				amount,
			},
		];

		const transaction = api.tx.portfolio.movePortfolioFunds(from, to, items);
		await sendTx(secondary_key, transaction);
		return true;
	} catch (err) {
		console.log(`Error: ${err.message}`);
		return false;
	}
}
