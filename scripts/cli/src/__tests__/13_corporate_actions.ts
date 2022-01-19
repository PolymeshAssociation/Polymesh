import {
  generateKeys,
  transferAmount,
  initMain,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  disconnect,
} from "../util/init";
import {
  changeDefaultTargetIdentitites,
  changeWithholdingTax,
  claimDistribution,
  createDistribution,
  initiateCorporateAction,
} from "../helpers/corporate_actions_helper";
import {
  authorizeJoinToIdentities,
  createIdentities,
} from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { addSecondaryKeys } from "../helpers/key_management_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import {
  addInstruction,
  affirmInstruction,
  createVenue,
} from "../helpers/settlement_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("13 - Corporate Actions Unit Test", () => {
  test("Corporate Actions", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    const bob = await generateEntityFromUri("13_bob");
    const bobDid = (await createIdentities(alice, [bob]))[0];
    expect(bobDid).toBeTruthy();
    const primaryDevSeed = "13_primary";
    const secondaryDevSeed = "13_secondary";
    const primaryKeys = await generateKeys(1, primaryDevSeed);
    const secondaryKeys = await generateKeys(1, secondaryDevSeed);
    await expect(createIdentities(alice, primaryKeys)).resolves.not.toThrow();

    const ticker = padTicker("13TICKER");
    const earnedTicker = padTicker("13EARNED");

    await expect(
      distributePolyBatch(alice, [primaryKeys[0]], transferAmount)
    ).resolves.not.toThrow();
    await expect(
      addSecondaryKeys(primaryKeys, secondaryKeys)
    ).resolves.not.toThrow();
    await expect(
      authorizeJoinToIdentities(secondaryKeys, primaryKeys)
    ).resolves.not.toThrow();
    await expect(
      distributePolyBatch(alice, [secondaryKeys[0], bob], transferAmount)
    ).resolves.not.toThrow();

    console.log("Distributing tokens");
    await expect(
      issueTokenToDid(alice, ticker, 1000000000, null)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(alice, earnedTicker, 20000000000, null)
    ).resolves.not.toThrow();

    console.log("adding compliance requirement");
    await expect(
      addComplianceRequirement(alice, ticker)
    ).resolves.not.toThrow();
    await expect(
      addComplianceRequirement(alice, earnedTicker)
    ).resolves.not.toThrow();

    console.log("transfering token to Bob");
    let venueCounter = await createVenue(alice, "Other");
    expect(venueCounter).toBeTruthy();
    let intructionCounterAB = await addInstruction(
      alice,
      venueCounter,
      aliceDid,
      bobDid,
      ticker,
      100000000
    );
    expect(intructionCounterAB).toBeTruthy();
    console.log("affirming transfer");
    await expect(
      affirmInstruction(alice, intructionCounterAB, aliceDid, 1)
    ).resolves.not.toThrow();
    await expect(
      affirmInstruction(bob, intructionCounterAB, bobDid, 0)
    ).resolves.not.toThrow();

    console.log("changing default target and taxes for corporate action");
    await expect(
      changeDefaultTargetIdentitites(alice, ticker, [bob], "include")
    ).resolves.not.toThrow();
    await expect(
      changeWithholdingTax(alice, ticker, 15)
    ).resolves.not.toThrow();

    await expect(
      initiateCorporateAction(
        alice,
        ticker,
        "PredictableBenefit",
        "100",
        { existing: 1 },
        "Regular dividend",
        null,
        null,
        null
      )
    ).resolves.not.toThrow();

    console.log("creating distribution");
    await expect(
      createDistribution(
        alice,
        ticker,
        "0",
        null,
        earnedTicker,
        1000000,
        2000000000,
        null
      )
    ).resolves.not.toThrow();

    await expect(claimDistribution(bob, ticker, 0)).resolves.not.toThrow();
  });
});
