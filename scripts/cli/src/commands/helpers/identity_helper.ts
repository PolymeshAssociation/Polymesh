import {nonces, sendTransaction} from "../util/init"
import { topUpIdentityBalance } from "./balance_helper"
import { IKeyringPair } from '@polkadot/types/types'
import { ApiPromise } from "@polkadot/api"
import { SigningItem } from "../types"

export default async function createIdentities(api: ApiPromise, accounts: IKeyringPair[], alice: IKeyringPair) {
    return await createIdentitiesWithExpiry(api, accounts, alice, []);
};

const createIdentitiesWithExpiry = async function(api: ApiPromise, accounts: IKeyringPair[], alice: IKeyringPair, expiries: SigningItem[]) {
  let dids = [];

  for (let i = 0; i < accounts.length; i++) {
    let nonceObj = {nonce: nonces.get(alice.address)};
    const transaction = api.tx.identity.cddRegisterDid(accounts[i].address, null, []);
    await sendTransaction(transaction, alice, nonceObj);

    nonces.set(alice.address, nonces.get(alice.address).addn(1));
  }
  
  for (let i = 0; i < accounts.length; i++) {
    const d: any = await api.query.identity.keyToIdentityIds(accounts[i].publicKey);
    dids.push(d.raw.asUnique);
  }
  let did_balance = 1000 * 10**6;
  for (let i = 0; i < dids.length; i++) {

    await topUpIdentityBalance(api, alice, dids[i], did_balance);

  }
  
  return dids;
}