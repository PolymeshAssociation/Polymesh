import type { KeyringPair } from "@polkadot/keyring/types";
import { u8aToHex, numberToU8a } from "@polkadot/util";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Propose a Bridge Transaction
 * @param {KeyringPair} signer - KeyringPair
 * @param {KeyringPair} alice - KeyringPair
 * @return {Promise<void>}
 */
export async function bridgeTransfer(signer: KeyringPair, alice: KeyringPair): Promise<void> {
	const api = await ApiSingleton.getInstance();
	let amount = 10;
	let bridge_tx = {
		nonce: 2,
		recipient: alice.publicKey,
		amount,
		tx_hash: u8aToHex(numberToU8a(1), 256),
	};

	const transaction = api.tx.bridge.proposeBridgeTx(bridge_tx);
	await sendTx(signer, transaction);
}

/**
 * @description Freeze a Bridge Transaction
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function freezeTransaction(signer: KeyringPair): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.bridge.freeze();
	await sendTx(signer, transaction);
}

/**
 * @description Unfreeze a Bridge Transaction
 * @param {KeyringPair} signer - KeyringPair
 * @return {Promise<void>}
 */
export async function unfreezeTransaction(signer: KeyringPair): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.bridge.unfreeze();
	await sendTx(signer, transaction);
}
