/* tslint:disable */
/* eslint-disable */
/**
* Creates a mercat account. It is the responsibility of the caller
* to properly store and safeguard the secret values returned by this function.
*
* # Outputs
* * `CreateAccountOutput`: Contains both the public and secret account information.
*
* # Errors
* * `AccountCreationError`: If mercat library throws an error while creating the account.
* @param {Uint8Array} seed
* @returns {CreateAccountOutput}
*/
export function create_account(seed: Uint8Array): CreateAccountOutput;
/**
* Creates a mercat mediator account. It is the responsibility of the caller
* to properly store and safeguard the secret values returned by this function.
*
* # Arguments
*
* # Outputs
* * `Account`: Contains the public and secret mediator account.
*
* # Errors
* @param {Uint8Array} seed
* @returns {Account}
*/
export function create_mediator_account(seed: Uint8Array): Account;
/**
* Creates a Zero Knowledge Proof of minting a confidential asset.
*
* # Arguments
* * `amount`: An integer with a max value of `2^32` representing the mint amount.
* * `issuer_account`: The mercat account. Can be obtained from `CreateAccountOutput.account`.
*
* # Outputs
* * `MintAssetOutput`: The ZKP of minting the asset.
*
* # Errors
* * `DeserializationError`: If the `issuer_account` cannot be deserialized to a mercat account.
* @param {Uint8Array} seed
* @param {number} amount
* @param {Account} issuer_account
* @returns {MintAssetOutput}
*/
export function mint_asset(seed: Uint8Array, amount: number, issuer_account: Account): MintAssetOutput;
/**
* Creates the ZKP for the initial phase of creating a confidential transaction. This function
* is called by the sender and depends on secret information from the sender and public
* information of the receiver and the mediator.
*
* # Arguments
* * `amount`: An integer with a max value of `2^32` representing the mint amount.
* * `sender_account`: The mercat account. Can be obtained from `CreateAccountOutput.account`.
* * `encrypted_pending_balance`: Sender's encrypted pending balance. Can be obtained from the
*                                chain.
* * `receiver_public_account`: Receiver's public account. Can be obtained from the chain.
* * `mediator_public_key`: Mediator's public key. Can be obtained from the chain.
*
* # Outputs
* * `CreateAccountOutput`: The ZKP of the initialized transaction.
*
* # Errors
* * `DeserializationError`: If either of the inputs cannot be deserialized to a mercat account.
* * `TransactionCreationError`: If the mercat library throws an error when creating the proof.
* @param {Uint8Array} seed
* @param {number} amount
* @param {Account} sender_account
* @param {Uint8Array} encrypted_pending_balance
* @param {number} pending_balance
* @param {PubAccount} receiver_public_account
* @param {Uint8Array | undefined} mediator_public_key
* @returns {CreateTransactionOutput}
*/
export function create_transaction(seed: Uint8Array, amount: number, sender_account: Account, encrypted_pending_balance: Uint8Array, pending_balance: number, receiver_public_account: PubAccount, mediator_public_key?: Uint8Array): CreateTransactionOutput;
/**
* Creates the ZKP for the finalized phase of creating a confidential transaction. This function
* is called by the receiver and depends on secret information from the receiver and public
* information of the sender.
*
* # Arguments
* * `amount`: An integer with a max value of `2^32` representing the mint amount.
* * `init_tx`: The initialized transaction proof. Can be obtained from the chain.
* * `receiver_account`: The mercat account. Can be obtained from `CreateAccountOutput.account`.
*
* # Errors
* * `DeserializationError`: If either of the inputs cannot be deserialized to a mercat account.
* * `TransactionFinalizationError`: If the mercat library throws an error when creating the proof.
* @param {number} amount
* @param {Uint8Array} init_tx
* @param {Account} receiver_account
*/
export function finalize_transaction(amount: number, init_tx: Uint8Array, receiver_account: Account): void;
/**
* Creates the ZKP for the justification phase of creating a confidential transaction.
* This function is called by the mediator and depends on secret information from the
* mediator and public information of the sender and the receiver.
*
* # Arguments
* * `finalized_tx`: The finalized transaction proof. Can be obtained from the chain.
* * `mediator_account`: The mediator's account.
* * `sender_public_account`: Sender's public account. Can be obtained from the chain.
* * `sender_encrypted_pending_balance`: Sender's encrypted pending balance.
*                                       Can be obtained from the chain.
* * `receiver_public_account`: Receiver's public account. Can be obtained from the chain.
*
* # Errors
* * `DeserializationError`: If either of the inputs cannot be deserialized to a mercat account.
* * `TransactionJustificationError`: If the mercat library throws an error when creating the proof.
* @param {Uint8Array} seed
* @param {Uint8Array} init_tx
* @param {Account} mediator_account
* @param {PubAccount} sender_public_account
* @param {Uint8Array} sender_encrypted_pending_balance
* @param {PubAccount} receiver_public_account
* @param {number | undefined} amount
*/
export function justify_transaction(seed: Uint8Array, init_tx: Uint8Array, mediator_account: Account, sender_public_account: PubAccount, sender_encrypted_pending_balance: Uint8Array, receiver_public_account: PubAccount, amount?: number): void;
/**
* Decrypts an `encrypted_value` given the secret account information.
*
* # Arguments
* * `encrypted_value`: The encrypted value.
* * `account`: The mercat account. Can be obtained from `CreateAccountOutput.account`.
*
* # Outputs
* * `Balance`: The decrypted value.
*
* # Errors
* * `DeserializationError`: If either of the inputs cannot be deserialized to a mercat account.
* * `DecryptionError`: If the mercat library throws an error while decrypting the value.
* @param {Uint8Array} encrypted_value
* @param {Account} account
* @returns {number}
*/
export function decrypt(encrypted_value: Uint8Array, account: Account): number;
/**
*/
export enum WasmError {
  AccountCreationError,
  AssetIssuanceError,
  TransactionCreationError,
  TransactionFinalizationError,
  TransactionJustificationError,
  DeserializationError,
  HexDecodingError,
  PlainTickerIdsError,
  DecryptionError,
  SeedTooShortError,
}
/**
* A wrapper around mercat account.
*/
export class Account {
  free(): void;
/**
* @param {Uint8Array} secret
* @param {PubAccount} pub_account
*/
  constructor(secret: Uint8Array, pub_account: PubAccount);
/**
* The public account.
*/
  readonly public_account: PubAccount;
/**
* The secret account must be kept confidential and not shared with anyone else.
*/
  readonly secret_account: Uint8Array;
}
/**
* Contains the secret and public account information of a party.
*/
export class CreateAccountOutput {
  free(): void;
/**
* The secret account must be kept confidential and not shared with anyone else.
*/
  readonly account: Account;
/**
* The Zero Knowledge proofs of the account creation.
*/
  readonly account_tx: Uint8Array;
}
/**
* Contains the Zero Knowledge Proof of initializing a confidential transaction by the sender.
*/
export class CreateTransactionOutput {
  free(): void;
/**
* The Zero Knowledge proofs of the initialized confidential transaction.
*/
  readonly init_tx: Uint8Array;
}
/**
* Contains the Zero Knowledge Proof of minting an asset by the issuer.
*/
export class MintAssetOutput {
  free(): void;
/**
* The Zero Knowledge proofs of the asset minting.
*/
  readonly asset_tx: Uint8Array;
}
/**
* A wrapper around mercat public account.
*/
export class PubAccount {
  free(): void;
/**
* @param {Uint8Array} public_key
*/
  constructor(public_key: Uint8Array);
/**
* The public key.
*/
  readonly public_key: Uint8Array;
}
