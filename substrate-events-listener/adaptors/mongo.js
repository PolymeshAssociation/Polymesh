// a simple mongodb js client

var mdb = require('mongodb');
const client = mdb.MongoClient;

require('dotenv').config();

const state = {
    conn: null
}

// build the connection string (server path)
const connectionUrl = "mongodb://" + process.env.MONGO_HOST + 
    ":" + 
    process.env.MONGO_PORT + 
    "/" + 
    process.env.MONGO_DB;

const collectionName = process.env.MONGO_COLLECTION;

// connect to mongo server and store the connection object
var connect = async function(url = connectionUrl) {
    console.log(connectionUrl);
    return new Promise((resolve, reject) => {
        client.connect(url, function (err, resp) {
            if (!err) {
                console.log("Connected to MongoDB server", url)
                state.conn = resp
                resolve(true)
            } else {
                reject(err)
            }
        })
    })
}

// close the connection
var close = function() {
    state.conn.close()
}

// insert an object
var insert = async function(data) {
    if(!state.conn) {
        await connect();
    }

    const db = state.conn.db(process.env.MONGO_DB);
    await db.collection(collectionName).insertOne(data);
}

module.exports = { connect, close, insert }

