import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, ExtrinsicPermissions } from "../types";
import { sendTx, ApiSingleton } from "../util/init";
import type { AgentGroup, IdentityId } from "../interfaces";

/**
 * @description Creates Group
 */
export async function createGroup(
	signer: KeyringPair,
	ticker: Ticker,
	perms: ExtrinsicPermissions
): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.createGroup(ticker, perms);
	await sendTx(signer, transaction);
}

export async function setGroupPermissions(
	signer: KeyringPair,
	ticker: Ticker,
	id: number,
	perms: ExtrinsicPermissions
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.setGroupPermissions(
		ticker,
		id,
		perms
	);
	await sendTx(signer, transaction);
}

export async function abdicate(signer: KeyringPair, ticker: Ticker) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.abdicate(ticker);
	await sendTx(signer, transaction);
}

export async function removeAgent(
	signer: KeyringPair,
	ticker: Ticker,
	agent: IdentityId
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.removeAgent(ticker, agent);
	await sendTx(signer, transaction);
}

export async function changeGroup(
	signer: KeyringPair,
	ticker: Ticker,
	agent: IdentityId,
	group: AgentGroup
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.changeGroup(ticker, agent, group);
	await sendTx(signer, transaction);
}

export async function acceptBecomeAgent(signer: KeyringPair, authId: number) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.acceptBecomeAgent(authId);
	await sendTx(signer, transaction);
}

export async function nextAgId(signer: KeyringPair, ticker: Ticker) {
    const api = await ApiSingleton.getInstance();
    return api.query.externalAgents.aGIdSequence(ticker);
}