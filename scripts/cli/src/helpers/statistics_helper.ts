import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker } from "../types";
import { sendTx, ApiSingleton } from "../util/init";

export async function setActiveAssetStats(signer: KeyringPair, ticker: Ticker, stats: any ) {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.statistics.setActiveAssetStats({ Ticker: ticker }, stats);
    await sendTx(signer, transaction);
}

export async function setAssetTransferCompliance(signer: KeyringPair, ticker: Ticker, conditions: any ) {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.statistics.setAssetTransferCompliance({ Ticker: ticker }, conditions);
    await sendTx(signer, transaction);
}
