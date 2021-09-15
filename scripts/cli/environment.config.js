// PM2 config file
let chain = "ci-dev";
module.exports = {
  apps: [
    {
      name: "pmesh-primary-node",
      script: "../../target/release/polymesh",
      args:
        "-d /tmp/pmesh-primary-node --pruning=archive --rpc-methods=unsafe --rpc-external --ws-external --pool-limit 100000 --ws-port 9944 --rpc-cors all --alice --validator --chain " +
        chain +
        " --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-1",
      script: "../../target/release/polymesh",
      args:
        "-d /tmp/pmesh-peer-node-1 --ws-port 9945 --bob --validator --chain " +
        chain +
        " --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-2",
      script: "../../target/release/polymesh",
      args:
        "-d /tmp/pmesh-peer-node-2 --ws-port 9946 --charlie --validator --chain " +
        chain +
        " --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-3",
      script: "../../target/release/polymesh",
      args:
        "-d /tmp/pmesh-peer-node-3 --ws-port 9947 --dave --validator --chain " +
        chain +
        " --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
    {
      name: "pmesh-peer-node-4",
      script: "../../target/release/polymesh",
      args:
        "-d /tmp/pmesh-peer-node-4 --ws-port 9948 --eve --validator --chain " +
        chain +
        " --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      },
    },
  ],
};
