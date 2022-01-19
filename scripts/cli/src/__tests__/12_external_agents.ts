import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  waitNextBlock,
  disconnect,
  getDefaultPortfolio,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import {
  createGroup,
  setGroupPermissions,
  acceptBecomeAgent,
  nextAgId,
  abdicate,
  removeAgent,
  changeGroup,
} from "../helpers/external_agent_helper";
import { ExtrinsicPermissions } from "../types";
import {
  addInstruction,
  affirmInstruction,
  createVenue,
} from "../helpers/settlement_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import {
  createFundraiser,
  freezeFundraiser,
  unfreezeFundraiser,
  modifyFundraiserWindow,
  stop,
} from "../helpers/sto_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("12 - External Agents Unit Test", () => {
  test("External Agents Test", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const aliceDid = await keyToIdentityIds(alice.publicKey);
    const bob = await generateEntityFromUri("12_bob");
    const dave = await generateEntityFromUri("12_dave");
    const [bobDid, daveDid] = await createIdentities(alice, [bob, dave]);
    expect(bobDid).toBeTruthy();
    expect(daveDid).toBeTruthy();
    let extrinsics: ExtrinsicPermissions = { These: [] };
    console.log("Identities Created");
    await expect(
      distributePolyBatch(alice, [bob, dave], transferAmount * 10)
    ).resolves.not.toThrow();
    const ticker = padTicker("12TICKER");
    const raisingTicker = padTicker("12TICKERRAIS");
    await expect(
      issueTokenToDid(alice, ticker, 1000000, null)
    ).resolves.not.toThrow();
    console.log("EA: Group");
    await expect(createGroup(alice, ticker, extrinsics)).resolves.not.toThrow();

    let agId = await nextAgId(ticker);
    console.log("EA: Group Permissions");
    await expect(
      setGroupPermissions(alice, ticker, agId, extrinsics)
    ).resolves.not.toThrow();
    console.log("EA: Become Agent");
    await expect(
      acceptBecomeAgent(bob, bobDid, alice, ticker, { Full: "" })
    ).resolves.not.toThrow();
    await expect(
      acceptBecomeAgent(dave, daveDid, alice, ticker, { Full: "" })
    ).resolves.not.toThrow();
    await expect(
      addComplianceRequirement(alice, ticker)
    ).resolves.not.toThrow();

    await expect(abdicate(alice, ticker)).resolves.not.toThrow();

    console.log("EA: Accept Agent");
    await expect(
      acceptBecomeAgent(alice, aliceDid, bob, ticker, { Full: "" })
    ).resolves.not.toThrow();
    await expect(removeAgent(alice, ticker, bobDid)).resolves.not.toThrow();
    console.log("EA: Group");
    await expect(createGroup(alice, ticker, extrinsics)).resolves.not.toThrow();
    agId = await nextAgId(ticker);
    await expect(
      setGroupPermissions(alice, ticker, agId, extrinsics)
    ).resolves.not.toThrow();
    console.log("EA: Change Group");
    await expect(
      changeGroup(alice, ticker, aliceDid, { Full: "" })
    ).resolves.not.toThrow();

    // External agent authorized extrinsics

    let venueCounter = await createVenue(alice, "Other");
    expect(venueCounter).toBeTruthy();
    let intructionCounterAB = await addInstruction(
      alice,
      venueCounter,
      aliceDid,
      daveDid,
      ticker,
      1000
    );
    expect(intructionCounterAB).toBeTruthy();

    await expect(
      affirmInstruction(alice, intructionCounterAB, aliceDid, 1)
    ).resolves.not.toThrow();
    await expect(
      affirmInstruction(dave, intructionCounterAB, daveDid, 0)
    ).resolves.not.toThrow();
    await waitNextBlock();

    const daveVenueCounter = await createVenue(dave, "Sto");
    const davePortfolio = getDefaultPortfolio(daveDid);
    await expect(
      createFundraiser(
        dave,
        davePortfolio,
        ticker,
        davePortfolio,
        raisingTicker,
        [{ total: 3, price: 4 }],
        daveVenueCounter,
        null,
        null,
        0,
        "mySto"
      )
    ).resolves.not.toThrow();
    await expect(freezeFundraiser(dave, ticker, 0)).resolves.not.toThrow();
    await expect(unfreezeFundraiser(dave, ticker, 0)).resolves.not.toThrow();
    await expect(
      modifyFundraiserWindow(dave, ticker, 0, Date.now() as any, null)
    ).resolves.not.toThrow();
    await expect(stop(dave, ticker, 0)).resolves.not.toThrow();
  });
});
