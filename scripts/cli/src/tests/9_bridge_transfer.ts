import { createApi, initMain, sleep } from "../util/init";
import { bridgeTransfer, freezeTransaction, unfreezeTransaction } from "../helpers/bridge_helper";

async function main(): Promise<void> {
		const api = await createApi();
		const testEntities = await initMain(api.api);
		const alice = testEntities[0];
		const relay = testEntities[1];
		await bridgeTransfer(api.api, relay, alice);
		await freezeTransaction(api.api, alice);
		await sleep(50000).then(async () => {
			await unfreezeTransaction(api.api, alice);
		});
}

main().catch((err) => console.log(`Error: ${err.message}`)).finally(() => process.exit());