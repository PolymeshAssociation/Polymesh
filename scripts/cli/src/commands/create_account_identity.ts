

import chalk from 'chalk';
import { getAPI, generateEntity } from "./util/init";
import createIdentities from "./helpers/identity_helper";
//const common = require('./common/common_functions');
//const input = require('./IO/input');


export default async function executeApp (entityName: string) {
 // common.logAsciiBull();
  console.log("********************************************");
  console.log("Welcome to the Command-Line Account Identity Generator.");
  console.log("********************************************");
  console.log("The following script will create a new Identity according to the parameters you entered.");
  
//   await setup();

  try {
    // Get API
    let api = getAPI();

    // Get Entities
    let entity = await generateEntity(api, entityName);
    let alice = await generateEntity(api, "Alice");

    // Create Identity
    let did = await createIdentities(api, [entity], alice );

    console.log(`DID Address: ${JSON.stringify(did)}`);

  } catch (err) {
    console.log(err);
  }
};

// async function setup () {
//   try {
   
//   } catch (err) {
//     console.log(err)
//     process.exit(0);
//   }
// }

