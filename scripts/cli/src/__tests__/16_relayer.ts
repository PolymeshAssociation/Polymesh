import {
  generateEntityFromUri,
  initMain,
  transferAmount,
  disconnect,
} from "../util/init";
import { createIdentities, getAuthId } from "../helpers/identity_helper";
import { distributePoly } from "../helpers/poly_helper";
import * as relayer from "../helpers/relayer_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("16 - Relayer Unit Test", () => {
  test("Relayer", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const bob = await generateEntityFromUri("16_bob");
    await expect(createIdentities(alice, [bob])).resolves.not.toThrow();
    await expect(
      distributePoly(alice, bob, transferAmount)
    ).resolves.not.toThrow();
    console.log("Set Paying Key");
    await expect(
      relayer.setPayingKey(alice, bob.publicKey, 100000)
    ).resolves.not.toThrow();
    const authId = await getAuthId();
    console.log("Accept Paying Key");
    await expect(
      relayer.acceptPayingKey(bob, authId)
    ).resolves.not.toThrow();
    console.log("Update POLYX Limit");
    await expect(
      relayer.updatePolyxLimit(alice, bob.publicKey, 500000)
    ).resolves.not.toThrow();
    console.log("Increase POLYX Limit");
    await expect(
      relayer.increasePolyxLimit(alice, bob.publicKey, 70000)
    ).resolves.not.toThrow();
    console.log("Decrease POLYX Limit");
    await expect(
      relayer.decreasePolyxLimit(alice, bob.publicKey, 30000)
    ).resolves.not.toThrow();
    console.log("Remove Paying Key");
    await expect(
      relayer.removePayingKey(alice, bob, "userKey")
    ).resolves.not.toThrow();
  });
});
