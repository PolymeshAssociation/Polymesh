const inquirer = require('inquirer')
const { ApiPromise, WsProvider } = require("@polkadot/api");
const { Keyring } = require("@polkadot/keyring");
const { stringToU8a, u8aToHex } = require("@polkadot/util");
const BN = require("bn.js");
const cli = require("command-line-args");
const cliProg = require("cli-progress");
const childProc = require("child_process");

const fs = require("fs");
const path = require("path");
let keyring;
let api;

initialise();

async function initialise() {

    console.log(
        `Welcome to Polymesh CLI`
    );

    const filePath = path.join(
        __dirname + "/../../../polymesh/polymesh_schema.json"
    );
    const customTypes = JSON.parse(fs.readFileSync(filePath, "utf8"));

    // const ws_provider = new WsProvider("ws://78.47.58.121:9944/");
    const ws_provider = new WsProvider("ws://127.0.0.1:9944/");
    api = await ApiPromise.create({
        types: customTypes,
        provider: ws_provider
    });
    keyring = new Keyring({ type: "sr25519" });
    userKey();
}

function userKey() {

    var questions = [{
        type: 'input',
        name: 'keyUri',
        message: "Enter keyUri",
    }]

    inquirer.prompt(questions).then(async answers => {
        let user = keyring.addFromUri(answers['keyUri'], { name: "User" });
        let user_balance = await api.query.balances.freeBalance(user.address);
        console.log("User: " + JSON.stringify(user));
        console.log("User Balance: " + JSON.stringify(user_balance));

    })

}

function identityMenu() {

    var questions = [{
        type: 'input',
        name: 'keyUri',
        message: "Enter keyUri",
    }]

    inquirer.prompt(questions).then(async answers => {
        let user = keyring.addFromUri(answers['keyUri'], { name: "User" });
        let user_balance = await api.query.balances.freeBalance(user.address);
        console.log("User: " + JSON.stringify(user));
        console.log("User Balance: " + JSON.stringify(user_balance));
    })
    
}