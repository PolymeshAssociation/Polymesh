#!/usr/bin/env node

import chalk from "chalk";
import clear from "clear";
import figlet from "figlet";
import path from "path";
import program from "commander";
import { setAPI, getAPI, setCddProvider } from "./commands/util/init";
import createAccountIdentity from "./commands/create_account_identity";
import createKeyIdentity from "./commands/create_key_identity";

console.log(
  chalk.blue(figlet.textSync("Polymesh-CLI", { horizontalLayout: "full" }))
);

program
  .version("0.0.1")
  .description("Executes major functionality of Polymesh with our CLI.")
  .option("-r, --remote-node <network>", "Connecting to a remote node.")
  .option(
    "-c, --cdd-provider <provider>",
    "polymesh cdd provider account name."
  );

program
  .command("createAccountIdentity")
  .alias("cai")
  .requiredOption("-e, --entityName <entityName>", "polymesh account name")
  .option(
    "-t, --topup",
    "A boolean flag to decide if the identity should be topped up."
  )
  .description(
    "Wizard-like script that will guide technical users in the creation of an account identity"
  )
  .action(async function (cmd) {
    await setAPI(program.remoteNode);
    let api = getAPI();
    await setCddProvider(api, program.cddProvider);
    await createAccountIdentity(cmd.entityName, cmd.topup);
    process.exit(0);
  });

program
  .command("createKeyIdentity")
  .alias("cki")
  .requiredOption(
    "-k, --keyAmount <keyAmount>",
    "The amount of account keys to create"
  )
  .requiredOption("-K, --keyPrepend <keyPrepend>", "The prepend of the keys")
  .option(
    "-t, --topup",
    "A boolean flag to decide if the identity should be topped up."
  )
  .description(
    "Wizard-like script that will guide technical users in the creation of a keys identity"
  )
  .action(async function (cmd) {
    await setAPI(program.remoteNode);
    let api = getAPI();
    await setCddProvider(api, program.cddProvider);
    await createKeyIdentity(cmd.keyAmount, cmd.keyPrepend, cmd.topup);
    process.exit(0);
  });

program.parse(process.argv);

if (program.commands.length == 0) {
  console.error("No command given!");
  process.exit(1);
}
