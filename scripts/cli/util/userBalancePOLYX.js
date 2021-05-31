// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

const { ExportToCsv } = require("export-to-csv");
const cliProgress = require("cli-progress");
let { reqImports } = require("./init.js");

async function main() {
	const api = await reqImports.createApi();
	await getUserBalances(api);
	process.exit();
}

// Gets user balance and stores them in an array
async function getUserBalances(api) {
	// create new progress bar
	const progressBar = new cliProgress.SingleBar({
		format: "progress [{bar}] {percentage}% || {value}/{total} || ETA: {eta_formatted}",
		barCompleteChar: "\u2588",
		barIncompleteChar: "\u2591",
		hideCursor: true,
	});
	const listOfDids = await api.query.identity.didRecords.entries();
	let empty_did = "0x0000000000000000000000000000000000000000000000000000000000000000";
	let data = [];

	// initializes the progress bar
	progressBar.start(listOfDids.length - 1, 0, {
		speed: "N/A",
	});

	for (let i = 0; i < listOfDids.length; i++) {
		let pk = listOfDids[i][1]["primary_key"];
		let listOfSecondaryKeyObjects = listOfDids[i][1]["secondary_keys"];

		for (let j = 0; j < listOfSecondaryKeyObjects.length; j++) {
			// secondary key
			let sk = JSON.parse(listOfSecondaryKeyObjects[j]["signer"]).Account;
			// get sk did
			let skDid = await api.query.identity.keyToIdentityIds(sk);
			// getting secondary key balance info
			let skAccountData = await api.query.system.account(sk);
			
			pushData(data, skDid, empty_did, skAccountData, sk, false);
		}

		let did = await api.query.identity.keyToIdentityIds(pk);
		let accountData = await api.query.system.account(pk);

		pushData(data, did, empty_did, accountData, pk, true);
		// update the progress bar
		progressBar.increment();
	}

	await createCSV(data);
	// stop the progress bar
	progressBar.stop();
}

function pushData(dataArray, did, emptyDid, accountData, key, keyType) {
	if (did.toString() != emptyDid) {
		dataArray.push({
			AccountId: key,
			isPrimaryKey: keyType,
			Identity: did,
			free: JSON.stringify(accountData.data.free),
			reserved: JSON.stringify(accountData.data.reserved),
			miscFrozen: JSON.stringify(accountData.data.miscFrozen),
		});
	}
}

// Creates CSV file using an array of user balances
async function createCSV(data) {
	const options = {
		fieldSeparator: ",",
		quoteStrings: "",
		decimalSeparator: ".",
		showLabels: false,
		showTitle: false,
		useTextFile: false,
		useBom: true,
		useKeysAsHeaders: true,
	};

	const csvExporter = new ExportToCsv(options);
	const csvData = csvExporter.generateCsv(data, true);
	reqImports.fs.writeFileSync("usersPolyxBalance.csv", csvData);
}

main().catch(console.error);
