import { initMain, sleep, disconnect } from "../util/init";
import {
  bridgeTransfer,
  freezeTransaction,
  unfreezeTransaction,
} from "../helpers/bridge_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("9 - Bridge Transfer Unit Test", () => {
  test("Bridging Transaction", async () => {
    const testEntities = await initMain();
    const relay = testEntities[1];
    const admin = testEntities[2];
    await expect(bridgeTransfer(relay, admin)).resolves.not.toThrow();
    await expect(freezeTransaction(admin)).resolves.not.toThrow();
    await sleep(5000);
    await expect(unfreezeTransaction(admin)).resolves.not.toThrow();
  });
});
