import { nonces, sendTransaction } from "../util/init";
import { IKeyringPair } from "@polkadot/types/types";
import { ApiPromise } from "@polkadot/api";
import { H256 } from "@polkadot/types/interfaces/runtime";

// Top up identity balance
export async function topUpIdentityBalance(
  api: ApiPromise,
  signer: IKeyringPair,
  did: H256,
  did_balance: number
) {
  let nonceObj = { nonce: nonces.get(signer.address) };
  const transaction = api.tx.balances.topUpIdentityBalance(did, did_balance);
  await sendTransaction(transaction, signer, nonceObj);

  nonces.set(signer.address, nonces.get(signer.address).addn(1));
}
