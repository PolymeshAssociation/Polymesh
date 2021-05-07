import type { KeyringPair } from "@polkadot/keyring/types";
import type { PortfolioId, Ticker, DocumentHash, DocumentName, Document, LegacyPalletPermissions } from "../types";
import { keyToIdentityIds, ApiSingleton } from "../util/init";
import { nextPortfolioNumber } from "../helpers/portfolio_helper";

/**
 * @description Adds ticker to ticker array
 */
export function setAsset(ticker: Ticker, assetArray: Ticker[]): void {
	assetArray.push(ticker);
}

/**
 * @description Adds document to document array
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
 * @description Adds portfolio to portfolioArray
 */
export async function setPortfolio(
	portfolioArray: PortfolioId[],
	receiver: KeyringPair,
	type: "Default" | "User"
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	let receiverDid = await keyToIdentityIds(receiver.publicKey);

	switch (type) {
		case "User":
			const portfolioNum = (await nextPortfolioNumber(receiverDid)) - 1;
			let userPortfolio: PortfolioId = {
				did: receiverDid,
				kind: { User: portfolioNum },
			};
			portfolioArray.push(userPortfolio);
			break;
		default:
			let defaultPortfolio: PortfolioId = {
				did: receiverDid,
				kind: { Default: "" },
			};
			portfolioArray.push(defaultPortfolio);
			break;
	}
}

/**
 * @description Adds extrinsic to extrinsic array
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
