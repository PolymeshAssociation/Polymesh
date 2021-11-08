import sql from "sqlite3";
import type { KeyringPair } from "@polkadot/keyring/types";

const sql3 = sql.verbose();
// Setting up a database for storing data.
const db = new sql3.Database("database.db");

export function createTable() {
  db.run(
    "CREATE TABLE IF NOT EXISTS next_nonce ( account TEXT PRIMARY KEY, nonce INT DEFAULT 0 )"
  );
}

export function insertNonce(accountAddr: string) {
  db.run("INSERT OR REPLACE INTO next_nonce(account) VALUES($account)", {
    $account: accountAddr,
  });
}

export function incrementNonce(accountAddr: string) {
  db.run(
    "INSERT INTO next_nonce(account) VALUES($account) ON CONFLICT(account) DO UPDATE SET nonce=nonce+1",
    { $account: accountAddr }
  );
}

export async function getNonce(signer: KeyringPair) {
  return new Promise<number>((resolve, reject) => {
    db.get(
      "SELECT account, nonce FROM next_nonce WHERE account = $account",
      { $account: signer.address },
      (err, row) => {
        if (err) reject(err);

        console.log(`accound: ${row.account} nonce: ${row.nonce}`);
        resolve(row.nonce);
      }
    );
  });
}
