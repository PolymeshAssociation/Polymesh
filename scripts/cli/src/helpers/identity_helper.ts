import type { ApiPromise } from "@polkadot/api";
import type { KeyringPair } from "@polkadot/keyring/types";
import type { AccountId } from "@polkadot/types/interfaces";
import type {
	IdentityId,
	Authorization,
	LegacyPalletPermissions,
	PortfolioId,
	Ticker,
	Permissions,
	Signatory,
	Scope,
	Expiry,
} from "../types";
import { sendTx, keyToIdentityIds, handle } from "../util/init";

// TODO Refactor function to deal with all the possible claim types and their values
/**
 * @description Adds a Claim to an Identity
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair} signer - KeyringPair
 * @param {IdentityId} did - IdentityId
 * @param {string} claimType - Type of Claim
 * @param {Scope} claimValue - Claim value
 * @param {Expiry=} expiry -
 * @return {Promise<void>}
 */
export async function addClaimsToDids(
	api: ApiPromise,
	signer: KeyringPair,
	did: IdentityId,
	claimType: string,
	claimValue: Scope,
	expiry?: Expiry
): Promise<void> {
	// Receieving Conditions Claim
	let claim = { [claimType]: claimValue };
	const transaction = api.tx.identity.addClaim(did, claim, expiry);
	await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
}

/**
 * @description Sets permission to signer key
 * @param {ApiPromise}  api - ApiPromise
 * @param {KeyringPair[]} primaryKeys - An array of KeyringPairs
 * @param {KeyringPair[]} secondaryKeys - An array of KeyringPairs
 * @param {LegacyPalletPermissions[]} extrinsic - An array of LegacyPalletPermissions
 * @param {PortfolioId[]} portfolio - An array of PortfolioIds
 * @param {Ticker[]} asset - An array of Tickers
 * @return {Promise<void>}
 */
export async function setPermissionToSigner(
	api: ApiPromise,
	primaryKeys: KeyringPair[],
	secondaryKeys: KeyringPair[],
	extrinsic: LegacyPalletPermissions[],
	portfolio: PortfolioId[],
	asset: Ticker[]
): Promise<void> {
	const permissions: Permissions = {
		asset,
		extrinsic,
		portfolio,
	};

	for (let i in primaryKeys) {
		let signer: Signatory = {
			Account: secondaryKeys[i].publicKey as AccountId,
		};
		let transaction = api.tx.identity.legacySetPermissionToSigner(signer, permissions);
		await sendTx(primaryKeys[i], transaction).catch((err) => console.log(`Error: ${err.message}`));
	}
}

/**
 * @description Authorizes the joining of secondary keys to a DID
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
		const [entriesErr, entries] = await handle(
			api.query.identity.authorizations.entries({
				Account: secondaryKeys[i].publicKey,
			})
		);
		if (entriesErr) throw new Error("Failed to get authorizations");
		const auths = (entries as unknown) as Authorization[][];
		let last_auth_id = 0;
		for (let j in auths) {
			if (auths[j][1].auth_id > last_auth_id) {
				last_auth_id = auths[j][1].auth_id;
			}
		}
		const transaction = api.tx.identity.joinIdentityAsKey([last_auth_id]);
		await sendTx(secondaryKeys[i], transaction).catch((err) => console.log(`Error: ${err.message}`));
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
	return createIdentitiesWithExpiry(api, accounts, signer, []);
}

async function createIdentitiesWithExpiry(
	api: ApiPromise,
	accounts: KeyringPair[],
	signer: KeyringPair,
	expiries: number[]
): Promise<IdentityId[]> {
	let dids: IdentityId[] = [];

	for (let account of accounts) {
		let account_did = (await keyToIdentityIds(api, account.publicKey)).toString();

		if (parseInt(account_did) == 0) {
			console.log(`>>>> [Register CDD Claim] acc: ${account.address}`);
			const transaction = api.tx.identity.cddRegisterDid(account.address, []);
			await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
		} else {
			console.log("Identity Already Linked.");
		}
	}
	await setDidsArray(api, dids, accounts).catch((err) => console.log(`Error: ${err.message}`));
	await addCddClaim(api, dids, expiries, signer).catch((err) => console.log(`Error: ${err.message}`));
	return dids;
}

async function setDidsArray(api: ApiPromise, dids: IdentityId[], accounts: KeyringPair[]) {
	for (let i in accounts) {
		const [didErr, did] = await handle(keyToIdentityIds(api, accounts[i].publicKey));
		if (didErr) throw new Error("keyToIdentityIds failed");
		dids.push(did);
		console.log(`>>>> [Get DID ] acc: ${accounts[i].address} did: ${dids[i]}`);
	}
}

async function addCddClaim(api: ApiPromise, dids: IdentityId[], expiries: number[], signer: KeyringPair) {
	// Add CDD Claim with CDD_ID
	for (let i in dids) {
		const cdd_id_byte = (parseInt(i) + 1).toString(16).padStart(2, "0");
		const claim = {
			CustomerDueDiligence: `0x00000000000000000000000000000000000000000000000000000000000000${cdd_id_byte}`,
		};
		const expiry = expiries.length == 0 ? null : expiries[i];

		console.log(`>>>> [add CDD Claim] did: ${dids[i]}, claim: ${JSON.stringify(claim)}`);
		const transaction = api.tx.identity.addClaim(dids[i], claim, expiry);
		await sendTx(signer, transaction).catch((err) => console.log(`Error: ${err.message}`));
	}
}
