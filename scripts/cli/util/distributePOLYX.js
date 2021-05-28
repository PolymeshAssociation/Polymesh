// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

let { reqImports } = require("./init.js");
const minimist = require('minimist');
const args = minimist(process.argv.slice(2), {
    string: 'account'
});

async function main() {
    const api = await reqImports.createApi();  
    const account = await reqImports.generateEntityFromUri(api, args.account);  
    const amount = args.amount; 

    let empty_did = "0x0000000000000000000000000000000000000000000000000000000000000000"
    const listOfDids = await api.query.identity.didRecords.entries();
    let all_dids = new Array();
    
    for (let i = 0; i < listOfDids.length; i++) {
      let pk = listOfDids[i][1]['primary_key'];
      let did = await api.query.identity.keyToIdentityIds(pk);
      if (did.toString() != empty_did) {
        all_dids.push(did);
      }
    }    
    
    await batchAtomic(api, account, all_dids, amount);
    process.exit();
}

async function batchAtomic(api, sender, receivers, amount) {

    let tx;
    let txArray = [];
    let batch;
    let batchTx;
    let senderDid = await api.query.identity.keyToIdentityIds(sender.publicKey);
    let batchSize = 10;

    for (i = 0; i < receivers.length; i++) {
        if (receivers[i] != senderDid.toString()) {
            console.log("Prepping for DID: ", receivers[i].toString());
            tx = await api.tx.balances.transfer(receivers[i].address, amount);
            txArray.push(tx);
        } else {
            console.log("Skipping Sender");
        }
    }
  
    for (i = 0; i < txArray.length; i += batchSize) {
        batch = txArray.slice(i, i + batchSize);
        batchTx = await api.tx.utility.batchAtomic(batch);
        await reqImports.sendTx(sender, batchTx);  
    }
}

main().catch(console.error);

