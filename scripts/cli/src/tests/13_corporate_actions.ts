import {
  generateKeys,
  transferAmount,
  initMain,
  generateRandomKey,
  generateRandomTicker,
  keyToIdentityIds,
  generateRandomEntity,
} from "../util/init";
import PrettyError from "pretty-error";
import {
  changeDefaultTargetIdentitites,
  changeWithholdingTax,
  initiateCorporateAction,
  linkCaToDoc,
  recordDateChange,
  removeCa,
} from "../helpers/corporate_actions_helper";
import { addAuthorization, createIdentities } from "../helpers/identity_helper";
import { addDocuments, issueTokenToDid } from "../helpers/asset_helper";
import { distributePoly, distributePolyBatch } from "../helpers/poly_helper";
import {
  acceptBecomeAgent,
  createGroup,
  nextAgId,
  setGroupPermissions,
} from "../helpers/external_agent_helper";
import { Document, ExtrinsicPermissions } from "../types";
import { setDoc } from "../helpers/permission_helper";

let documents: Document[] = [];
async function main(): Promise<void> {
  const testEntities = await initMain();
  const alice = testEntities[0];
  const aliceDid = await keyToIdentityIds(alice.publicKey);
  const bob = await generateRandomEntity();
  const bobDid = (await createIdentities(alice, [bob]))[0];
  console.log("Identities Created");

  let extrinsics: ExtrinsicPermissions = { These: [] };
  await distributePoly(alice, bob, transferAmount);
  const ticker = generateRandomTicker();
  await issueTokenToDid(alice, ticker, 1000000, null);

  await changeDefaultTargetIdentitites(alice, ticker, [bob], "exclude");
  await changeWithholdingTax(alice, ticker, 15);
  await initiateCorporateAction(
    alice,
    ticker,
    "PredictableBenefit",
    "100",
    null,
    "Regular dividend",
    null,
    null,
    null
  );
  // const doc = {
  //   uri: "https://example.com",
  //   content_hash: "H512",
  //   name: "Example Doc",
  // };
  setDoc(documents, "www.google.com", { None: "" }, "google");
  await addDocuments(alice, ticker, documents);
  await linkCaToDoc(alice, ticker, 0, []); // we probably want to link real docs
  await recordDateChange(alice, ticker, "0", null);
  await removeCa(alice, ticker, 0);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: CORPORATE_ACTIONS");
    process.exit();
  });
