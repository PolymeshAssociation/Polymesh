import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  waitNextBlock,
  disconnect,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { invest, createFundraiser } from "../helpers/sto_helper";
import {
  addInstruction,
  affirmInstruction,
  createVenue,
} from "../helpers/settlement_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("14 - Investment Unit Test", () => {
  test("Investment", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    const bob = await generateEntityFromUri("14_bob");
    const dave = await generateEntityFromUri("14_dave");
    const [bobDid, daveDid] = await createIdentities(alice, [bob, dave]);

    console.log("Identities Created");
    await distributePolyBatch(alice, [bob, dave], transferAmount * 10);
    const ticker = padTicker("14TICKER");
    const raisingTicker = padTicker("14TICKERRAIS");
    await issueTokenToDid(alice, ticker, 1000000, null);
    await issueTokenToDid(dave, raisingTicker, 1000000, null);

    await addComplianceRequirement(alice, ticker);
    await addComplianceRequirement(dave, raisingTicker);

    let venueCounter = await createVenue(dave, "Other");
    let intructionCounterAB = await addInstruction(
      dave,
      venueCounter,
      daveDid,
      bobDid,
      raisingTicker,
      1000
    );

    await affirmInstruction(dave, intructionCounterAB, daveDid, 1);
    await affirmInstruction(bob, intructionCounterAB, bobDid, 0);
    await waitNextBlock();

    const aliceVenueCounter = await createVenue(alice, "Sto");

    await createFundraiser(
      alice,
      { did: aliceDid, kind: { Default: "" } },
      ticker,
      { did: aliceDid, kind: { Default: "" } },
      raisingTicker,
      [{ total: 3000, price: 4 }],
      aliceVenueCounter,
      null,
      null,
      0,
      "mySto"
    );

    await invest(
      dave,
      { did: daveDid, kind: { Default: "" } },
      { did: daveDid, kind: { Default: "" } },
      ticker,
      0,
      40,
      null,
      null
    );

    await invest(
      bob,
      { did: bobDid, kind: { Default: "" } },
      { did: bobDid, kind: { Default: "" } },
      ticker,
      0,
      10,
      null,
      null
    );
  });
});
