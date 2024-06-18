import type { KeyringPair } from "@polkadot/keyring/types";
import type {
  PortfolioId,
  Ticker,
  DocumentHash,
  DocumentName,
  Document,
  These,
  DispatchableName,
  PalletName,
  PalletPermissions,
} from "../types";
import { keyToIdentityIds, ApiSingleton } from "../util/init";
import { nextPortfolioNumber } from "../helpers/portfolio_helper";

/**
 * @description Adds ticker to ticker array
 */
export function setAsset(ticker: string, assets: These<Ticker>): void {
  assets.These.push(ticker);
  assets.These.sort((a, b) => (a > b ? 1 : -1));
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
 * @description Adds portfolio to portfolios
 */
export async function setPortfolio(
  portfolios: These<PortfolioId>,
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
      portfolios.These.push(userPortfolio);
      break;
    default:
      let defaultPortfolio: PortfolioId = {
        did: receiverDid,
        kind: { Default: "" },
      };
      portfolios.These.push(defaultPortfolio);
      break;
  }
}

/**
 * @description Adds extrinsic to extrinsic array
 */
export function setExtrinsic(
  extrinsics: These<[PalletName, PalletPermissions]>,
  palletName: string,
  dispatchName: string
): void {
  for (const t of extrinsics.These) {
    if (t[0] == palletName) {
      (t[1].extrinsics as These<DispatchableName>).These.push(
        dispatchName
      );
      return;
    }
  }
  extrinsics.These.push([palletName, { extrinsics: { These: [dispatchName] }}]);
  extrinsics.These.sort((a, b) => (a[0] > b[0] ? 1 : -1));
}
