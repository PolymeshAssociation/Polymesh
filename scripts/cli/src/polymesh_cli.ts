#!/usr/bin/env node

import chalk from 'chalk';
import clear from 'clear';
import figlet from 'figlet';
import path from 'path';
import program from 'commander';
import { setAPI } from './commands/util/init';
import createAccountIdentity from "./commands/create_account_identity";
import createKeyIdentity from "./commands/create_key_identity";


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
  .command('createAccountIdentity')
  .alias('cai')
  .requiredOption('-e, --entityName <entityName>', 'polymesh account name')
  .description('Wizard-like script that will guide technical users in the creation of an account identity')
  .action(async function (cmd) {
    
    await setAPI(program.remoteNode);
    await createAccountIdentity(cmd.entityName);
    process.exit(0);
  });

program
  .command('createKeyIdentity')
  .alias('cki')
  .requiredOption('-k, --keyAmount <keyAmount>', 'The amount of account keys to create')
  .requiredOption('-K, --keyPrepend <keyPrepend>', 'The prepend of the keys')
  .description('Wizard-like script that will guide technical users in the creation of a keys identity')
  .action(async function (cmd) {
    
    await setAPI(program.remoteNode);
    await createKeyIdentity(cmd.keyAmount, cmd.keyPrepend);
    process.exit(0);
  });

  //await program.parseAsync(process.argv);
  program.parse(process.argv);

  if (program.commands.length == 0) {
    console.error('No command given!');
    process.exit(1);
  }
