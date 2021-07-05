import type { KeyringPair } from "@polkadot/keyring/types";
import { sendTx, ApiSingleton, keyToIdentityIds } from "../util/init";

export async function addToCommittee(signer: KeyringPair, member: KeyringPair) {
  const api = await ApiSingleton.getInstance();
  const did = await keyToIdentityIds(member.publicKey);
  const a = await api.query.committeeMembership.activeMembers();
  console.log(JSON.stringify(a[0].toJSON(), null, 2));
  const b = await keyToIdentityIds(member.publicKey);
  console.log(JSON.stringify(b.toJSON(), null, 2));
  const tx = api.tx.committeeMembership.setActiveMembersLimit(5);
  await sendTx(signer, tx);
  const tx2 = api.tx.committeeMembership.addMember(did);
  await sendTx(signer, tx2);
}
