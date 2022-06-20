import {
  initMain,
  generateKeys,
  disconnect,
  transferAmount,
  padTicker,
} from "../util/init";
import { createIdentities, addClaim } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("5 - Claim Management Unit Test", () => {
  test("Adding Claims", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const issuerSeed1 = "5_issuer_1";
    const issuerSeed2 = "5_issuer_2";
    const claimeeSeed = "5_claimee";
    const issuerKeys1 = await generateKeys(2, issuerSeed1);
    const issuerKeys2 = await generateKeys(2, issuerSeed2);
    const claimeeKeys = await generateKeys(2, claimeeSeed);
    await expect(createIdentities(alice, issuerKeys1)).resolves.not.toThrow();
    await expect(createIdentities(alice, issuerKeys2)).resolves.not.toThrow();
    const claimeeDids = await createIdentities(alice, claimeeKeys);
    expect(claimeeDids).toBeTruthy();
    const ticker = padTicker("5TICKER");
    await expect(
      distributePolyBatch(
        alice,
        issuerKeys1.concat(issuerKeys2).concat(claimeeKeys),
        transferAmount
      )
    ).resolves.not.toThrow();
    console.log("Adding Exempted claim");
    await expect(
      addClaim(
        issuerKeys1[0],
        claimeeDids[0],
        { Exempted: { Identity: claimeeDids[1] } },
        null
      )
    ).resolves.not.toThrow();
    console.log("Adding SellLockup claim");
    await expect(
      addClaim(
        issuerKeys1[0],
        claimeeDids[0],
        { SellLockup: { Ticker: ticker } },
        null
      )
    ).resolves.not.toThrow();
    console.log("Adding Accredited claim");
    await expect(
      addClaim(
        issuerKeys2[0],
        claimeeDids[0],
        { Accredited: { Ticker: ticker } },
        null
      )
    ).resolves.not.toThrow();
    console.log("Adding Affiliate claim");
    await expect(
      addClaim(
        issuerKeys2[0],
        claimeeDids[0],
        { Affiliate: { Ticker: ticker } },
        Date.now() as any
      )
    ).resolves.not.toThrow();
  });
});
