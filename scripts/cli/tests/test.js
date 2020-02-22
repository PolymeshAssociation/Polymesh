// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

const {
  path,
  fs,
  ApiPromise,
  WsProvider,
  keyring,
  createIdentities,
  BN,
  nonces
} = require("../util/init.js");

async function main() {
  // Schema path
  const filePath = path.join(__dirname + "/../../../polymesh_schema.json");
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new WsProvider("ws://127.0.0.1:9944/");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });

//Create alice (carry-over from the keyring section)
const alice = keyring.addFromUri('//Alice', { name: 'Alice' });
let aliceRawNonce = await api.query.system.accountNonce(alice.address);
let alice_nonce = new BN(aliceRawNonce.toString());
nonces.set(alice.address, alice_nonce);

// const bob = keyring.addFromUri('//Bob', { name: 'Bob' });
// let bobRawNonce = await api.query.system.accountNonce(bob.address);
// let bob_nonce = new BN(bobRawNonce.toString());
// nonces.set(bob.address, bob_nonce);

let sammy = keyring.addFromUri('//Sammy', { name: 'Sammy' });
let sammyRawNonce = await api.query.system.accountNonce(sammy.address);
let sammy_nonce = new BN(sammyRawNonce.toString());
nonces.set(sammy.address, sammy_nonce);


// This code as it is right now makes it applicable to be able to 
// create custom entities with their own names just we have to first supply
// them with tokens before they can create an identity 
//
// next to solve the run the same function without errors of it trying to 
// recreate identities that already exist what I will do is use the keyring.getPair()
// function to pull the object if it exists and if it does not then I would allow 
// for the identity to be created..God is good 







await createIdentities(api, [alice], true);

// try {
// keyring.getPair("0x565965b1f92b552d734e9ff8b01324eda51d80958115771461e6473f99523ca1");
// }catch(e) {console.log("We Got An Error");}


// if(keyring.getPair("0x565965b1f92b552d734e9ff8b01324eda51d80958115771461e6473f99523ca1")) {
//   console.log("Can Find It");
// }
// else {
//   console.log("Can't Find It");
// }

// Get the current sudo key in the system
const sudoKey = await api.query.sudo.key().catch(() => {console.log("Error Caught 1");});
// Lookup from keyring (assuming we have added all, on --dev this would be `//Alice`)
const sudoPair = keyring.getPair(sudoKey.toString());

// Send the actual sudo transaction
const unsub = await api.tx.sudo
  .sudo(
    api.tx.balances.setBalance(sammy.address, 40000, 20000)
  )
  .signAndSend(sudoPair, (result) => {  })
  .catch(() => {console.log("Error Caught 3");});




//Make a transfer from Alice to BOB, waiting for inclusion
// const unsub = await api.tx.balances.transfer(sammy.address, 20000).signAndSend(alice, (result) => {
//     console.log(`Current status is ${result.status}`);

//     if (result.status.isFinalized) {
//       console.log(`Transaction included at blockHash ${result.status.asFinalized}`);
//       unsub();
//     }
//   }).catch(() => {
//       console.log("It Failed.");
//   });  

// console.log("Done");
process.exit();
}

main();