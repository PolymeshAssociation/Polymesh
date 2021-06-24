import {
  initMain,
  generateRandomKey,
  generateKeys,
  transferAmount,
} from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const primaryDevSeed = generateRandomKey();
  const keys = await generateKeys(2, primaryDevSeed);
  await createIdentities(alice, keys);
  await distributePolyBatch(alice, keys, transferAmount);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
