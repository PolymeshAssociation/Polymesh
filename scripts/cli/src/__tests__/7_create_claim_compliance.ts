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
import { createClaimCompliance } from "../helpers/compliance_manager_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("7 - Claim Compliance Unit Test", () => {
  test("Create Claim Compliance", async () => {
    const ticker = padTicker("7TICKER");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "7_primary";
    const primaryKeys = await generateKeys(1, primaryDevSeed);
    let issuerDid = await createIdentities(alice, primaryKeys);
    expect(issuerDid).toBeTruthy();
    await expect(
      distributePolyBatch(alice, primaryKeys, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(primaryKeys[0], ticker, 1000000, null)
    ).resolves.not.toThrow();
    await expect(
      createClaimCompliance(primaryKeys[0], issuerDid[0], ticker)
    ).resolves.not.toThrow();
  });
});
