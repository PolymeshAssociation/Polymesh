import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import {
  addSecondaryKeys,
  createMultiSig,
} from "../helpers/key_management_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await init.initMain();
  const primaryDevSeed = init.generateRandomKey();
  const secondaryDevSeed = init.generateRandomKey();
  const alice = testEntities[0];
  const bob = await init.generateRandomEntity();
  const charlie = await init.generateRandomEntity();
  const dave = await init.generateRandomEntity();
  const primaryKeys = await init.generateKeys(2, primaryDevSeed);
  const secondaryKeys = await init.generateKeys(2, secondaryDevSeed);
  const bobSignatory = await init.signatory(alice, bob);
  const charlieSignatory = await init.signatory(alice, charlie);
  const daveSignatory = await init.signatory(alice, dave);
  const signatoryArray = [bobSignatory, charlieSignatory, daveSignatory];
  await createIdentities(alice, primaryKeys);
  await distributePolyBatch(alice, primaryKeys, init.transferAmount);
  await addSecondaryKeys(primaryKeys, secondaryKeys);
  await createMultiSig(alice, signatoryArray, 2);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: POLY TRANSFERS");
    process.exit();
  });