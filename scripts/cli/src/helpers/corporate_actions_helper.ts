import { Keyring } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import { CAKind, Document } from "polymesh-typegen/interfaces";
import { Ticker } from "../types";
import {
  sendTx,
  ApiSingleton,
  keyToIdentityIds,
  transferAmount,
} from "../util/init";

export async function changeDefaultTargetIdentitites(
  signer: KeyringPair,
  ticker: Ticker,
  targets: KeyringPair[],
  treatment: "include" | "exclude"
) {
  const api = await ApiSingleton.getInstance();
  const identities = targets.map(toIdentity);

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
  recordDate: string | null,
  details: string,
  defaultWithholdingTax: string | null,
  withholdingTax: string | null,
  targets: KeyringPair[] | null
) {
  const api = await ApiSingleton.getInstance();
  const checkPointTx = api.tx.checkpoint.createCheckpoint(ticker);
  await sendTx(signer, checkPointTx);
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
      local_id: caId,
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
    local_id: caId,
  });
  await sendTx(signer, transaction);
}

async function toIdentity(keyPair: KeyringPair) {
  return (await keyToIdentityIds(keyPair.publicKey)).toString();
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
