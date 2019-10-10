// Required imports
// @ts-check
import { ApiPromise, WsProvider } from "@polkadot/api";
import { Option, u128 } from "@polkadot/types";
import inquirer from "inquirer";
import BN from "bn.js";
import * as fs from 'fs';
import * as path from 'path';

const Alice = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const filePath = path.join(__dirname + "../../../polymesh_schema.json");
const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8").toString());

const initialQuestions = [
  {
    type: "input",
    name: "ticker",
    message: "Enter the token ticker: ",
    default: "HW"
  },
  {
    type: "input",
    name: "checkpoint",
    message: "Enter the checkpoint number: ",
    default: 1
  }
];
const userAddress = [
  {
    type: "input",
    name: "address",
    message:
      "Enter user to address to query their checkpoint balance or 0 to exit: ",
    default: Alice
  }
];

async function main(answers) {
  const wsProvider = new WsProvider("ws://127.0.0.1:9944");
  const api = await ApiPromise.create({
    types: customTypes,
    provider: wsProvider
  });
  const tickerUpper = answers.ticker.toUpperCase();
  let maxCheckpoint = new BN(
    await api.query.asset.totalCheckpoints(tickerUpper)
  ).toNumber();
  if (answers.checkpoint > maxCheckpoint || answers.checkpoint <= 0) {
    throw new RangeError("Checkpoint does not exist");
  }
  const tickerInfo = JSON.parse(
    (await api.query.asset.tokens(tickerUpper)).toString()
  );
  const cpTotalSupply = await api.query.asset.checkpointTotalSupply([
    tickerUpper,
    answers.checkpoint
  ]);
  console.log(
    `Total supply of ${Buffer.from(
      tickerInfo.name
    ).toString()} token at checkpoint ${
      answers.checkpoint
    } was ${cpTotalSupply}`
  );
  const input = await inquirer.prompt(userAddress);
  if (input.address != "0") {
    let userBalance;
    let latestUserCheckpoint = new BN(
      await api.query.asset.latestUserCheckpoint([tickerUpper, input.address])
    ).toNumber();
    if (answers.checkpoint <= latestUserCheckpoint) {
      maxCheckpoint = answers.checkpoint;
      // Looing through all checkpoints from n to 0 to return balance.
      // According to the logic, Balance at n and n-1 is same if we didn't explicitly store balance for n.
      while (maxCheckpoint > 0) {
        let bal: Option<u128> = ((await api.query.asset.checkpointBalance([
          tickerUpper,
          input.address,
          maxCheckpoint
        ])) as unknown) as Option<u128>;
        if (bal.isSome) {
          userBalance = bal.value;
          break;
        }
        maxCheckpoint--;
      }
    } else {
      // No checkpoint balance stored for user. We should return latest balance.
      userBalance = new BN(
        await api.query.asset.balanceOf([tickerUpper, input.address])
      ).toNumber();
    }
    console.log(
      `Balance of ${input.address} at checkpoint ${answers.checkpoint} was ${userBalance}`
    );
  }
  process.exit();
}

inquirer.prompt(initialQuestions).then(answers =>
  main(answers).catch(err => {
    console.error(err);
    process.exit();
  })
);
