import { initMain, generateKeys, transferAmount, throws } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePoly, distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const primaryDevSeed = "1_primary";
  const keys = await generateKeys(2, primaryDevSeed);
  await createIdentities(alice, keys);
  await distributePolyBatch(alice, keys, transferAmount);

  // fail transfers
  await throws(
    async () => await distributePoly(keys[0], keys[1], transferAmount * 10)
  );
  await throws(
    async () => await distributePoly(keys[1], keys[0], transferAmount * 10)
  );
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
