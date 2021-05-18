// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

const { ExportToCsv } = require("export-to-csv");
let { reqImports } = require("./init.js");

async function main() {
	const api = await reqImports.createApi();
	await getUserBalances(api);
	process.exit();
}

// Gets user balances and stores them in an array
async function getUserBalances(api) {
	const listOfDids = await api.query.identity.didRecords.entries();
	let empty_did = "0x0000000000000000000000000000000000000000000000000000000000000000";
	let data = [];

	for (let i = 0; i < listOfDids.length; i++) {
		let pk = listOfDids[i][1]["primary_key"];
		let did = await api.query.identity.keyToIdentityIds(pk);
		let accountData = await api.query.system.account(pk);

		if (did.toString() != empty_did) {
			data.push({
				AccountId: pk,
				Identity: did,
				free: JSON.stringify(accountData.data.free),
				reserved: JSON.stringify(accountData.data.reserved),
				miscFrozen: JSON.stringify(accountData.data.miscFrozen),
			});
		}
	}
	await createCSV(data);
}

// Creates CSV file using an array of user balances
async function createCSV(data) {
	const options = {
		fieldSeparator: ",",
		quoteStrings: '',
		decimalSeparator: ".",
		showLabels: false,
		showTitle: false,
		useTextFile: false,
		useBom: true,
		useKeysAsHeaders: true,
	};

	const csvExporter = new ExportToCsv(options);
	const csvData = csvExporter.generateCsv(data, true);
  reqImports.fs.writeFileSync('usersPolyxBalance.csv',csvData);
}

main().catch(console.error);
