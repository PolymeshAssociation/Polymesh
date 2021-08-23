import {
	initMain,
	generateRandomEntity,
	generateRandomTicker,
	transferAmount,
	keyToIdentityIds,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePoly } from "../helpers/poly_helper";
import {
	createGroup,
	setGroupPermissions,
	acceptBecomeAgent,
	nextAgId,
	abdicate,
	removeAgent,
	changeGroup,
} from "../helpers/external_agent_helper";
import { ExtrinsicPermissions } from "../types";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const aliceDid = await keyToIdentityIds(alice.publicKey);
	const bob = await generateRandomEntity();
	const bobDid = (await createIdentities(alice, [bob]))[0];
	let extrinsics: ExtrinsicPermissions = { These: [] };
	console.log("Identities Created")
	await distributePoly(alice, bob, transferAmount);
	const ticker = generateRandomTicker();
	await issueTokenToDid(alice, ticker, 1000000, null);
	console.log("EA: Group")
	await createGroup(alice, ticker, extrinsics);
	let agId = await nextAgId(ticker);
	console.log("EA: Group Permissions")
	await setGroupPermissions(alice, ticker, agId, extrinsics);
	console.log("EA: Become Agent")
	await acceptBecomeAgent(bob, bobDid, alice, ticker, { Full: "" });
	await abdicate(alice, ticker);
	console.log("EA: Accept Agent")
	await acceptBecomeAgent(alice, aliceDid, bob, ticker, { Full: "" });
	await removeAgent(alice, ticker, bobDid);
	console.log("EA: Group")
	await createGroup(alice, ticker, extrinsics);
	agId = await nextAgId(ticker);
	await setGroupPermissions(alice, ticker, agId, extrinsics);
	console.log("EA: Change Group")
	await changeGroup(alice, ticker, aliceDid, { Full: "" });
}

main()
	.catch((err: any) => {
		const pe = new PrettyError();
		console.error(pe.render(err));
		process.exit(1);
	})
	.finally(() => {
		console.log("Completed: EXTERNAL AGENTS");
		process.exit();
	  });
