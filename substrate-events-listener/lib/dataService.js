// The data service module provides an abstraction between the listener and the data sink.
// To add a new data sink,
//      1. Create the js client in the adaptors dir (similar to mongo client)
//      2. Add client related config (server, port, etc.) in .env
//      3. Import the new js client here
//      4. Use init() for initializing the client - connection, priming, etc.
//      5. Use insert() for (of course) inserting substrate events data

const mongo = require('../adaptors/mongo');
// const ipfs = require('../adaptors/ipfs'); // example

// intialize the storage adaptors
// more than one can be used asynchronously
var init = async function() {
    await mongo.connect();
    // await ipfs.connect(); // example
}

// insert in one or more storage sinks
// more than one can be used asynchronously
var insert = async function(data) {
    await mongo.insert(data);
    // await ipfs.insert(data); // example
}

module.exports = { init, insert }