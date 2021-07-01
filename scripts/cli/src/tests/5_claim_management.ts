import {
  initMain,
  generateRandomKey,
  generateKeys,
  transferAmount,
} from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const primaryDevSeed = generateRandomKey();
  const claimDevSeed = generateRandomKey();
  const primaryKeys = await generateKeys(2, primaryDevSeed);
  const claimKeys = await generateKeys(2, claimDevSeed);
  const issuerDids = await createIdentities(alice, primaryKeys);
  const claimIssuerDids = await createIdentities(alice, claimKeys);
  await distributePolyBatch(
    alice,
    primaryKeys.concat(claimKeys),
    transferAmount
  );
  await addClaimsToDids(
    claimKeys[0],
    issuerDids[0],
    "Exempted",
    { Identity: claimIssuerDids[1] },
    null
  );
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
