import sql from "sqlite3";
import type { KeyringPair } from "@polkadot/keyring/types";

const sql3 = sql.verbose();
// Setting up a database for storing data.
const db = new sql3.Database("database.db");

export function createTable() {
  db.run(
    "CREATE TABLE IF NOT EXISTS accounts ( address TEXT PRIMARY KEY, nonce INT DEFAULT 0 )"
  );
}

export async function getNonce(signer: KeyringPair) {
  return new Promise<number>((resolve, reject) => {
    db.run(
      "INSERT INTO accounts(address) VALUES($address) ON CONFLICT(address) DO UPDATE SET nonce=nonce+1",
      { $address: signer.address },
      (runErr) => {
        if (runErr) {
          reject("Couldn't get next nonce");
        } else {
          db.get(
            "SELECT address, nonce FROM accounts WHERE address = $address",
            { $address: signer.address },
            (getErr, row) => {
              if (getErr) {
                reject(getErr);
              } else {
                resolve(row.nonce);
              }
            }
          );
        }
      }
    );
  });
}
