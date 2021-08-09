import type { KeyringPair } from "@polkadot/keyring/types";
import type { Ticker, ExtrinsicPermissions, AuthorizationData, AgentGroup } from "../types";
import type { AnyNumber } from "@polkadot/types/types";
import { sendTx, ApiSingleton, signatory } from "../util/init";
import { addAuthorization, getAuthId } from "../helpers/identity_helper";
import type { IdentityId } from "../interfaces";

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
	id: AnyNumber,
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
	group: any
) {
	const api = await ApiSingleton.getInstance();
	const transaction = api.tx.externalAgents.changeGroup(ticker, agent, group);
	await sendTx(signer, transaction);
}

export async function acceptBecomeAgent(
	signer: KeyringPair,
	signerDid: IdentityId,
	from: KeyringPair,
	ticker: Ticker,
	group: AgentGroup
) {
	const api = await ApiSingleton.getInstance();
	let data = { BecomeAgent: [ticker, group] };
	let sig = {
		Identity: signerDid,
	};
	await addAuthorization(from, sig, data, null);
	let authId = await getAuthId();
	const transaction = api.tx.externalAgents.acceptBecomeAgent(authId);
	await sendTx(signer, transaction);
}

export async function nextAgId(ticker: Ticker) {
	const api = await ApiSingleton.getInstance();
	const agId = await api.query.externalAgents.aGIdSequence(ticker);
	return agId as unknown as number;
}