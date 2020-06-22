#!/usr/bin/env node

const chalk = require('chalk');
const clear = require('clear');
const figlet = require('figlet');
const path = require('path');
const program = require('commander');

clear();
console.log(
  chalk.blue(
    figlet.textSync('Polymesh-CLI', { horizontalLayout: 'full' })
  )
);

program
  .version('0.0.1')
  .description("Executes major functionality of Polymesh with our CLI.")
  .option('-r, --remote-node', 'Connecting to a remote node.')
  .parse(process.argv);