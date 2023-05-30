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
* * `CreateMediatorAccountOutput`: Contains the public and secret mediator account.
*
* # Errors
* @param {Uint8Array} seed
* @returns {CreateMediatorAccountOutput}
*/
export function create_mediator_account(seed: Uint8Array): CreateMediatorAccountOutput;
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
export function create_transaction(seed: Uint8Array, amount: number, sender_account: Account, encrypted_pending_balance: Uint8Array, pending_balance: number, receiver_public_account: PubAccount, mediator_public_key: Uint8Array | undefined): CreateTransactionOutput;
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
* * `mediator_account`: The secret portion of the mediator's account. Can be obtained from
*                       `CreateMediatorAccountOutput.secret_account`.
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
export function justify_transaction(seed: Uint8Array, init_tx: Uint8Array, mediator_account: Account, sender_public_account: PubAccount, sender_encrypted_pending_balance: Uint8Array, receiver_public_account: PubAccount, amount: number | undefined): void;
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
export function __wbindgen_string_new(arg0: any, arg1: any): number;
export function __wbindgen_throw(arg0: any, arg1: any): void;
/**
*/
export const WasmError: Readonly<{
    AccountCreationError: 0;
    "0": "AccountCreationError";
    AssetIssuanceError: 1;
    "1": "AssetIssuanceError";
    TransactionCreationError: 2;
    "2": "TransactionCreationError";
    TransactionFinalizationError: 3;
    "3": "TransactionFinalizationError";
    TransactionJustificationError: 4;
    "4": "TransactionJustificationError";
    DeserializationError: 5;
    "5": "DeserializationError";
    HexDecodingError: 6;
    "6": "HexDecodingError";
    PlainTickerIdsError: 7;
    "7": "PlainTickerIdsError";
    DecryptionError: 8;
    "8": "DecryptionError";
    SeedTooShortError: 9;
    "9": "SeedTooShortError";
}>;
/**
* A wrapper around mercat account.
*/
export class Account {
    static __wrap(ptr: any): any;
    /**
    * @param {Uint8Array} secret
    * @param {PubAccount} public
    */
    constructor(secret: Uint8Array, public: PubAccount);
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The secret account must be kept confidential and not shared with anyone else.
    */
    get secret_account(): any;
    /**
    * The public account.
    */
    get public_account(): any;
}
/**
* Contains the secret and public account information of a party.
*/
export class CreateAccountOutput {
    static __wrap(ptr: any): any;
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The secret account must be kept confidential and not shared with anyone else.
    */
    get account(): any;
    /**
    * The Zero Knowledge proofs of the account creation.
    */
    get account_tx(): any;
}
/**
* Contains the secret and public account information of a mediator.
*/
export class CreateMediatorAccountOutput {
    static __wrap(ptr: any): any;
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The secret account must be kept confidential and not shared with anyone else.
    */
    get account(): any;
}
/**
* Contains the Zero Knowledge Proof of initializing a confidential transaction by the sender.
*/
export class CreateTransactionOutput {
    static __wrap(ptr: any): any;
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The Zero Knowledge proofs of the initialized confidential transaction.
    */
    get init_tx(): any;
}
/**
* Contains the Zero Knowledge Proof of minting an asset by the issuer.
*/
export class MintAssetOutput {
    static __wrap(ptr: any): any;
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The Zero Knowledge proofs of the asset minting.
    */
    get asset_tx(): any;
}
/**
* A wrapper around mercat public account.
*/
export class PubAccount {
    static __wrap(ptr: any): any;
    /**
    * @param {Uint8Array} public_key
    */
    constructor(public_key: Uint8Array);
    __destroy_into_raw(): number | undefined;
    ptr: number | undefined;
    free(): void;
    /**
    * The public key.
    */
    get public_key(): any;
}
