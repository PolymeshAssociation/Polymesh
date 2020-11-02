const fs = require("fs");
const path = require("path");
const mnemonic = "bottom drive obey lake curtain smoke basket hold race lonely fit walk";
// Schema path
const filePath = path.join(
    __dirname + "/../../polymesh_schema.json"
  );
const { types } = JSON.parse(fs.readFileSync(filePath, "utf8"));

module.exports = {
  networks: {
    test: {
      mnemonic,
      ws: "ws://127.0.0.1:9944",
    },
  },
  polkadotjs: {
    types: types
  }
}