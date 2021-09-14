import { ApiSingleton } from "../util/init";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
	const api = await ApiSingleton.getInstance();
	// We only display a couple, then unsubscribe
	let count = 0;
	let bridgeTxCounter = 0;
	const BRIDGE_LIMIT = 1000;
	const BRIDGE_POLY_TOTAL = "";
	const BRIDGE_TIME_PERIOD = "";

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
					// Show what we are busy with
					console.log(
						`\t${event.section}:${event.method}:: (phase=${phase.toString()})`
					);
					console.log(`\t\t${event.meta.documentation.toString()}`);

					// Loop through each of the parameters, displaying the type and data
					event.data.forEach((data, index) => {
						console.log(`\t\t\t${types[index].type}: ${data.toString()}`);
					});
				}
			});
		});

		//placeholder for now while testing
		if (++count === 256) {
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
