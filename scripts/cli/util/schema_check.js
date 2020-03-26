// Set options as a parameter, environment variable, or rc file.
require = require("esm")(module /*, options*/);
module.exports = require("./init.js");

let { reqImports } = require("./init.js");

async function main() {
  // Schema path
  const filePath = reqImports["path"].join(__dirname + "/../../../polymesh_schema.json");
  const customTypes = JSON.parse(reqImports["fs"].readFileSync(filePath, "utf8"));

  // Start node instance
  const ws_provider = new reqImports["WsProvider"]("ws://127.0.0.1:9944/");
  await reqImports["ApiPromise"].create({
    types: customTypes,
    provider: ws_provider
  });

process.exit();
}

main().catch(console.error);
