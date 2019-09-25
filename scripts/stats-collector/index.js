const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");
const cli = require("command-line-args");

const fs = require("fs");
const path = require("path");

const BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

const cli_opts = [
  {
    name: "order", // Order of magnitude, results in 10^order transactions per stage
    alias: "o",
    type: Number,
    defaultValue: 3
  }
];

async function main() {
  // Parse CLI args and compute tx count
  const opts = cli(cli_opts);
  let n_txes = 10 ** opts.order;

  console.log(
    "Welcome to Polymesh Stats Collector. Performing " +
      n_txes +
      " txes per stat."
  );

  const filePath = path.join(
    __dirname + "/../../../polymesh/polymesh_substrate/substrateui_dev.json"
  );
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8").toString());

  const ws_provider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });
  const keyring = new Keyring({ type: "sr25519" });
  /*
   *let accounts = [];
   *accounts.push(keyring.addFromUri("//Alice", { name: "Alice" }));
   *accounts.push(keyring.addFromUri("//Bob", { name: "Bob" }));
   *accounts.push(keyring.addFromUri("//Charlie", { name: "Charlie" }));
   *accounts.push(keyring.addFromUri("//Dave", { name: "Dave" }));
   */

  // Execute each stats collection stage
  console.log("=== TPS ===");
  await tps(api, keyring, n_txes); // base currency transfer sanity-check
  console.log("=== MESH315 SCENARIO === ");
  await mesh315(api, keyring, n_txes0);
}

// Spams the network with `n_txes` transfer transactions in an attempt to measure base
// currency TPS.
async function tps(api, keyring, n_txes) {
  let alice = keyring.addFromUri("//Alice", { name: "Alice" });
  let bob = keyring.addFromUri("//Bob", { name: "Bob" });

  console.time("Transactions sent to the node in");
  let aliceRawNonce = await api.query.system.accountNonce(alice.address);
  let aliceNonce = new BN(aliceRawNonce.toString());
  for (let j = 0; j < n_txes / 2; j++) {
    const txHash = await api.tx.balances
      .transfer(bob.address, 10)
      .signAndSend(alice, { nonce: aliceNonce });
    aliceNonce = aliceNonce.addn(1);
  }
  console.log("Alice -> Bob scheduled");

  let bobRawNonce = await api.query.system.accountNonce(bob.address);
  let bobNonce = new BN(bobRawNonce.toString());
  for (let j = 0; j < n_txes / 2; j++) {
    const txHash = await api.tx.balances
      .transfer(alice.address, 10)
      .signAndSend(bob, { nonce: bobNonce });
    bobNonce = bobNonce.addn(1);
  }
  console.log("Bob -> Alice scheduled");

  const unsub = await api.rpc.chain.subscribeNewHeads(header => {
    console.log("Block " + header.number + " Mined. Hash: " + header.hash);
  });
  console.timeEnd("Transactions sent to the node in");
  let i = 0;
  let j = 0;
  let oldPendingTx = 0;
  let still_polling = true;
  while (still_polling) {
    await api.rpc.author.pendingExtrinsics(extrinsics => {
      i++;
      j++;
      if (oldPendingTx > extrinsics.length) {
        console.log("Approx TPS: ", (oldPendingTx - extrinsics.length) / j);
        j = 0;
      }
      if (extrinsics.length === 0) {
        console.log(i + " Second passed, No pending extrinsics in the pool.");
        unsub();
        still_polling = false;
      }
      console.log(
        i +
          " Second passed, " +
          extrinsics.length +
          " pending extrinsics in the pool"
      );
      oldPendingTx = extrinsics.length;
    });

    // Wait one second
    await new Promise(resolve => setTimeout(resolve, 1000));
  }
}

main().catch(console.error);

