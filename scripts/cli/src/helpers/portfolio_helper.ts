import type { KeyringPair } from "@polkadot/keyring/types";
import type { PortfolioId, Ticker, MovePortfolioItem } from "../types";
import { sendTx, keyToIdentityIds, ApiSingleton } from "../util/init";
import type { IdentityId } from "../interfaces";

/**
 * @description Returns the next portfolio number
 * @param {IdentityId} did - IdentityId
 * @return {Promise<number>}
 */
export async function nextPortfolioNumber(did: IdentityId): Promise<number> {
	const api = await ApiSingleton.getInstance();
	return (await api.query.portfolio.nextPortfolioNumber(did)).toNumber();
}

/**
 * @description Creates a portfolio
 * @param {string} name - Name of portfolio
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<boolean>}
 */
export async function createPortfolio(name: string, signer: KeyringPair): Promise<boolean> {
	const api = await ApiSingleton.getInstance();
	try {
		const transaction = api.tx.portfolio.createPortfolio(name);
		await sendTx(signer, transaction);
		return true;
	} catch (err) {
		console.log(`Error: ${err.message}`);
		return false;
	}
}

/**
 * @description Moves portfolio funds
 * @param {KeyringPair} primaryKey - KeyringPair
 * @param {KeyringPair} secondaryKey - KeyringPair
 * @param {Ticker} ticker - Ticker to be moved
 * @param {number} amount - Amount to move
 * @return {Promise<boolean>}
 */
export async function movePortfolioFunds(
	primary_key: KeyringPair,
	secondary_key: KeyringPair,
	ticker: Ticker,
	amount: number
): Promise<boolean> {
	const api = await ApiSingleton.getInstance();
	try {
		const primaryKeyDid = await keyToIdentityIds(primary_key.publicKey);
		const secondaryKeyDid = await keyToIdentityIds(secondary_key.publicKey);
		const portfolioNum = (await nextPortfolioNumber(secondaryKeyDid)) - 1;

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
