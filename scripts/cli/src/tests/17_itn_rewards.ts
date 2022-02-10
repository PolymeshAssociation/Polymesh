import { initMain, generateEntityFromUri } from "../util/init";
import { createIdentities } from "../helpers/identity_helper";
import { claimItnReward, setItnRewardStatus } from "../helpers/rewards_helper";
import { createTable } from "../util/sqlite3";
import PrettyError from "pretty-error";
import assert from "assert";

async function main(): Promise<void> {
  createTable();
  const testEntities = await initMain();
  const alice = testEntities[0];
  const dave = await generateEntityFromUri("17_dave");
  const charlie = await generateEntityFromUri("17_charlie");
  const dave2 = await generateEntityFromUri("17_dave2");
  console.log("Create Identity");
  await createIdentities(alice, [dave2]);
  console.log("Set ITN Rewards Claim Status");
  await setItnRewardStatus(alice, dave, { Unclaimed: 2_000_000 });
  console.log("Claim ITN Rewards");
  await claimItnReward(dave, dave2);
  console.log("Claim ITN Reward that doesn't exist");
  await assert.rejects(async () => {
    await claimItnReward(charlie, dave2);
  });
  console.log("Set ITN Rewards Claim Status for Charlie");
  await setItnRewardStatus(alice, charlie, { Unclaimed: 2_000_000 });
  console.log("Claim Charlie ITN Reward with Dave");
  await assert.rejects(async () => {
    await claimItnReward(dave, charlie);
  });
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => {
    console.log("Completed: ITN Rewards Test");
    process.exit();
  });
