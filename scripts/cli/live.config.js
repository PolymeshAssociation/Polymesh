// PM2 config file
module.exports = {
  apps: [
    {
      name: "pmesh-primary-node",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-primary-node --pool-limit 100000 --ws-port 9944 --validator --chain ../../chain_spec.json --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      }
    },
    {
      name: "pmesh-peer-node-1",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-1 --ws-port 9945 --validator --chain ../../chain_spec.json",
      env: {
        RUST_BACKTRACE: "1"
      }
    },
    {
      name: "pmesh-peer-node-2",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-2 --ws-port 9946 --validator --chain ../../chain_spec.json",
      env: {
        RUST_BACKTRACE: "1"
      }
    },
    {
      name: "pmesh-peer-node-3",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-2 --ws-port 9947 --validator --chain ../../chain_spec.json",
      env: {
        RUST_BACKTRACE: "1"
      }
    },
    {
      name: "pmesh-peer-node-4",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-2 --ws-port 9948 --validator --chain ../../chain_spec.json",
      env: {
        RUST_BACKTRACE: "1"
      }
    }
  ]
};
