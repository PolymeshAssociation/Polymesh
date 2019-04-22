// Required imports
// @ts-check
import { ApiPromise, WsProvider } from '@polkadot/api';
import { Option, u128 } from '@polkadot/types';
import inquirer from 'inquirer';
import BN from 'bn.js';

const Alice = '5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY';
const customTypes = {
  "TokenBalance": "u128",
  "SecurityToken": {
    "name": "Vec<u8>",
    "total_supply": "u128",
    "owner": "AccountId"
  },
  "Restriction": {
    "name": "Vec<u8>",
    "token_id": "u32",
    "can_transfer": "bool"
  },
  "Whitelist": {
    "investor": "AccountId",
    "canSendAfter": "Moment",
    "canReceiveAfter": "Moment"
  },
  "Issuer": {
    "account": "AccountId",
    "access_level": "u16",
    "active": "bool"
  },
  "Investor": {
    "account": "AccountId",
    "access_level": "u16",
    "active": "bool",
    "jurisdiction": "u16"
  },
  "FeeOf": "Balance"
};
const initialQuestions = [
  {
    type: 'input',
    name: 'ticker',
    message: "Enter the token ticker: ",
    default: 'HW'
  },
  {
    type: 'input',
    name: 'checkpoint',
    message: "Enter the checkpoint number: ",
    default: 1
  }
];
const userAddress = [
  {
    type: 'input',
    name: 'address',
    message: "Enter user to address to query their checkpoint balance or 0 to exit: ",
    default: Alice
  }
];

async function main (answers) {
  const wsProvider = new WsProvider('ws://127.0.0.1:9944');
  const api = await ApiPromise.create({
    types: customTypes,
    provider: wsProvider
  });
  const tickerUpper = answers.ticker.toUpperCase();
  const tickerInfo = JSON.parse((await api.query.asset.tokens(tickerUpper)).toString());
  const cpTotalSupply =  await api.query.asset.checkpointTotalSupply([tickerUpper, answers.checkpoint]);
  console.log(`Total supply of ${new Buffer(tickerInfo.name).toString()} token at checkpoint ${answers.checkpoint} was ${cpTotalSupply}`);
  const input = await inquirer.prompt(userAddress);
  if (input.address != '0') {
    let userBalance;
    let maxCheckpoint = new BN(await api.query.asset.totalCheckpoints(tickerUpper)).toNumber();
    let fetchingOldBalance = false;
    if (answers.checkpoint < maxCheckpoint) {
      while (maxCheckpoint >= answers.checkpoint) {
        let bal: Option<u128> = await api.query.asset.checkpointBalance([tickerUpper, input.address, maxCheckpoint]) as unknown as Option<u128>;
        if (bal.isSome) {
          fetchingOldBalance = true;
          break;
        }
        maxCheckpoint--;
      }
      if (fetchingOldBalance) {
        maxCheckpoint = answers.checkpoint;
        while (maxCheckpoint > 0) {
          console.log(maxCheckpoint);
          let bal: Option<u128> = await api.query.asset.checkpointBalance([tickerUpper, input.address, maxCheckpoint]) as unknown as Option<u128>;
          if (bal.isSome) {
            userBalance = bal.value;
            break;
          }
          maxCheckpoint--;
        }
      } else {
        maxCheckpoint = 0;
      }
    } else {
      maxCheckpoint = 0;
    }
    if (maxCheckpoint == 0) {
      userBalance = new BN(await api.query.asset.balanceOf([tickerUpper, input.address])).toNumber();
    }
    console.log(`Balance of ${input.address} at checkpoint ${answers.checkpoint} was ${userBalance}`);
  }
  process.exit();
}

inquirer.prompt(initialQuestions).then(answers => main(answers).catch(console.error));
