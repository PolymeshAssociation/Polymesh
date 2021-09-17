import { ApiSingleton } from "../util/init";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
	const api = await ApiSingleton.getInstance();
	const BRIDGE_LIMIT = 500_000_000;
	const BRIDGE_POLY_TOTAL = 1_000_000_000;
	const BRIDGE_TIME_PERIOD = hoursToSeconds(2); //e.g 4 * 3600
	let count = 0;
	let userArray: {
		recipient: string;
		polyx: number;
		lastTimeBridged: number;
	}[] = [];

	function currentTimeToSeconds() {
		const currentDateTime = new Date();
		const resultInSeconds = currentDateTime.getTime() / 1000;
		return resultInSeconds;
	}

	function hoursToSeconds(hours: number) {
		let seconds = hours * 3600;
		return seconds;
	}

	// Subscribe to the new headers on-chain. The callback is fired when new headers
	// are found, the call itself returns a promise with a subscription that can be
	// used to unsubscribe from the newHead subscription
	const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
		console.log(`Chain is at block: #${header.number}`);

		// Subscribe to system events via storage
		api.query.system.events((events) => {
			console.log(`\nReceived ${events.length} events:`);

			// Loop through the Vec<EventRecord>
			events.forEach((record) => {
				// Extract the phase, event and the event types
				const { event, phase } = record;
				const types = event.typeDef;
				if (event.section === "bridge") {
					if (event.method === "BridgeTxScheduled") {
						// Loop through each of the parameters, displaying the type and data
						event.data.forEach((data, index) => {
							if (types[index].type === "BridgeTx") {
								const txData = JSON.parse(data.toString());

								if (
									userArray.length === 0 ||
									!userArray.some((user) => user.recipient === txData.recipient)
								) {
									console.log("User hasn't been added to array yet.");
									userArray.push({
										recipient: txData.recipient,
										polyx: txData.value,
										lastTimeBridged: currentTimeToSeconds(),
									});
								} else {
									console.log("User exists.");

									const index = userArray.findIndex(
										(user) => user.recipient === txData.recipient
									);
									const timeDifference =
										currentTimeToSeconds() - userArray[index].lastTimeBridged;

									//check timeDifference to make sure its not less than 16mins
									// if it is then no modifications should be done as
									// seperate events have the same information
									if (timeDifference > hoursToSeconds(0.25)) {
										const newBalance = userArray[index].polyx + txData.value;
										if (
											newBalance > BRIDGE_POLY_TOTAL &&
											timeDifference <= BRIDGE_TIME_PERIOD
										) {
											console.log(`Error: Total POLYX Bridge limit exceeded.`);
											if (index > -1) userArray.splice(index, 1);
										} else {
											console.log("Updates user data.");
											userArray[index].polyx = newBalance;
											userArray[index].lastTimeBridged = currentTimeToSeconds();
										}
									}
								}

								if (txData.value > BRIDGE_LIMIT)
									console.log(`Error: Transaction POLYX limit exceeded.`);
							}
						});
					}
				}
			});
		});

		//placeholder for now while testing
		if (++count === 512) {
			unsubscribe();
			process.exit(0);
		}
	});
}

main().catch((err: any) => {
	const pe = new PrettyError();
	console.error(pe.render(err));
	process.exit(1);
});
