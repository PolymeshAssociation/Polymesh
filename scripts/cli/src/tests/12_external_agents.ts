import {
  initMain,
  transferAmount,
  keyToIdentityIds,
  generateEntityFromUri,
  padTicker,
  sendTx,
  ApiSingleton,
  waitNextBlock,
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
import PrettyError from "pretty-error";
import {
  addInstruction,
  affirmInstruction,
  createVenue,
} from "../helpers/settlement_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";

async function main(): Promise<void> {
  const api = await ApiSingleton.getInstance();
  const testEntities = await initMain();
  const alice = testEntities[0];
  const aliceDid = await keyToIdentityIds(alice.publicKey);
  const bob = await generateEntityFromUri("12_bob");
  const dave = await generateEntityFromUri("12_dave");
  const [bobDid, daveDid] = await createIdentities(alice, [bob, dave]);
  let extrinsics: ExtrinsicPermissions = { These: [] };
  console.log("Identities Created");
  await distributePolyBatch(alice, [bob, dave], transferAmount * 10);
  const ticker = padTicker("12TICKER");
  const raisingTicker = padTicker("12TICKERRAIS");
  await issueTokenToDid(alice, ticker, 1000000, null);
  console.log("EA: Group");
  await createGroup(alice, ticker, extrinsics);

  let agId = await nextAgId(ticker);
  console.log("EA: Group Permissions");
  await setGroupPermissions(alice, ticker, agId, extrinsics);
  console.log("EA: Become Agent");
  await acceptBecomeAgent(bob, bobDid, alice, ticker, { Full: "" });
  await acceptBecomeAgent(dave, daveDid, alice, ticker, { Full: "" });
  await addComplianceRequirement(alice, ticker);

  await abdicate(alice, ticker);

  console.log("EA: Accept Agent");
  await acceptBecomeAgent(alice, aliceDid, bob, ticker, { Full: "" });
  await removeAgent(alice, ticker, bobDid);
  console.log("EA: Group");
  await createGroup(alice, ticker, extrinsics);
  agId = await nextAgId(ticker);
  await setGroupPermissions(alice, ticker, agId, extrinsics);
  console.log("EA: Change Group");
  await changeGroup(alice, ticker, aliceDid, { Full: "" });

  // External agent authorized extrinsics

  let venueCounter = await createVenue(alice);
  let intructionCounterAB = await addInstruction(
    alice,
    venueCounter,
    aliceDid,
    daveDid,
    ticker,
    1000
  );

  await affirmInstruction(alice, intructionCounterAB, aliceDid, 1);
  await affirmInstruction(dave, intructionCounterAB, daveDid, 0);
  await waitNextBlock();

  await sendTx(
    dave,
    api.tx.settlement.createVenue("", [dave.publicKey], "Sto")
  );
  venueCounter++;
  await sendTx(
    dave,
    api.tx.sto.createFundraiser(
      { did: daveDid, kind: { Default: null } },
      ticker,
      { did: daveDid, kind: { Default: null } },
      raisingTicker,
      [{ total: 3, price: 4 }],
      venueCounter,
      null,
      null,
      0,
      "mySto"
    )
  );
  await sendTx(dave, api.tx.sto.freezeFundraiser(ticker, 0));
  await sendTx(dave, api.tx.sto.unfreezeFundraiser(ticker, 0));
  await sendTx(
    dave,
    api.tx.sto.modifyFundraiserWindow(ticker, 0, Date.now() as any, null)
  );
  await sendTx(dave, api.tx.sto.stop(ticker, 0));
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: EXTERNAL AGENTS");
    process.exit();
  });
