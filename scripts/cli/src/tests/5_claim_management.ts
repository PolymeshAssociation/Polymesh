import {
  initMain,
  generateRandomKey,
  generateKeys,
  transferAmount,
  ApiSingleton,
  sendTx,
  generateRandomTicker,
} from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const api = await ApiSingleton.getInstance();

  const testEntities = await initMain();
  const alice = testEntities[0];

  const issuerSeed1 = "fooIssuer";
  const issuerSeed2 = "barIssuer";
  const claimeeSeed = "theClaimee";

  const issuerKeys1 = await generateKeys(2, issuerSeed1);
  const issuerKeys2 = await generateKeys(2, issuerSeed2);
  const claimeeKeys = await generateKeys(2, claimeeSeed);

  await createIdentities(alice, issuerKeys1);
  await createIdentities(alice, issuerKeys2);
  const claimeeDids = await createIdentities(alice, claimeeKeys);

  const ticker = "FOOBARTICKER";

  await distributePolyBatch(
    alice,
    issuerKeys1.concat(issuerKeys2).concat(claimeeKeys),
    transferAmount
  );

  await sendTx(
    issuerKeys1[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { Exempted: { Identity: claimeeDids[1] } },
      null
    )
  );

  await sendTx(
    issuerKeys1[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { SellLockup: { Ticker: ticker } },
      null
    )
  );

  await sendTx(
    issuerKeys2[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { Accredited: { Ticker: ticker } },
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
    console.log("Completed: CLAIM MANAGEMENT");
    process.exit();
  });
