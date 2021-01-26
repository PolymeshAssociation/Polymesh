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
    const ticker = args.ticker; 
    const account = await reqImports.generateEntityFromUri(api, args.account);  
    const amount = args.amount; 
    const venueId = args.venueId;

    const listOfDids = await api.query.identity.didRecords.entries();
    
    await reqImports.issueTokenPerDid(api, [account], ticker, 100_000_000_000_000, null);
    
    await reqImports.addComplianceRequirement(api, account, ticker);
    
    await batchAtomic(api, account, listOfDids, amount, ticker, venueId);

    process.exit();
}

async function batchAtomic(api, sender, receivers, amount, ticker, venueId) {

    let tx;
    let txArray = [];
    let batch;
    let batchTx;
    let batchArray = [];
    let senderDid = await api.query.identity.keyToIdentityIds(sender.publicKey);
    let batchSize = 10;

    for (i = 0; i < receivers.length; i++) {
        if (receivers[i][0] != senderDid) {
            tx = await addAndAffirmInstruction(api, venueId, senderDid, receivers[i][0], ticker, amount);
            txArray.push(tx);
        }
    }

    
    for (i = 0; i < txArray.length; i += batchSize) {
        batch = txArray.slice(i, i + batchSize);
        batchTx = await api.tx.utility.batchAtomic(batch);
        batchArray.push(batchTx);
    }

    let completeBatchTx = await api.tx.utility.batchAtomic(batchArray);  
    await reqImports.sendTx(sender, completeBatchTx);  
}

async function addAndAffirmInstruction(api, venueId, senderDid, receiverDid, ticker, amount) {

    let senderPortfolio = reqImports.getDefaultPortfolio(senderDid);
    let receiverPortfolio = reqImports.getDefaultPortfolio(receiverDid);

    let leg = {
        from: senderPortfolio,
        to: receiverPortfolio,
        asset: ticker,
        amount: amount,
      };
     
   return await api.tx.settlement.addAndAffirmInstruction(venueId, 0, null, null, [leg], [senderPortfolio]);
}

main().catch(console.error);

