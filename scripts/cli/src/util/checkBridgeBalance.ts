import { ApiSingleton } from "../util/init";
import PrettyError from "pretty-error";
import * as prom from "./promClient.js";
import express from "express";

/// Express
const app = express();
const port = 8888;

async function main(): Promise<void> {
	type userInfo = {
		polyx: number;
		lastTimeBridged: number;
	};
	prom.injectMetricsRoute(app);
	prom.startCollection();
	console.log(
		`Starting events monitor watching endpoint: ${process.env.WS_PROVIDER}`
	);
	app.listen(port, () =>
		console.log(`Offences monitor running on port ${port}`)
	);
	const api = await ApiSingleton.getInstance();
	const BRIDGE_LIMIT = 500_000_000;
	const BRIDGE_POLY_TOTAL = 1_000_000_000;
	const BRIDGE_TIME_PERIOD = await blocksToSeconds();
	let userArray: Map<string, userInfo> = new Map();

	async function blocksToSeconds() {
		const timelock = await api.query.bridge.timelock();
		return timelock.toNumber() * 6;
	}

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
		userMap: Map<string, userInfo>,
		newPolyx: any,
		offenderAddress: any,
		blockNumber: number
	) {
		const user = userMap.get(offenderAddress)!;
		const timeDifference = currentTimeToSeconds() - user.lastTimeBridged;

		//check timeDifference to make sure its not less than 16mins
		// if it is then no modifications should be done as
		// seperate events have the same information
		if (timeDifference > hoursToSeconds(0.25)) {
			const newBalance = user.polyx + newPolyx;

			checkPolyxLimit(newPolyx, offenderAddress, blockNumber);

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
				// resets polyx to zero to check for new errors that break polyx limit
				userMap.set(offenderAddress, {
					polyx: 0,
					lastTimeBridged: currentTimeToSeconds(),
				});
			} else {
				console.log("Updates user data.");
				userMap.set(offenderAddress, {
					polyx: newBalance,
					lastTimeBridged: currentTimeToSeconds(),
				});
			}
		}
	}

	function checkPolyxLimit(
		newPolyx: number,
		offenderAddress: string,
		blockNumber: number
	) {
		if (newPolyx > BRIDGE_LIMIT) {
			console.log(`Error: Transaction POLYX limit exceeded.`);
			console.log(
				"Polyx Bridge Limit Equivocation: ",
				offenderAddress,
				blockNumber
			);
			prom.polyxBridgeLimitEquivocations.inc({ offenderAddress });
		}
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
							const userExist = userArray.get(offenderAddress);

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
								checkPolyxLimit(
									newPolyx,
									offenderAddress,
									header.number.toNumber()
								);
								userArray.set(offenderAddress, {
									polyx: newPolyx,
									lastTimeBridged: currentTimeToSeconds(),
								});
							}
						});
				});
		});
	});
}

main().catch((err: any) => {
	const pe = new PrettyError();
	console.error(pe.render(err));
	process.exit(1);
});
