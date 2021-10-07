import { initMain, generateKeys } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

test("Create Identities", async () => {
	expect.assertions(1);
	const testEntities = await initMain();
	const alice = testEntities[0];
	const primaryDevSeed = "random_1244";
	const keys = await generateKeys(2, primaryDevSeed);
	await expect(await createIdentities(alice, keys)).resolves.toHaveReturned();
});
