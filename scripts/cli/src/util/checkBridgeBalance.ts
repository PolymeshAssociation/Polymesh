import { ApiSingleton } from "../util/init";
import PrettyError from "pretty-error";
import * as prom from "./promClient.js";
import express from "express";

/// Express
const app = express();
const port = 5556;

async function main(): Promise<void> {
	type user = {
		recipient: string;
		polyx: number;
		lastTimeBridged: number;
	}[];
	prom.injectMetricsRoute(app);
	prom.startCollection();
	console.log(
		`Starting events monitor watching endpoint: ${process.env.WS_PROVIDER}`
	);
	const api = await ApiSingleton.getInstance();
	const BRIDGE_LIMIT = 500_000_000;
	const BRIDGE_POLY_TOTAL = 1_000_000_000;
	const BRIDGE_TIME_PERIOD = hoursToSeconds(2); //e.g 4 * 3600
	let count = 0;
	let userArray: user = [];

	function currentTimeToSeconds() {
		const currentDateTime = new Date();
		const resultInSeconds = currentDateTime.getTime() / 1000;
		return resultInSeconds;
	}

	function hoursToSeconds(hours: number) {
		let seconds = hours * 3600;
		return seconds;
	}

	function checksUser(
		userArray: user,
		newPolyx: any,
		offenderAddress: any,
		blockNumber: number
	) {
		userArray.forEach((user, index) => {
			if (user.recipient === offenderAddress) {
				const timeDifference =
					currentTimeToSeconds() - userArray[index].lastTimeBridged;

				//check timeDifference to make sure its not less than 16mins
				// if it is then no modifications should be done as
				// seperate events have the same information
				if (timeDifference > hoursToSeconds(0.25)) {
					const newBalance = userArray[index].polyx + newPolyx;
					if (
						newBalance > BRIDGE_POLY_TOTAL &&
						timeDifference <= BRIDGE_TIME_PERIOD
					) {
						console.log(`Error: Total POLYX Bridge limit exceeded.`);
						console.log(
							"Polyx Total Bridge Limit Equivocation: ",
							offenderAddress,
							blockNumber
						);
						prom.polyxTotalBridgeLimitEquivocations.inc({ offenderAddress });
						if (index > -1) userArray.splice(index, 1);
					} else {
						console.log("Updates user data.");
						userArray[index].polyx = newBalance;
						userArray[index].lastTimeBridged = currentTimeToSeconds();
					}
				}
			}
		});
	}

	// Subscribe to the new headers on-chain. The callback is fired when new headers
	// are found, the call itself returns a promise with a subscription that can be
	// used to unsubscribe from the newHead subscription
	const unsubscribe = await api.rpc.chain.subscribeNewHeads((header) => {
		console.log(`Chain is at block: #${header.number}`);

		// Subscribe to system events via storage
		api.query.system.events((events) => {
			console.log(`\nReceived ${events.length} events:`);

			events
				.map((record) => {
					return record.event;
				})
				.filter((event) => {
					return (
						event.section === "bridge" && event.method === "BridgeTxScheduled"
					);
				})
				.forEach((event) => {
					const types = event.typeDef;

					event.data
						.filter((data, index) => types[index].type === "BridgeTx")
						.forEach((data) => {
							const txData = JSON.parse(data.toString());
							const newPolyx = txData.value;
							const offenderAddress = txData.recipient;
							const userExist = userArray.some(
								(user) => user.recipient === offenderAddress
							);

							if (userExist) {
								console.log("User exists.");
								checksUser(
									userArray,
									newPolyx,
									offenderAddress,
									header.number.toNumber()
								);
							} else {
								console.log("User hasn't been added to array yet.");
								userArray.push({
									recipient: offenderAddress,
									polyx: newPolyx,
									lastTimeBridged: currentTimeToSeconds(),
								});
							}
							if (newPolyx > BRIDGE_LIMIT) {
								console.log(`Error: Transaction POLYX limit exceeded.`);
								console.log(
									"Polyx Bridge Limit Equivocation: ",
									offenderAddress,
									header.number
								);
								prom.polyxBridgeLimitEquivocations.inc({ offenderAddress });
							}
						});
				});
		});

		app.listen(port, () =>
			console.log(`Offences monitor running on port ${port}`)
		);
	});
}

main().catch((err: any) => {
	const pe = new PrettyError();
	console.error(pe.render(err));
	process.exit(1);
});
