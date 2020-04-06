./target/release/polymesh purge-chain --dev -y
./target/release/polymesh --dev &>/dev/null &
cd scripts/cli
npm test
