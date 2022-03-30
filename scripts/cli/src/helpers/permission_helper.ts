import type { KeyringPair } from "@polkadot/keyring/types";
import type {
  PortfolioId,
  Ticker,
  DocumentHash,
  DocumentName,
  Document,
  PalletPermissions,
  These,
  DispatchableName,
} from "../types";
import { keyToIdentityIds, ApiSingleton } from "../util/init";
import { nextPortfolioNumber } from "../helpers/portfolio_helper";

/**
 * @description Adds ticker to ticker array
 */
export function setAsset(ticker: Ticker, assetArray: These<Ticker>): void {
  assetArray.These.push(ticker);
  assetArray.These.sort((a, b) => (a > b ? 1 : -1));
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
  portfolioArray: These<PortfolioId>,
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
      portfolioArray.These.push(userPortfolio);
      break;
    default:
      let defaultPortfolio: PortfolioId = {
        did: receiverDid,
        kind: { Default: "" },
      };
      portfolioArray.These.push(defaultPortfolio);
      break;
  }
}

/**
 * @description Adds extrinsic to extrinsic array
 */
export function setExtrinsic(
  extrinsicArray: These<PalletPermissions>,
  palletName: string,
  dispatchName: string
): void {
  for (const t of extrinsicArray.These) {
    if (t.pallet_name == palletName) {
      (t.dispatchable_names as These<DispatchableName>).These.push(
        dispatchName
      );
      return;
    }
  }
  extrinsicArray.These.push({
    pallet_name: palletName,
    dispatchable_names: { These: [dispatchName] },
  });
  extrinsicArray.These.sort((a, b) => (a.pallet_name > b.pallet_name ? 1 : -1));
}
