import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, ApiSingleton } from "../util/init";
import { distributePolyBatch } from "../helpers/poly_helper";
import BN from "bn.js";

/**
 * @description Take the origin account as a stash and lock up `value` of its balance.
 */
export async function bond(signer: KeyringPair, controller: KeyringPair, value: number, payee: string): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.staking.bond(controller.publicKey, value, payee);
	await sendTx(signer, transaction);
}

export async function addNominator(controller: KeyringPair[], stash: KeyringPair[], from: KeyringPair, validator: KeyringPair[]) {
	const api = await ApiSingleton.getInstance();
    let transfer_amount = new BN(1).mul(new BN(10).pow(new BN(6)));
    let operators = [validator[0].address, validator[1].address];
    let bond_amount = new BN(3).mul(new BN(10).pow(new BN(6)));

    // bond nominator first
    for (let i = 0; i < stash.length; i++) {
        const tx = api.tx.staking.bond(controller[i].address, bond_amount, "Controller");
		await sendTx(stash[i], tx);
    }
    //await blockTillPoolEmpty();
    // fund controller keys
    await distributePolyBatch(from, controller, transfer_amount.toNumber());
    //await blockTillPoolEmpty();

    for (let i = 0; i < controller.length; i++) {
        const tx = api.tx.staking.nominate(operators);
		await sendTx(controller[i], tx);
    }
}
