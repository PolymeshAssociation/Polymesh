import { generateEntityFromUri, initMain, transferAmount } from "../util/init";
import { createIdentities, getAuthId } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import * as relayer from "../helpers/relayer_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const bob = await generateEntityFromUri("16_bob");
  await createIdentities(alice, [bob]);
  await distributePoly(alice, bob, transferAmount);
  console.log("Set Paying Key");
  await relayer.setPayingKey(alice, bob.publicKey, 100000);
  const authId = await getAuthId();
  console.log("Accept Paying Key");
  await relayer.acceptPayingKey(bob, authId.toNumber());
  console.log("Update POLYX Limit");
  await relayer.updatePolyxLimit(alice, bob.publicKey, 500000);
  console.log("Increase POLYX Limit");
  await relayer.increasePolyxLimit(alice, bob.publicKey, 70000);
  console.log("Decrease POLYX Limit");
  await relayer.decreasePolyxLimit(alice, bob.publicKey, 30000);
  console.log("Remove Paying Key");
  await relayer.removePayingKey(alice, bob, "userKey");
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
