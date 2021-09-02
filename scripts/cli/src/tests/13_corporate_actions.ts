import {
  generateKeys,
  transferAmount,
  initMain,
  generateRandomKey,
  generateRandomTicker,
  keyToIdentityIds,
  generateRandomEntity,
} from "../util/init";
import PrettyError from "pretty-error";
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

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const aliceDid = await keyToIdentityIds(alice.publicKey);
  const bob = await generateRandomEntity();
  const bobDid = (await createIdentities(alice, [bob]))[0];
  const primaryDevSeed = generateRandomKey();
  const secondaryDevSeed = generateRandomKey();
  const primaryKeys = await generateKeys(1, primaryDevSeed);
  const secondaryKeys = await generateKeys(1, secondaryDevSeed);
  await createIdentities(alice, primaryKeys);

  const ticker = generateRandomTicker();
  const earnedTicker = generateRandomTicker();

  await distributePolyBatch(alice, [primaryKeys[0]], transferAmount);
  await addSecondaryKeys(primaryKeys, secondaryKeys);
  await authorizeJoinToIdentities(secondaryKeys, primaryKeys);
  await distributePolyBatch(alice, [secondaryKeys[0], bob], transferAmount);

  console.log("Distributing tokens");
  await Promise.all([
    issueTokenToDid(alice, ticker, 1000000, null),
    issueTokenToDid(alice, earnedTicker, 200000, null),
  ]);

  console.log("adding compliance requirement");
  await Promise.all([
    addComplianceRequirement(alice, ticker),
    addComplianceRequirement(alice, earnedTicker),
  ]);

  console.log("transfering token to Bob");
  let venueCounter = await createVenue(alice);
  let intructionCounterAB = await addInstruction(
    alice,
    venueCounter,
    aliceDid,
    bobDid,
    ticker,
    100
  );
  console.log("affirming transfer");
  await Promise.all([
    affirmInstruction(alice, intructionCounterAB, aliceDid, 1),
    affirmInstruction(bob, intructionCounterAB, bobDid, 0),
  ]);

  console.log("changing default target and taxes for corporate action");
  await Promise.all([
    changeDefaultTargetIdentitites(alice, ticker, [bob], "include"),
    changeWithholdingTax(alice, ticker, 15),
  ]);

  await initiateCorporateAction(
    alice,
    ticker,
    "PredictableBenefit",
    "100",
    { existing: 1 },
    "Regular dividend",
    null,
    null,
    null
  );

  console.log("creating distribution");
  await createDistribution(
    alice,
    ticker,
    "0",
    null,
    earnedTicker,
    100,
    100000,
    null
  );

  await claimDistribution(bob, ticker, 0);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: CORPORATE_ACTIONS");
    process.exit();
  });
