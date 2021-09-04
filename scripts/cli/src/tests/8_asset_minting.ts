import * as init from "../util/init";
import { createIdentities, addClaimsToDids } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import { issueTokenToDid, mintingAsset } from "../helpers/asset_helper";
import { addComplianceRequirement } from "../helpers/compliance_manager_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const ticker = init.padTicker("8TICKER");
  const testEntities = await init.initMain();
  const alice = testEntities[0];
  const primaryDevSeed = "8_primary";
  const primaryKey = (await init.generateKeys(1, primaryDevSeed))[0];
  let issuerDid = await createIdentities(alice, [primaryKey]);
  await distributePoly(alice, primaryKey, init.transferAmount);
  await issueTokenToDid(primaryKey, ticker, 1000000, null);
  await addClaimsToDids(
    primaryKey,
    issuerDid[0],
    "Exempted",
    { Ticker: ticker },
    null
  );
  await addComplianceRequirement(primaryKey, ticker);
  await mintingAsset(primaryKey, ticker);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: ASSET MINTING");
    process.exit();
  });
