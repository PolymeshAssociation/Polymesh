import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { PortfolioId, Ticker, DocumentHash, DocumentName, Document, LegacyPalletPermissions } from "../types";
import { keyToIdentityIds } from "../util/init";
import { nextPortfolioNumber } from "../helpers/portfolio_helper";

/**
 * @description Adds ticker to ticker array
 * @param {Ticker} ticker - Ticker
 * @param {Ticker[]} assetArray - An array of Tickers
 * @return {void}
 */
export function setAsset(ticker: Ticker, assetArray: Ticker[]): void {
	assetArray.push(ticker);
}

/**
 * @description Adds document to document array
 * @param {Document[]}  docArray - An array of Documents
 * @param {string} uri - URI
 * @param {DocumentHash} contentHash - Hash type
 * @param {DocumentName} name - Name of the Document
 * @param {string=} docType - Type of Document
 * @param {number=} filingDate - number
 * @return {void}
 */
export function setDoc(
	docArray: Document[],
	uri: string,
	contentHash: DocumentHash,
	name: DocumentName,
	docType?: string,
	filingDate?: number
): void {
	docArray.push({
		uri,
		content_hash: contentHash,
		name,
		doc_type: docType,
		filing_date: filingDate,
	});
}

/**
 * @description Adds ticker to ticker array
 * @param {ApiPromise} api - Ticker
 * @param {PortfolioId[]} portfolioArray - An array of PortfolioIds
 * @param {KeyringPair} key - KeyringPair
 * @param {"Default" | "User"} type - Type of Portfolio
 * @return {Promise<void>}
 */
export async function setPortfolio(
	api: ApiPromise,
	portfolioArray: PortfolioId[],
	key: KeyringPair,
	type: "Default" | "User"
): Promise<void> {
	let keyDid = await keyToIdentityIds(api, key.publicKey);

	switch (type) {
		case "User":
			const portfolioNum = (await nextPortfolioNumber(api, keyDid)) - 1;
			let userPortfolio: PortfolioId = {
				did: keyDid,
				kind: { User: portfolioNum },
			};
			portfolioArray.push(userPortfolio);
			break;
		default:
			let defaultPortfolio: PortfolioId = {
				did: keyDid,
				kind: { Default: "" },
			};
			portfolioArray.push(defaultPortfolio);
			break;
	}
}

/**
 * @description Adds extrinsic to extrinsic array
 * @param {LegacyPalletPermissions[]} extrinsicArray - An array of LegacyPalletPermissions
 * @param {string} palletName - The name of the Pallet
 * @param {string} dispatchName - the name of the dispatchable
 * @return {void}
 */
export function setExtrinsic(
	extrinsicArray: LegacyPalletPermissions[],
	palletName: string,
	dispatchName: string
): void {
	extrinsicArray.push({
		pallet_name: palletName,
		total: false,
		dispatchable_names: [dispatchName],
	});
}
