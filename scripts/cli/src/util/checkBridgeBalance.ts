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
	const bridgeData = await api.query.bridge.bridgeLimit();
	const BRIDGE_LIMIT = 500_000_000;
	const BRIDGE_POLY_TOTAL = bridgeData[0].toNumber();
	const BRIDGE_TIME_PERIOD = await blocksToSeconds(bridgeData[1].toNumber());
	let userTxs: Map<string, userInfo[]> = new Map();

	async function blocksToSeconds(numberOfBlocks: number) {
		return numberOfBlocks * 6;
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
		userMap: Map<string, userInfo[]>,
		newPolyx: any,
		offenderAddress: any,
		blockNumber: number
	) {
		const user = userMap.get(offenderAddress)!;
		// removes all transactions over BRIDGE_TIME_PERIOD
		removeOldTx(offenderAddress, user, userMap);
		let userPolyxSum = 0;
		user.forEach((data) => (userPolyxSum += data.polyx));
		userPolyxSum + newPolyx;

		checkPolyxLimit(newPolyx, offenderAddress, blockNumber);

		if (userPolyxSum > BRIDGE_POLY_TOTAL) {
			console.log(`Error: Total POLYX Bridge limit exceeded.`);
			console.log(
				"Polyx Total Bridge Limit Equivocation: ",
				offenderAddress,
				blockNumber
			);
			prom.polyxTotalBridgeLimitEquivocations.inc({ offenderAddress });
		} else {
			console.log("Updates user data.");
			user.push({
				polyx: newPolyx,
				lastTimeBridged: currentTimeToSeconds(),
			});
			userMap.set(offenderAddress, user);
		}
	}

	function removeOldTx(
		offenderAddress: string,
		user: userInfo[],
		userMap: Map<string, userInfo[]>
	) {
		const validTxArray = user.filter((data) => {
			const timeDifference = currentTimeToSeconds() - data.lastTimeBridged;
			return timeDifference < BRIDGE_TIME_PERIOD;
		});
		userMap.set(offenderAddress, validTxArray);
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
							const userExist = userTxs.get(offenderAddress);

							if (userExist) {
								console.log("User exists.");
								checksUser(
									userTxs,
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

								userTxs.set(offenderAddress, [
									{
										polyx: newPolyx,
										lastTimeBridged: currentTimeToSeconds(),
									},
								]);
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
