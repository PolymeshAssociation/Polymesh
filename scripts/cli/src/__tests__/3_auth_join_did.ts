import {
  initMain,
  generateKeys,
  disconnect,
  transferAmount,
} from "../util/init";
import {
  createIdentities,
  authorizeJoinToIdentities,
} from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("3 - Auth Unit Test", () => {
  test("Authorizing join to Identities", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "3_primary";
    const secondaryDevSeed = "3_secondary";
    const primaryKeys = await generateKeys(2, primaryDevSeed);
    const secondaryKeys = await generateKeys(2, secondaryDevSeed);
    const dids = await createIdentities(alice, primaryKeys);
    expect(dids).toBeTruthy();
    await expect(
      distributePolyBatch(alice, primaryKeys, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      addSecondaryKeys(primaryKeys, secondaryKeys)
    ).resolves.not.toThrow();
    await expect(
      authorizeJoinToIdentities(secondaryKeys, primaryKeys)
    ).resolves.not.toThrow();
  });
});
