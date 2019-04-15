# Substrate Events Listener & Storage Adapter

A simple node.js based web-socket listener which subscribes to Parity Substrate runtime events and stores them in a data store. At the moment, MongoDB is supported but new stores can be added easily.

Substrate node and data sink related config lives in the `.env` file.

## Why?

Ideally, we should avoid running iterators and loops on the on-chain data stored in a blockchain runtime or smart contract. The chain should store the minimum amount of data which is needed for conflict resolution. Everything else can and should be stored off-chain with it's hash on the chain. In cases where on-chain data is stored in collections (maps) and needs to be shown on a UI, an event based approach can be used to build an off-chain storage which can be queried directly from the UI.

This app listens to all Substrate runtime events using the web-socket connection, parses each event based on (extensible) custom rules and then stores the processed event data in one or more data stores. This off-chain storage can then be used for data analysis, querying, showing it on the UI, etc.

## Usage

1. Clone this repository.
2. Make sure your Substrate node and MongoDB is running. Update the `.env` file with the server and ports, if needed.
3. Set event section filters (comma separated list of module names, no spaces) in the `.env` file to process filtered events. 

For example, to select all events,

```
SUBSTRATE_EVENT_SECTIONS=all
```

and to select events only from `balances`,`assets`,`token` modules,

```
SUBSTRATE_EVENT_SECTIONS=balances,assets,token
```

4. Run the following docker command to build the image,

```
docker build -t substrate-listener .
```

5. Run the following docker command to start the listener in a container,

```
docker run -d substrate-listener
```

## Add new data stores

To use a new data store with the listener,

1. Create the storage client in the `adaptors` dir (similar to existing mongo client)
2. Add the connection config in `.env` and use it in the js client
3. Import the new js client in `dataService.js`
4. Use init() for initializing the client - connection, priming, etc.
5. Use insert() for (of course) inserting substrate events data

## Note

This app is **not** for production usage. It is mainly built to suggest an event based pattern for priming an off-chain storage. Please feel free to extend and use as needed.