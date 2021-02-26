import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx } from "../util/init";

/**
 * @description Transfers Poly to KeyringPair
 * @param {ApiPromise}  api - ApiPromise 
 * @param {KeyringPair} receiver - KeyringPair
 * @param {number} amount - Transfer amount
 * @param {KeyringPair} sender - KeyringPair 
 * @return {Promise<void>} 
 */
export async function distributePoly(
	api: ApiPromise,
	receiver: KeyringPair,
	amount: number,
	sender: KeyringPair
): Promise<void> {
	// Perform the transfers
	const transaction = api.tx.balances.transfer(receiver.address, amount);
	await sendTx(sender, transaction);
}

/**
 * @description Transfers Poly to KeyringPair Array
 * @param {ApiPromise}  api - ApiPromise 
 * @param {KeyringPair[]} receivers - KeyringPair[]
 * @param {number} amount - Transfer amount
 * @param {KeyringPair} sender - KeyringPair 
 * @return {Promise<void>} 
 */
export async function distributePolyBatch(
	api: ApiPromise,
	receivers: KeyringPair[],
	amount: number,
	sender: KeyringPair
): Promise<void> {
	// Perform the transfers
	for (let account of receivers) {
		await distributePoly(api, account, amount, sender);
	}
}