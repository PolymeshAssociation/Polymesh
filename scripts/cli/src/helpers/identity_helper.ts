import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Moment } from "@polkadot/types/interfaces/runtime";
import { sendTx, keyToIdentityIds } from "../util/init";
import { IdentityId, Authorization } from "../types";

// Authorizes the join of secondary keys to a DID
/**
 * @description Creates an Identity for KeyringPairs.
 * @param {ApiPromise}  api - ApiPromise 
 * @param {KeyringPair[]} primaryKeys - An array of KeyringPairs
 * @param {IdentityId[]} dids - An array of IdentityIds
 * @param {KeyringPair[]} secondaryKeys - An array of KeyringPairs
 * @return {Promise<IdentityId[]>} Creates an array of identities
 */
export async function authorizeJoinToIdentities(
	api: ApiPromise,
	primaryKeys: KeyringPair[],
	dids: IdentityId[],
	secondaryKeys: KeyringPair[]
): Promise<IdentityId[]> {
	for (let i in primaryKeys) {
		// 1. Authorize
		const auths = ((await api.query.identity.authorizations.entries({
			Account: secondaryKeys[i].publicKey,
		})) as unknown) as Authorization[][];
		let last_auth_id = 0;
		for (let j in auths) {
			if (auths[j][1].auth_id > last_auth_id) {
				last_auth_id = auths[j][1].auth_id;
			}
		}

		const transaction = api.tx.identity.joinIdentityAsKey([last_auth_id]);
		await sendTx(secondaryKeys[i], transaction);
	}

	return dids;
}

/**
 * @description Creates an Identity for KeyringPairs.
 * @param {ApiPromise}  api - ApiPromise 
 * @param {KeyringPair[]} accounts - An array of KeyringPairs
 * @param {KeyringPair} signer - KeyringPair 
 * @return {Promise<IdentityId[]>} Creates an array of identities
 */
export async function createIdentities(
	api: ApiPromise,
	accounts: KeyringPair[],
	signer: KeyringPair
): Promise<IdentityId[]> {
	return await createIdentitiesWithExpiry(api, accounts, signer, []);
}

async function createIdentitiesWithExpiry(
	api: ApiPromise,
	accounts: KeyringPair[],
	signer: KeyringPair,
	expiries: Moment[]
): Promise<IdentityId[]> {
	let dids: IdentityId[] = [];

	for (let account of accounts) {
		let account_did = (await keyToIdentityIds(api, account.publicKey)).toString();
		
		if (parseInt(account_did) == 0) {
			console.log(
				`>>>> [Register CDD Claim] acc: ${account.address}`
			);
			const transaction = api.tx.identity.cddRegisterDid(
				account.address,
				[]
			);
			await sendTx(signer, transaction);
		} else {
			console.log("Identity Already Linked.");
		}
	}
	//await blockTillPoolEmpty(api);

	for (let i in accounts) {
		const d = await keyToIdentityIds(api, accounts[i].publicKey);
		dids.push(d);
		console.log(
			`>>>> [Get DID ] acc: ${accounts[i].address} did: ${dids[i]}`
		);
	}

	// Add CDD Claim with CDD_ID
	for (let i in dids) {
		const cdd_id_byte = (parseInt(i) + 1).toString(16).padStart(2, "0");
		const claim = {
			CustomerDueDiligence: `0x00000000000000000000000000000000000000000000000000000000000000${cdd_id_byte}`,
		};
		const expiry = expiries.length == 0 ? null : expiries[i];

		console.log(
			`>>>> [add CDD Claim] did: ${dids[i]}, claim: ${JSON.stringify(
				claim
			)}`
		);
		const transaction = api.tx.identity.addClaim(dids[i], claim, expiry);
		await sendTx(signer, transaction);	
	}

	return dids;
}