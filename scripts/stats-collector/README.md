# Polymesh Stats Collector
A small client-side Substrate stats collector. The `polkadot.js` logic lives in
`index.js` while orchestration config can be found in `environment.config.js`.

## Installation
```shell
$ yarn install #Project deps
$ npm install -g pm2 #test setup orchestration
```
## Usage
Running and viewing the stats collector logs
```shell
$ ./run.sh # Orchestrate the environment.
$ pm2 log stats-collector # view stats collector logs

#Viewing Substrate logs
$ pm2 log pmesh-primary-node # The node stats-collector connects to

# You can also connect to peer nodes with which the primary node interacts
$ pm2 log pmesh-peer-node-1
$ pm2 log pmesh-primary-node-2
```

## Running the collector process standalone
If you want to tweak a  setting real quick you can run the stats-collector
process separately from the Polymesh nodes:
```
./run.sh && pm2 stop stats-collector # Disable the automatically run stats-collector instance
node index.js -o 5 -c 2 # Now you can alter order of magnitude for tx count or claim count. See index.js for full option specs
```
