import chalk from "chalk";
import { getAPI, generateEntity, getCddProvider } from "./util/init";
import createIdentities from "./helpers/identity_helper";
//const common = require('./common/common_functions');
//const input = require('./IO/input');

export default async function executeApp(entityName: string, topup: boolean) {
  // common.logAsciiBull();
  console.log("********************************************");
  console.log("Welcome to the Command-Line Account Identity Generator.");
  console.log("********************************************");
  console.log(
    "The following script will create a new Identity according to the parameters you entered."
  );

  try {
    // Get API
    let api = getAPI();

    // Get Entities
    let entity = await generateEntity(api, entityName);
    let cdd_provider = getCddProvider();

    // Create Identity
    let did = await createIdentities(api, [entity], cdd_provider, topup);

    console.log(`DID Address: ${JSON.stringify(did)}`);
  } catch (err) {
    console.log(err);
  }
}
