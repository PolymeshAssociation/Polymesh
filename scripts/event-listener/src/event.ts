// @ts-check
// Required imports
import { ApiPromise, WsProvider } from '@polkadot/api';
import * as fs from 'fs';
import * as path from 'path';
import inquirer from 'inquirer';
import * as Utils from 'web3-utils';

const filePath = path.join(__dirname + "../../../../polymesh_schema.json");
const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8").toString());

console.log(`Welcome to a notification tracking device:\n`);

var openingQuestions = [
    {
        type: 'list',
        name: 'operation',
        message: "Which operation do you want to perform: ",
        choices: [
            "Subscribe an event",
            "Get events of module"
        ],
        filter: function(val) {
            if (val == "Subscribe an event")
                return 0;
            else
                return 1;
        }
    }
]

var questionsOp1 = [
    {
        type: 'input',
        name: 'module',
        message: "What is the module name whose events you want to fetch: ",
        default: "asset",
        filter: function(val) {
            return val.toLowerCase();
        }
    },
    {
        type: 'number',
        name: 'from',
        message: "Enter the from block: ",
        default: 1,
        validate: function(val) {
            if (val <= 0)
                return "Please enter the valid block no.";
            else
                return true;

        }
    },
    {
        type: 'number',
        name: 'to',
        message: "Enter the to block: ",
        default: async function() {
            // Initialise the provider to connect to the local node
            const wsProvider = new WsProvider('ws://127.0.0.1:9944');
            // Create the API and wait until ready
            const api = await ApiPromise.create({
                provider: wsProvider
            });
            return parseInt((await api.derive.chain.bestNumber()).toString());
        },
        validate: function(val) {
            if (val <= 0)
                return "Please enter the valid block no.";
            else 
                return true;

        }
    }
]

var getEventAllowed = [
    {
        type: 'confirm',
        name: "isAllowed",
        message: "Do you want to filter the events by event name? ",
        default: false,
    }
]

var getEventName = [
    {
        type: 'input',
        name: "eventName",
        message: "Enter the event name: ",
        default: "Transfer",
    }
]


var questionsOp2 = [
    {
        type: 'input',
        name: 'module',
        message: "What is the module name whose events you want to fetch: ",
        default: "asset",
        filter: function(val) {
            return val.toLowerCase();
        }
    }
]

async function moduleEvents (api) {
    let event_name;
    let answers = await inquirer.prompt(questionsOp1);
    let filterByEvent = await inquirer.prompt(getEventAllowed);
    if (filterByEvent.isAllowed)
        event_name = (await inquirer.prompt(getEventName)).eventName;
    let currentBlock = await api.derive.chain.bestNumber();
    if (answers.to >= answers.from && parseInt(currentBlock.toString()) >= answers.to) {
        let diff = parseInt(answers.to) - parseInt(answers.from);
        for (let k = 0; k <= diff; k++) {
            let blockNo = answers.from + k;
            let hash = await api.rpc.chain.getBlockHash(blockNo);
            let events = await api.query.system.events.at(hash.toString());
            for (let i = 0; i < Object.keys(events).length - 1; i++) {
                if (events[i].event.data["_section"]== answers.module) {
                    let typeList = events[i].event.data["_typeDef"];
                    if (events[i].event.data["_method"] != event_name && filterByEvent.isAllowed)
                        continue;
                    console.log(`EventName - ${events[i].event.data["_method"]} at block number ${blockNo}`);
                    for (let j = 0; j < typeList.length; j++) {
                        let value = events[i].event.data[j];
                        if (typeList[j].type == "Bytes")
                            value = Utils.hexToString(Utils.bytesToHex(events[i].event.data[j]));
                        console.log(`${typeList[j].type} : ${value}`);
                    }
                    console.log("***************************************"); 
                }
            }
        }
    } else {
        throw new Error("Invalid block numbers");
    }
    process.exit(0);
}

async function subscribeEvents(api) {
    // Subscribe to chain updates and log the current block  number on update.
    let answers = await inquirer.prompt(questionsOp2);
    const unsubscribe = await api.rpc.chain.subscribeNewHead(async (header) => {
        console.log(`Chain is at block: #${header.blockNumber}`);
        let hash = await api.rpc.chain.getBlockHash(header.blockNumber);
        let events = await api.query.system.events.at(hash.toString());
        for (let i = 0; i < Object.keys(events).length - 1; i++) {
            if (events[i].event.data["_section"]== answers.module) {
                let typeList = events[i].event.data["_typeDef"];
                console.log(`EventName - ${events[i].event.data["_method"]} at block number ${header.blockNumber}`);
                for (let j = 0; j < typeList.length; j++) {
                    let value = events[i].event.data[j];
                    if (typeList[j].type == "Bytes")
                        value = Utils.hexToString(Utils.bytesToHex(events[i].event.data[j]));
                    console.log(`${typeList[j].type} : ${value}`);
                }
                console.log("***************************************"); 
            }
        }
    });
}

async function main() {
    // Initialise the provider to connect to the local node
    const wsProvider = new WsProvider('ws://127.0.0.1:9944');
    // Create the API and wait until ready
    const api = await ApiPromise.create({
        types: customTypes,
        provider: wsProvider
    });
    // Retrieve the chain & node information information via rpc calls
    const [chain, nodeName, nodeVersion] = await Promise.all([
        api.rpc.system.chain(),
        api.rpc.system.name(),
        api.rpc.system.version()
    ]);
    console.log(`You are connected to chain ${chain} using ${nodeName} v${nodeVersion}`);

    inquirer.prompt(openingQuestions).then((answers) => {
        if (answers.operation == 1) {
            moduleEvents(api);
        } else {
            subscribeEvents(api);
        }
    }); 
}

main().catch((error) => {
    console.error(error);
    process.exit(-1);
});
