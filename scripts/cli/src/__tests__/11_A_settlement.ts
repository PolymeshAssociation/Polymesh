import {
  initMain,
  generateEntityFromUri,
  keyToIdentityIds,
  disconnect,
  transferAmount,
  padTicker,
  waitBlocks,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, assetBalance } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("11 A - Settlement Unit Test", () => {
  test("Settlement A", async () => {
    const ticker = padTicker("11ATICKER");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const bob = await generateEntityFromUri("11A_bob");
    const bobDid = (await createIdentities(alice, [bob]))[0];
    expect(bobDid).toBeTruthy();
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    await expect(
      distributePoly(alice, bob, transferAmount)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(alice, ticker, 1_000_000, null)
    ).resolves.not.toThrow();
    await expect(
      addComplianceRequirement(alice, ticker)
    ).resolves.not.toThrow();

    let aliceBalance = await assetBalance(ticker, aliceDid);
    expect(aliceBalance).toEqual(1_000_000);
    let bobBalance = await assetBalance(ticker, bobDid);
    expect(bobBalance).toEqual(0);

    let venueCounter = await settlement.createVenue(alice, "Other");
    expect(venueCounter).toBeTruthy();

    let intructionCounterAB = await settlement.addInstruction(
      alice,
      venueCounter,
      aliceDid,
      bobDid,
      ticker,
      100
    );
    expect(intructionCounterAB).toBeTruthy();

    await expect(
      settlement.affirmInstruction(alice, intructionCounterAB, aliceDid, 1)
    ).resolves.not.toThrow();
    await expect(
      settlement.affirmInstruction(bob, intructionCounterAB, bobDid, 0)
    ).resolves.not.toThrow();

    // Wait for settlement to be executed - happens in the next block
    await waitBlocks(2);

    //await rejectInstruction(bob, intructionCounter);
    //await unathorizeInstruction(alice, instructionCounter);

    aliceBalance = await assetBalance(ticker, aliceDid);
    expect(aliceBalance).toEqual(999_900);
    bobBalance = await assetBalance(ticker, bobDid);
    expect(bobBalance).toEqual(100);
  });
});
