// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

let { reqImports } = require("./init.js");

async function main() {
  await reqImports.createApi();
  process.exit();
}

main().catch(console.error);
