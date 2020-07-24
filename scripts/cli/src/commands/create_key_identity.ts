import chalk from "chalk";
import { getAPI, generateKeys, getCddProvider } from "./util/init";
import createIdentities from "./helpers/identity_helper";
//const common = require('./common/common_functions');
//const input = require('./IO/input');

export default async function executeApp(
  keyNumber: number,
  keyPrepend: string,
  topup: boolean
) {
  // common.logAsciiBull();
  console.log("********************************************");
  console.log("Welcome to the Command-Line Key Identity Generator.");
  console.log("********************************************");
  console.log(
    "The following script will create a new Identity according to the parameters you entered."
  );

  try {
    // Get API
    let api = getAPI();

    // Get CDD Provider
    let cdd_provider = getCddProvider();

    // Get Keys
    let keys = await generateKeys(api, keyNumber, keyPrepend);

    // Create Identity
    let dids = await createIdentities(api, keys, cdd_provider, topup);

    console.log(`DID Address/s: ${JSON.stringify(dids)}`);
  } catch (err) {
    console.log(err);
  }
}
