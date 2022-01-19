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
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid, assetBalance } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import * as settlement from "../helpers/settlement_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("11 B - Settlement Unit Test", () => {
  test("Settlement B", async () => {
    const ticker = padTicker("11BTICKER1");
    const ticker2 = padTicker("11BTICKER2");
    const testEntities = await initMain();
    const alice = testEntities[0];
    const bob = await generateEntityFromUri("11B_bob");
    const charlie = await generateEntityFromUri("11B_charlie");
    const dave = await generateEntityFromUri("11B_dave");
    const eve = await generateEntityFromUri("11B_eve");
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    const bobDid = (await createIdentities(alice, [bob]))[0];
    expect(bobDid).toBeTruthy();
    const charlieDid = (await createIdentities(alice, [charlie]))[0];
    expect(charlieDid).toBeTruthy();
    const daveDid = (await createIdentities(alice, [dave]))[0];
    expect(daveDid).toBeTruthy();
    const eveDid = (await createIdentities(alice, [eve]))[0];
    expect(eveDid).toBeTruthy();

    await expect(
      distributePolyBatch(alice, [bob, charlie, dave, eve], transferAmount)
    ).resolves.not.toThrow();

    await expect(
      issueTokenToDid(alice, ticker, 1_000_000, null)
    ).resolves.not.toThrow();
    await expect(
      issueTokenToDid(bob, ticker2, 1_000_000, null)
    ).resolves.not.toThrow();

    await expect(
      addComplianceRequirement(alice, ticker)
    ).resolves.not.toThrow();
    await expect(addComplianceRequirement(bob, ticker2)).resolves.not.toThrow();

    let aliceBalance = await assetBalance(ticker, aliceDid);
    expect(aliceBalance).toEqual(1_000_000);
    let bobBalance = await assetBalance(ticker, bobDid);
    expect(bobBalance).toEqual(0);
    let charlieBalance = await assetBalance(ticker, charlieDid);
    expect(charlieBalance).toEqual(0);
    let daveBalance = await assetBalance(ticker, daveDid);
    expect(daveBalance).toEqual(0);
    let eveBalance = await assetBalance(ticker, eveDid);
    expect(eveBalance).toEqual(0);

    aliceBalance = await assetBalance(ticker2, aliceDid);
    expect(aliceBalance).toEqual(0);
    bobBalance = await assetBalance(ticker2, bobDid);
    expect(bobBalance).toEqual(1_000_000);
    charlieBalance = await assetBalance(ticker2, charlieDid);
    expect(charlieBalance).toEqual(0);
    daveBalance = await assetBalance(ticker2, daveDid);
    expect(daveBalance).toEqual(0);
    eveBalance = await assetBalance(ticker2, eveDid);
    expect(eveBalance).toEqual(0);

    const venueCounter = await settlement.createVenue(alice, "Other");
    expect(venueCounter).toBeTruthy();
    let instructionCounter = await settlement.addGroupInstruction(
      alice,
      venueCounter,
      [aliceDid, bobDid, charlieDid, daveDid, eveDid],
      ticker,
      ticker2,
      100
    );
    expect(instructionCounter).toBeTruthy();

    await expect(
      settlement.affirmInstruction(alice, instructionCounter, aliceDid, 4)
    ).resolves.not.toThrow();
    await expect(
      settlement.affirmInstruction(bob, instructionCounter, bobDid, 1)
    ).resolves.not.toThrow();
    await expect(
      settlement.affirmInstruction(charlie, instructionCounter, charlieDid, 0)
    ).resolves.not.toThrow();
    await expect(
      settlement.affirmInstruction(dave, instructionCounter, daveDid, 0)
    ).resolves.not.toThrow();
    //await settlement.rejectInstruction(eve, instructionCounter);
    await expect(
      settlement.affirmInstruction(eve, instructionCounter, eveDid, 0)
    ).resolves.not.toThrow();

    // Wait for settlement to be executed - happens in the next block
    await waitBlocks(2);

    aliceBalance = await assetBalance(ticker, aliceDid);
    expect(aliceBalance).toEqual(999_600);
    bobBalance = await assetBalance(ticker, bobDid);
    expect(bobBalance).toEqual(100);
    charlieBalance = await assetBalance(ticker, charlieDid);
    expect(charlieBalance).toEqual(100);
    daveBalance = await assetBalance(ticker, daveDid);
    expect(daveBalance).toEqual(100);
    eveBalance = await assetBalance(ticker, eveDid);
    expect(eveBalance).toEqual(100);

    aliceBalance = await assetBalance(ticker2, aliceDid);
    expect(aliceBalance).toEqual(100);
    bobBalance = await assetBalance(ticker2, bobDid);
    expect(bobBalance).toEqual(999_900);
    charlieBalance = await assetBalance(ticker2, charlieDid);
    expect(charlieBalance).toEqual(0);
    daveBalance = await assetBalance(ticker2, daveDid);
    expect(daveBalance).toEqual(0);
    eveBalance = await assetBalance(ticker2, eveDid);
    expect(eveBalance).toEqual(0);
  });
});
