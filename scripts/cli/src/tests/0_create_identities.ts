import { initMain, generateRandomKey, generateKeys } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const primaryDevSeed = generateRandomKey();
  const keys = await generateKeys(2, primaryDevSeed);
  await createIdentities(alice, keys);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: CREATE IDENTITIES");
    process.exit();
  });