import * as init from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const ticker = init.generateRandomTicker();
  const testEntities = await init.initMain();
  const alice = testEntities[0];
  const primaryDevSeed = init.generateRandomKey();
  const primaryKeys = await init.generateKeys(1, primaryDevSeed);
  await createIdentities(alice, primaryKeys);
  await distributePolyBatch(alice, primaryKeys, init.transferAmount);
  await issueTokenToDid(primaryKeys[0], ticker, 1000000, null);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
