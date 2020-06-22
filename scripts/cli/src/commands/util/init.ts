import { AccountId, Address } from "@polkadot/types/interfaces/runtime"
import { Type } from "@polkadot/types"
import { SubmittableExtrinsic } from '@polkadot/api/types';
import { IKeyringPair , ISubmittableResult } from '@polkadot/types/types';


export type Account = {
    address: Address,
    isLocked: Boolean,
    meta: {name: String},
    publicKey: AccountId,
    type: Type
}

export let nonces = new Map();

export function sendTransaction(transaction: SubmittableExtrinsic<"promise">, signer: IKeyringPair, nonceObj:any) {
    return new Promise((resolve, reject) => {
    
        let receipt: ISubmittableResult;
        const gettingUnsub = transaction.signAndSend(signer, nonceObj, receipt => {
  
        const { status } = receipt;
  
        if (receipt.isCompleted) {
  
          /*
           * isCompleted === isFinalized || isError, which means
           * no further updates, so we unsubscribe
           */
          gettingUnsub.then(unsub => {
  
            unsub();
  
          });
  
          if (receipt.isInBlock) {
  
            // tx included in a block and finalized
            const failed = receipt.findRecord('system', 'ExtrinsicFailed');
  
            if (failed) {
  
              // get revert message from event
              let message = "";
              const dispatchError: any = failed.event.data[0];
  
              if (dispatchError.isModule) {
  
                // known error
                const mod = dispatchError.asModule;
                const { section, name, documentation } = mod.registry.findMetaError(
                  new Uint8Array([mod.index.toNumber(), mod.error.toNumber()])
                );
  
                message = `${section}.${name}: ${documentation.join(' ')}`;
              } else if (dispatchError.isBadOrigin) {
                message = 'Bad origin';
              } else if (dispatchError.isCannotLookup) {
                message = 'Could not lookup information required to validate the transaction';
              } else {
                message = 'Unknown error';
              }
  
              reject(new Error(message));
            } else {
  
              resolve(receipt);
            }
          } else if (receipt.isError) {
  
            reject(new Error('Transaction Aborted'));
  
          }
        }
      });
    });
  }