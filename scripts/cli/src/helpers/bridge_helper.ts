import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import { u8aToHex, numberToU8a } from "@polkadot/util";
import { sendTx } from "../util/init";

/**
 * @description Propose a Bridge Transaction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @param {KeyringPair} alice - KeyringPair
 * @return {Promise<void>}
 */
export async function bridgeTransfer(api: ApiPromise, signer: KeyringPair, alice: KeyringPair): Promise<void> {
	let amount = 10;
	let bridge_tx = {
		nonce: 2,
		recipient: alice.publicKey,
		amount,
		tx_hash: u8aToHex(numberToU8a(1), 256),
	};

	const transaction = api.tx.bridge.proposeBridgeTx(bridge_tx);
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Freeze a Bridge Transaction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function freezeTransaction(api: ApiPromise, signer: KeyringPair): Promise<void> {
	const transaction = api.tx.bridge.freeze();
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Unfreeze a Bridge Transaction
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function unfreezeTransaction(api: ApiPromise, signer: KeyringPair): Promise<void> {
	const transaction = api.tx.bridge.unfreeze();
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}
