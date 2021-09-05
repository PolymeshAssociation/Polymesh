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

  let venueCounter = await createVenue(dave);
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

  await sendTx(
    alice,
    api.tx.settlement.createVenue("", [alice.publicKey], "Sto")
  );
  venueCounter++;
  await sendTx(
    alice,
    api.tx.sto.createFundraiser(
      { did: aliceDid, kind: { Default: null } },
      ticker,
      { did: aliceDid, kind: { Default: null } },
      raisingTicker,
      [{ total: 3000, price: 4 }],
      venueCounter,
      null,
      null,
      0,
      "mySto"
    )
  );
  await sendTx(
    dave,
    api.tx.sto.invest(
      { did: daveDid, kind: { Default: null } },
      { did: daveDid, kind: { Default: null } },
      ticker,
      0,
      40,
      null,
      null
    )
  );
  await sendTx(
    bob,
    api.tx.sto.invest(
      { did: bobDid, kind: { Default: null } },
      { did: bobDid, kind: { Default: null } },
      ticker,
      0,
      10,
      null,
      null
    )
  );
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: INVESTMENTS");
    process.exit();
  });
