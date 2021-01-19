// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("../util/init.js");

let { reqImports } = require("../util/init.js");
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

    let txBatch = [];
    let senderDid = await api.query.identity.keyToIdentityIds(sender.publicKey);

    for (i = 0; i < receivers.length; i++) {
        let tx = await addAndAffirmInstruction(api, venueId, senderDid, receivers[i][0], ticker, amount);
        txBatch.push(tx);
    }

    let completeBatchTx = await api.tx.utility.batchAtomic(txBatch);  
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

