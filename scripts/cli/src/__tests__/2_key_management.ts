import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import {
  addSecondaryKeys,
  createMultiSig,
} from "../helpers/key_management_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await init.disconnect();
});

describe("2 - Key Management Unit Test", () => {
  test("Key Management", async () => {
    const testEntities = await init.initMain();
    const primaryDevSeed = "2_primary";
    const secondaryDevSeed = "2_secondary";
    const alice = testEntities[0];
    const bob = await init.generateEntityFromUri("2_bob");
    const charlie = await init.generateEntityFromUri("2_charlie");
    const dave = await init.generateEntityFromUri("2_dave");
    const primaryKeys = await init.generateKeys(2, primaryDevSeed);
    const secondaryKeys = await init.generateKeys(2, secondaryDevSeed);
    const bobSignatory = await init.signatory(alice, bob);
    const charlieSignatory = await init.signatory(alice, charlie);
    const daveSignatory = await init.signatory(alice, dave);
    const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
    const dids = await createIdentities(alice, primaryKeys);
    expect(dids).toBeTruthy();
    await expect(
      distributePolyBatch(alice, primaryKeys, init.transferAmount)
    ).resolves.not.toThrow();
    await expect(
      addSecondaryKeys(primaryKeys, secondaryKeys)
    ).resolves.not.toThrow();
    await expect(
      createMultiSig(alice, signatoryArray, 2)
    ).resolves.not.toThrow();
  });
});
