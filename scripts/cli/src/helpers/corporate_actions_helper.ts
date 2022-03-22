import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker } from "../types";
import { sendTx, ApiSingleton, keyToIdentityIds } from "../util/init";

export async function changeDefaultTargetIdentitites(
  signer: KeyringPair,
  ticker: Ticker,
  targets: KeyringPair[],
  treatment: "include" | "exclude"
) {
  const api = await ApiSingleton.getInstance();
  let identities = [];
  for (let id of targets) {
    identities.push((await keyToIdentityIds(id.publicKey)).toString());
  }
  const transaction = api.tx.corporateAction.setDefaultTargets(ticker, {
    identities,
    treatment,
  });
  await sendTx(signer, transaction);
}

export async function changeWithholdingTax(
  signer: KeyringPair,
  ticker: Ticker,
  amount: number
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.corporateAction.setDefaultWithholdingTax(
    ticker,
    amount
  );
  await sendTx(signer, transaction);
}

type kind =
  | "PredictableBenefit"
  | "UnpredictableBenefit"
  | "IssuerNotice"
  | "Reorganization"
  | "Other";
export async function initiateCorporateAction(
  signer: KeyringPair,
  ticker: Ticker,
  kind: kind,
  declDate: string,
  recordDate: any,
  details: string,
  defaultWithholdingTax: string | null,
  withholdingTax: string | null,
  targets: KeyringPair[] | null
) {
  const api = await ApiSingleton.getInstance();
  console.log("making checkpoint");
  const checkPointTx = api.tx.checkpoint.createCheckpoint(ticker);
  await sendTx(signer, checkPointTx);
  console.log("initing corporate action");
  const transaction = api.tx.corporateAction.initiateCorporateAction(
    ticker,
    kind,
    declDate,
    recordDate,
    details,
    defaultWithholdingTax,
    withholdingTax,
    targets
  );
  await sendTx(signer, transaction);
}

export async function linkCaToDoc(
  signer: KeyringPair,
  ticker: Ticker,
  caId: number,
  docIds: number[]
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.corporateAction.linkCaDoc(
    {
      ticker,
      localId: caId,
    },
    docIds
  );
  await sendTx(signer, transaction);
}

export async function removeCa(
  signer: KeyringPair,
  ticker: Ticker,
  caId: number
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.corporateAction.removeCa({
    ticker: ticker,
    localId: caId,
  });
  await sendTx(signer, transaction);
}

export async function recordDateChange(
  signer: KeyringPair,
  ticker: Ticker,
  caId: string,
  recordDate: string | null
) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.corporateAction.changeRecordDate(
    { ticker },
    recordDate
  );
  await sendTx(signer, transaction);
}

export async function createDistribution(
  signer: KeyringPair,
  ticker: Ticker,
  caId: string,
  portfolio: string | null,
  currency: string,
  perShare: number,
  amount: number,
  expiresAt: string | null
) {
  const api = await ApiSingleton.getInstance();
  const currentBlockTime = (await api.query.timestamp.now()).toNumber();
  const payAt = currentBlockTime;

  const transaction = api.tx.capitalDistribution.distribute(
    { ticker, localId: caId },
    portfolio,
    currency,
    perShare,
    amount,
    payAt,
    expiresAt
  );
  console.log("distributing");
  await sendTx(signer, transaction);
}

export async function pauseCompliance(signer: KeyringPair, ticker: Ticker) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.complianceManager.pauseAssetCompliance(ticker);
  await sendTx(signer, transaction);
}

export async function claimDistribution(
  signer: KeyringPair,
  ticker: Ticker,
  caId: number
) {
  const api = await ApiSingleton.getInstance();

  const transaction = api.tx.capitalDistribution.claim({
    ticker,
    localId: caId,
  });
  console.log("claiming distribution");
  await sendTx(signer, transaction);
}
