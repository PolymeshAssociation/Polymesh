

import chalk from 'chalk';
import { getAPI, generateEntity, generateKeys } from "./util/init";
import createIdentities from "./helpers/identity_helper";
//const common = require('./common/common_functions');
//const input = require('./IO/input');

//


// Artifacts


export default async function executeApp (entityName: string, keyNumber: number, keyPrepend: string) {
 // common.logAsciiBull();
  console.log("********************************************");
  console.log("Welcome to the Command-Line Identity Generator.");
  console.log("********************************************");
  console.log("The following script will create a new Identity according to the parameters you enter.");
  
//   await setup();

  try {
    // Get API
    let api = getAPI();

    // Get Entities
    let entity = await generateEntity(api, entityName);
    let alice = await generateEntity(api, "Alice");

    // Get Keys
    let keys = await generateKeys(api, keyNumber, keyPrepend);

    // Create Identity
    let did = await createIdentities(api, keys, alice );

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

