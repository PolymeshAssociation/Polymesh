import {
  initMain,
  generateKeys,
  disconnect,
  transferAmount,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("1 - Poly Unit Test", () => {
  test("Polyx Transaction", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "1_primary";
    const keys = await generateKeys(2, primaryDevSeed);
    const dids = await createIdentities(alice, keys);
    expect(dids).toBeTruthy();
    const tx = distributePolyBatch(alice, keys, transferAmount);
    await expect(tx).resolves.not.toThrow();
  });
});
