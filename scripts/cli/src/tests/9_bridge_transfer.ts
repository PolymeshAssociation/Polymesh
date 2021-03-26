import { initMain, sleep } from "../util/init";
import { bridgeTransfer, freezeTransaction, unfreezeTransaction } from "../helpers/bridge_helper";

async function main(): Promise<void> {
	const testEntities = await initMain();
	const alice = testEntities[0];
	const relay = testEntities[1];
	await bridgeTransfer(relay, alice);
	await freezeTransaction(alice);
	await sleep(50000).then(async () => {
		await unfreezeTransaction(alice);
	});
}

main()
	.catch((err: unknown) => {
		if (err instanceof Error) {
			console.log(`Error: ${err.message}`);
		}
	})
	.finally(() => process.exit());
