import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker } from "../types";
import { sendTx, ApiSingleton } from "../util/init";

export async function addTransferManager(signer: KeyringPair, ticker: Ticker, countTransferManager: any ) {
    const api = await ApiSingleton.getInstance();
    const transaction = api.tx.statistics.addTransferManager(ticker, countTransferManager);
    await sendTx(signer, transaction);
}