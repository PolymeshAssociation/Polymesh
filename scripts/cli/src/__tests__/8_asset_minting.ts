import {
  initMain,
  generateKeys,
  disconnect,
  transferAmount,
  padTicker,
} from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("8 - Asset Minting Unit Test", () => {
  test("Minting Asset", async () => {
    const ticker = padTicker("8TICKER");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const primaryDevSeed = "8_primary";
    const primaryKey = (await generateKeys(1, primaryDevSeed))[0];
    let issuerDid = await createIdentities(alice, [primaryKey]);
    expect(issuerDid).toBeTruthy();
    await distributePoly(alice, primaryKey, transferAmount);
    await expect(
      distributePoly(alice, primaryKey, transferAmount)
    ).resolves.not.toThrow();
    await issueTokenToDid(primaryKey, ticker, 1000000, "first");
    await expect(
      issueTokenToDid(primaryKey, ticker, 1000000, "first")
    ).resolves.not.toThrow();
    await expect(
      addClaimsToDids(
        primaryKey,
        issuerDid[0],
        "Exempted",
        { Ticker: ticker },
        null
      )
    ).resolves.not.toThrow();
    await expect(
      addComplianceRequirement(primaryKey, ticker)
    ).resolves.not.toThrow();
    await expect(mintingAsset(primaryKey, ticker)).resolves.not.toThrow();
  });
});
