import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, ApiSingleton } from "../util/init";

/**
 * @description Transfers Poly to KeyringPair
 * @param {KeyringPair} receiver - KeyringPair
 * @param {number} amount - Transfer amount
 * @param {KeyringPair} sender - KeyringPair
 * @return {Promise<void>}
 */
export async function distributePoly(receiver: KeyringPair, amount: number, sender: KeyringPair): Promise<void> {
	const api = await ApiSingleton.getInstance();
	// Perform the transfers
	const transaction = api.tx.balances.transfer(receiver.address, amount);
	await sendTx(sender, transaction);
}

/**
 * @description Transfers Poly to KeyringPair Array
 * @param {KeyringPair[]} receivers - KeyringPair[]
 * @param {number} amount - Transfer amount
 * @param {KeyringPair} sender - KeyringPair
 * @return {Promise<void>}
 */
export async function distributePolyBatch(
	receivers: KeyringPair[],
	amount: number,
	sender: KeyringPair
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	// Perform the transfers
	for (let account of receivers) {
		await distributePoly(account, amount, sender);
	}
}
