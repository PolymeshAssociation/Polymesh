const { ApiPromise, WsProvider, Keyring } = require("@polkadot/api");
const fs = require("fs");
const path = require("path");
const request = require("request");

const schema_url =
  process.env.SCHEMA || "https://raw.githubusercontent.com/PolymathNetwork/Polymesh/5.0.2/polymesh_schema.json";
const ws_providers = new WsProvider(process.env.RPC || "wss://staging-rpc.polymesh.live");
const mnemonic = process.env.MNEMONIC || "//Alice";

async function main() {
    request.get(schema_url, async function (error, response, body) {
        if (!error && response.statusCode == 200) {
            const { types, rpc } = JSON.parse(body);
            const api = await ApiPromise.create({
                provider: ws_providers,
                types,
                rpc,
            });

            const alice = new Keyring({ type: "sr25519" }).addFromMnemonic(mnemonic);
            const r = Math.random();
            console.log(`Random number used to generate secondary key: `, r);
            const secondary_key = new Keyring({ type: "sr25519" }).addFromUri(`//` + r + `testtest`);

            const totalPermissions = {
                asset: { These: [] },
                extrinsic: { These: [] },
                portfolio: { These: [] },
            };
            let authData = {
                JoinIdentity: totalPermissions,
            };
            let expiry = null;

            let target = {
                Account: secondary_key.publicKey,
            };
            let tx = api.tx.identity
                .addAuthorization(target, authData, expiry);    

            tx.signAndSend(alice, ({ events = [], status }) => {
                if (status.isFinalized) {
                    console.log('Successfully added authorization: ' + status.asFinalized.toHex());
                    events.forEach(({ phase, event: { data, method, section } }) => {
                        if (section == "identity" && method == "AuthorizationAdded") {
                            let eventData = JSON.parse(data.toString());     
                            let authId = eventData[3];                       
                            console.log('Authorization Id: ', authId);
                            let joinTx = api.tx.identity.joinIdentityAsKey(authId);
                            joinTx.signAndSend(secondary_key, ({ events = [], status }) => {
                                if (status.isFinalized) {
                                    console.log('Successfully accepted authorization: ' + status.asFinalized.toHex());
                                }
                            });
                        }
                    });
                } else {
                    console.log('Status of transaction: ' + status.type);
                }
            
            });
        };
    });
}


main();