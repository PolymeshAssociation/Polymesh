import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, ApiSingleton, sleep } from "../util/init";
import { distributePolyBatch } from "../helpers/poly_helper";
import BN from "bn.js";

/**
 * @description Take the origin account as a stash and lock up `value` of its balance.
 */
export async function bond(signer: KeyringPair, value: number, payee: string): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.staking.bond(value, payee);
	await sendTx(signer, transaction);
}

export async function addNominator(stash: KeyringPair[], from: KeyringPair, validator: KeyringPair[]) {
	const api = await ApiSingleton.getInstance();
    let transfer_amount = new BN(1).mul(new BN(10).pow(new BN(6)));
    let operators = [validator[0].address, validator[1].address];
    let bond_amount = new BN(3).mul(new BN(10).pow(new BN(6)));

    // bond nominator first
    for (let i = 0; i < stash.length; i++) {
        const tx1 = api.tx.staking.bond(bond_amount, "Controller");
		await sendTx(stash[i], tx1);
        const tx2 = api.tx.staking.nominate(operators);
		await sendTx(stash[i], tx2);
    }
}

export async function forceNewEra(signer: KeyringPair) {
  const api = await ApiSingleton.getInstance();
  const transaction = api.tx.sudo.sudo(api.tx.staking.forceNewEra());
  await sendTx(signer, transaction);
}

export async function unbond(signer: KeyringPair, amount: number) {
  const api = await ApiSingleton.getInstance();
	const transaction = api.tx.staking.unbond(amount);
	await sendTx(signer, transaction);
}

export async function nominate(signer: KeyringPair, target: Uint8Array) {
  const api = await ApiSingleton.getInstance();
	const transaction = api.tx.staking.nominate([target]);
	await sendTx(signer, transaction);
}
