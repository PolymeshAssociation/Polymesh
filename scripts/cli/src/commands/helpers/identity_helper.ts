import { nonces, sendTransaction } from "../util/init";
import { topUpIdentityBalance } from "./balance_helper";
import { IKeyringPair } from "@polkadot/types/types";
import { ApiPromise } from "@polkadot/api";
import { SecondaryKey } from "../types";

export default async function createIdentities(
  api: ApiPromise,
  accounts: IKeyringPair[],
  cdd_provider: IKeyringPair,
  topup: boolean
) {
  return await createIdentitiesWithExpiry(
    api,
    accounts,
    cdd_provider,
    [],
    topup
  );
}

const createIdentitiesWithExpiry = async function (
  api: ApiPromise,
  accounts: IKeyringPair[],
  cdd_provider: IKeyringPair,
  expires: SecondaryKey[],
  topup: boolean
) {
  let dids = [];

  for (let i = 0; i < accounts.length; i++) {
    let nonceObj = { nonce: nonces.get(cdd_provider.address) };
    const transaction = api.tx.identity.cddRegisterDid(
      accounts[i].address,
      null,
      []
    );
    await sendTransaction(transaction, cdd_provider, nonceObj);

    nonces.set(cdd_provider.address, nonces.get(cdd_provider.address).addn(1));
  }

  for (let i = 0; i < accounts.length; i++) {
    const d: any = await api.query.identity.keyToIdentityIds(
      accounts[i].publicKey
    );
    dids.push(d.toHuman().Unique);
  }
  let did_balance = 1000 * 10 ** 6;
  for (let i = 0; i < dids.length; i++) {
    if (topup)
      await topUpIdentityBalance(api, cdd_provider, dids[i], did_balance);
  }

  return dids;
};
