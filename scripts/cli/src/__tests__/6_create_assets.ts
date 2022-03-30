import {
  initMain,
  generateKeys,
  disconnect,
  transferAmount,
  padTicker,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("6 - Create Assets Unit Test", () => {
  test("Creating Assets", async () => {
    const ticker = padTicker("6TICKER");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "6_primary";
    const primaryKeys = await generateKeys(1, primaryDevSeed);
    await expect(createIdentities(alice, primaryKeys)).resolves.not.toThrow();
    await expect(
      distributePolyBatch(alice, primaryKeys, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(primaryKeys[0], ticker, 1000000, null)
    ).resolves.not.toThrow();
  });
});
