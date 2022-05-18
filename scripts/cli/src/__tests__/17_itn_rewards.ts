import { initMain, generateEntityFromUri, disconnect } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { claimItnReward, setItnRewardStatus } from "../helpers/rewards_helper";

// Disconnects api after all the tests have completed
afterAll(async () => {
  await disconnect();
});

describe("17 - ITN Rewards Unit Test", () => {
  test("ITN Rewards", async () => {
    const testEntities = await initMain();
    const alice = testEntities[0];
    const dave = await generateEntityFromUri("17_dave");
    const charlie = await generateEntityFromUri("17_charlie");
    const dave2 = await generateEntityFromUri("17_dave2");
    console.log("Create Identity");
    await expect(createIdentities(alice, [dave2])).resolves.not.toThrow();
    console.log("Set ITN Rewards Claim Status");
    await expect(
      setItnRewardStatus(alice, dave, { Unclaimed: 2_000_000 })
    ).resolves.not.toThrow();
    console.log("Claim ITN Rewards");
    await expect(claimItnReward(dave, dave2)).resolves.not.toThrow();
    console.log("Claim ITN Reward that doesn't exist");
    await expect(claimItnReward(charlie, dave2)).rejects.toThrow();
    console.log("Set ITN Rewards Claim Status for Charlie");
    await expect(
      setItnRewardStatus(alice, charlie, { Unclaimed: 2_000_000 })
    ).resolves.not.toThrow();
    console.log("Claim Charlie ITN Reward with Dave");
    await expect(claimItnReward(dave, charlie)).rejects.toThrow();
  });
});
