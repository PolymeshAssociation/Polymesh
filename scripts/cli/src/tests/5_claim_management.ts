import {
  initMain,
  generateKeys,
  transferAmount,
  ApiSingleton,
  sendTx,
  padTicker,
} from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePolyBatch } from "../helpers/poly_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const api = await ApiSingleton.getInstance();

  const testEntities = await initMain();
  const alice = testEntities[0];

  const issuerSeed1 = "5_issuer_1";
  const issuerSeed2 = "5_issuer_2";
  const claimeeSeed = "5_claimee";

  const issuerKeys1 = await generateKeys(2, issuerSeed1);
  const issuerKeys2 = await generateKeys(2, issuerSeed2);
  const claimeeKeys = await generateKeys(2, claimeeSeed);

  await createIdentities(alice, issuerKeys1);
  await createIdentities(alice, issuerKeys2);
  const claimeeDids = await createIdentities(alice, claimeeKeys);

  const ticker = padTicker("5TICKER");

  await distributePolyBatch(
    alice,
    issuerKeys1.concat(issuerKeys2).concat(claimeeKeys),
    transferAmount
  );

  console.log("Adding Exempted claim");
  await sendTx(
    issuerKeys1[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { Exempted: { Identity: claimeeDids[1] } },
      null
    )
  );

  console.log("Adding SellLockup claim");
  await sendTx(
    issuerKeys1[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { SellLockup: { Ticker: ticker } },
      null
    )
  );

  console.log("Adding Accredited claim");
  await sendTx(
    issuerKeys2[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { Accredited: { Ticker: ticker } },
      null
    )
  );

  console.log("Adding Affiliate claim");
  await sendTx(
    issuerKeys2[0],
    api.tx.identity.addClaim(
      claimeeDids[0],
      { Affiliate: { Ticker: ticker } },
      Date.now() as any
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
