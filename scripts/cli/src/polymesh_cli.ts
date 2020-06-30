#!/usr/bin/env node

import chalk from 'chalk';
import clear from 'clear';
import figlet from 'figlet';
import path from 'path';
import program from 'commander';
import { setAPI } from './commands/util/init';
import createIdentity from "./commands/create_identities";

clear();
console.log(
  chalk.blue(
    figlet.textSync('Polymesh-CLI', { horizontalLayout: 'full' })
  )
);

program
  .version('0.0.1')
  .description("Executes major functionality of Polymesh with our CLI.")
  .option('-r, --remote-node <network>', 'Connecting to a remote node.');
  

program
  .command('createIdentity')
  .alias('ci')
  .option('-e, --entityName <entityName>', 'polymesh account name')
  .option('-k, --keyNumber <keyNumber>', 'The amount of account keys to create')
  .option('-K, --keyPrepend <keyPrepend>', 'The prepend of the keys')
  .description('Wizard-like script that will guide technical users in the creation of an identity')
  .action(async function (cmd) {
    await setAPI(program.remoteNode);
    console.log(`api url ${program.remoteNode}`);
    await createIdentity(cmd.entityName, cmd.keyNumber, cmd.keyPrepend);
    console.log(`create identity finished`);
  });
