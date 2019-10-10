// PM2 config file
module.exports = {
  apps: [
    {
      name: "pmesh-primary-node",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-primary-node --pool-limit 100000 --ws-port 9944 --alice --validator --chain stats-testnet --force-authoring",
      env: {
        RUST_BACKTRACE: "1",
      }
    },
    {
      name: "pmesh-peer-node-1",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-1 --ws-port 9945 --bob --validator --chain stats-testnet --force-authoring",
      env: {
        RUST_BACKTRACE: "1"
      }
    },
    {
      name: "pmesh-peer-node-2",
      script:
        "../../target/release/polymesh",
      args: "-d /tmp/pmesh-peer-node-2 --ws-port 9946 --charlie --validator --chain stats-testnet --force-authoring",
      env: {
        RUST_BACKTRACE: "1"
      }
    },
    {
      name: "stats-collector",
      script: "./index.js",
      log_file: 'combined.log',
    }
  ]
};
