import { initMain, sleep } from "../util/init";
import {
  bridgeTransfer,
  freezeTransaction,
  unfreezeTransaction,
} from "../helpers/bridge_helper";
import PrettyError from "pretty-error";

async function main(): Promise<void> {
  const testEntities = await initMain();
  const relay = testEntities[1];
  const admin = testEntities[2];
  await bridgeTransfer(relay, admin);
  await freezeTransaction(admin);
  await sleep(5000);
  await unfreezeTransaction(admin);
}

main()
  .catch((err: any) => {
    const pe = new PrettyError();
    console.error(pe.render(err));
    process.exit(1);
  })
  .finally(() => process.exit());
