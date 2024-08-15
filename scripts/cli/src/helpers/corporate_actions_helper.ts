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
    null,
  );
  await sendTx(signer, transaction);
}
