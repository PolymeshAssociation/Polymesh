// PM2 config file
let chain = "local";
let polymesh_bin = "../../target/release/polymesh";
let common = "--wasm-execution compiled " +
  //  "--db-cache=3000 " +
  "--state-pruning=archive " +
  "--rpc-methods=unsafe --rpc-external " +
  "--rpc-cors all --no-prometheus --no-telemetry --no-mdns " +
  "--validator --chain " + chain + " ";
// Use node-key parameter for the primary node.
let primary = " --node-key 0000000000000000000000000000000000000000000000000000000000000001 " +
  common;
// The peer nodes need to connect to the primary.
let peer = " --bootnodes /ip4/127.0.0.1/tcp/30334/p2p/12D3KooWEyoppNCUx8Yx66oV9fJnriXwCcXwDDUA2kj6vnc6iDEp " +
  common;
module.exports = {
  apps: [
    {
      name: "pmesh-primary-node",
      script: polymesh_bin,
      args:
        "-d /tmp/pmesh-primary-node --alice " + primary +
        " --port 30334 --rpc-port 9944",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-1",
      script: polymesh_bin,
      args:
        "-d /tmp/pmesh-peer-node-1 --bob " + peer +
        " --port 30335 --rpc-port 9945",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-2",
      script: polymesh_bin,
      args:
        "-d /tmp/pmesh-peer-node-2 --charlie " + peer +
        " --port 30336 --rpc-port 9946",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-3",
      script: polymesh_bin,
      args:
        "-d /tmp/pmesh-peer-node-3 --dave " + peer +
        " --port 30337 --rpc-port 9947",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-4",
      script: polymesh_bin,
      args:
        "-d /tmp/pmesh-peer-node-4 --eve " + peer +
        " --port 30338 --rpc-port 9948",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
  ],
};
