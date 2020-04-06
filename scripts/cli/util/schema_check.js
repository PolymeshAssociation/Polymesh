// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

let { reqImports } = require("./init.js");

async function main() {
  try {
  await reqImports.createApi();
  }
  catch(err) {
    console.log(err);
    console.log("ErrorOccurred");
    process.exitCode = 1;
  }
  process.exit();
}

main().catch(console.error);
