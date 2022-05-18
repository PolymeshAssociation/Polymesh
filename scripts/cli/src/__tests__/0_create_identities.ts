import {
  initMain,
  generateKeys,
  disconnect,
  generateEntity,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("0 - Identity Unit Test", () => {
  test("Create Identities", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "0_primary";
    const keys = await generateKeys(2, primaryDevSeed);
    const dids = await createIdentities(alice, keys);
    expect(dids).toBeTruthy();
  });

  test("Errors when creating identities", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const entity = await generateEntity("0_entity");
    const entity1 = await generateEntity("1_entity");
    await createIdentities(alice, [entity]);
    await expect(createIdentities(entity1, [entity])).rejects.toThrow();
  });
});
