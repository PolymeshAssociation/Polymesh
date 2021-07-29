import { initMain, generateRandomEntity, generateRandomTicker } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { issueTokenToDid } from "../helpers/asset_helper";
import { createGroup, setGroupPermissions } from "../helpers/external_agent_helper";
import { ExtrinsicPermissions } from "../types";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const bob = await generateRandomEntity();
  const bobDid = (await createIdentities(alice, [bob]))[0];
  let extrinsics: ExtrinsicPermissions = { These: [] };

  // Mint tokens
  const ticker = generateRandomTicker();
  await issueTokenToDid(alice, ticker, 1000000, null);

  // Create Group
  await createGroup(alice, ticker, extrinsics);

  // Set Group Permissions
  await setGroupPermissions(alice, ticker, 1, extrinsics);
  


}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
