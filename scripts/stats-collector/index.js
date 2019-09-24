const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");

const fs = require("fs");
const path = require("path");
//import * as fs from 'fs';
//import * as path from 'path';

const BOB = "5FHneW46xGXgs5mUiveU4sbTyGBzmstUspZC92UhjJM694ty";

// Spams the network with transfer transaction in an attempt to measure base
// currency TPS. As of 2019-09-24 average value is 35-40
async function tps() {
  const filePath = path.join(
    __dirname + "/../polymesh/polymesh_substrate/substrateui_dev.json"
  );
  const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8").toString());

  const ws_provider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: ws_provider
  });
  const keyring = new Keyring({ type: "sr25519" });
  let accounts = [];
  accounts.push(keyring.addFromUri("//Alice", { name: "Alice" }));
  accounts.push(keyring.addFromUri("//Bob", { name: "Bob" }));
  accounts.push(keyring.addFromUri("//Charlie", { name: "Charlie" }));
  accounts.push(keyring.addFromUri("//Dave", { name: "Dave" }));

  console.time("Transactions sent to the node in");
  for (let i = 0; i < accounts.length; i++) {
    let rawNonce = await api.query.system.accountNonce(
      keyring.getPairs()[i].address
    );
    let nonce = new BN(rawNonce.toString());
    console.log("current account: " + accounts[i].meta.name);
    console.log("current account address: " + accounts[i].address);
    for (let j = 0; j < 250; j++) {
      const txHash = await api.tx.balances
        .transfer(BOB, 1000)
        .signAndSend(accounts[i], { nonce });
      nonce = nonce.add(new BN(1));
    }
  }
  const unsub = await api.rpc.chain.subscribeNewHeads((header) => {
    console.log("Block " + header.number + " Mined. Hash: " + header.hash);
  });
  console.timeEnd("Transactions sent to the node in");
  let i = 0;
  let j = 0;
  let oldPendingTx = 0;
  let interval = setInterval(async () => {
    await api.rpc.author.pendingExtrinsics(extrinsics => {
      i++;
      j++;
      if (oldPendingTx > extrinsics.length) {
        console.log("Approx TPS: ", (oldPendingTx - extrinsics.length) / j);
        j = 0;
      }
      if (extrinsics.length === 0) {
        console.log(i + " Second passed, No pending extrinsics in the pool.");
        clearInterval(interval);
        unsub();
        process.exit();
      }
      console.log(
        i +
          " Second passed, " +
          extrinsics.length +
          " pending extrinsics in the pool"
      );
      oldPendingTx = extrinsics.length;
    });
  }, 1000);
}

tps().catch(console.error);
