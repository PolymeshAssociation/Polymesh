import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Transfers Poly to KeyringPair
 * @param {KeyringPair} signer - KeyringPair
 * @param {KeyringPair} receiver - KeyringPair
 * @param {number} amount - Transfer amount
 * @return {Promise<void>}
 */
export async function distributePoly(signer: KeyringPair, receiver: KeyringPair, amount: number): Promise<void> {
	const api = await ApiSingleton.getInstance();
	// Perform the transfers
	const transaction = api.tx.balances.transfer(receiver.address, amount);
	await sendTx(signer, transaction);
}

/**
 * @description Transfers Poly to KeyringPair Array
 * @param {KeyringPair} signer - KeyringPair
 * @param {KeyringPair[]} receivers - KeyringPair[]
 * @param {number} amount - Transfer amount
 * @return {Promise<void>}
 */
export async function distributePolyBatch(
	signer: KeyringPair,
	receivers: KeyringPair[],
	amount: number
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	// Perform the transfers
	for (let receiver of receivers) {
		await distributePoly(signer, receiver, amount);
	}
}
