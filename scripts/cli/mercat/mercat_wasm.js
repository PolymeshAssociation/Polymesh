let imports = {};
imports['__wbindgen_placeholder__'] = module.exports;
let wasm;
const { TextDecoder } = require(`util`);

let cachedTextDecoder = new TextDecoder('utf-8', { ignoreBOM: true, fatal: true });

cachedTextDecoder.decode();

let cachedUint8Memory0;
function getUint8Memory0() {
    if (cachedUint8Memory0.byteLength === 0) {
        cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);
    }
    return cachedUint8Memory0;
}

function getStringFromWasm0(ptr, len) {
    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));
}

const heap = new Array(32).fill(undefined);

heap.push(undefined, null, true, false);

let heap_next = heap.length;

function addHeapObject(obj) {
    if (heap_next === heap.length) heap.push(heap.length + 1);
    const idx = heap_next;
    heap_next = heap[idx];

    heap[idx] = obj;
    return idx;
}

let cachedInt32Memory0;
function getInt32Memory0() {
    if (cachedInt32Memory0.byteLength === 0) {
        cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
    }
    return cachedInt32Memory0;
}

function getArrayU8FromWasm0(ptr, len) {
    return getUint8Memory0().subarray(ptr / 1, ptr / 1 + len);
}

let WASM_VECTOR_LEN = 0;

function passArray8ToWasm0(arg, malloc) {
    const ptr = malloc(arg.length * 1);
    getUint8Memory0().set(arg, ptr / 1);
    WASM_VECTOR_LEN = arg.length;
    return ptr;
}

function _assertClass(instance, klass) {
    if (!(instance instanceof klass)) {
        throw new Error(`expected instance of ${klass.name}`);
    }
    return instance.ptr;
}

function getObject(idx) { return heap[idx]; }

function dropObject(idx) {
    if (idx < 36) return;
    heap[idx] = heap_next;
    heap_next = idx;
}

function takeObject(idx) {
    const ret = getObject(idx);
    dropObject(idx);
    return ret;
}
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
module.exports.create_account = function(seed) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(seed, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.create_account(retptr, ptr0, len0);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        var r2 = getInt32Memory0()[retptr / 4 + 2];
        if (r2) {
            throw takeObject(r1);
        }
        return CreateAccountOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

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
module.exports.create_mediator_account = function(seed) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(seed, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        wasm.create_mediator_account(retptr, ptr0, len0);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        var r2 = getInt32Memory0()[retptr / 4 + 2];
        if (r2) {
            throw takeObject(r1);
        }
        return Account.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

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
module.exports.mint_asset = function(seed, amount, issuer_account) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(seed, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(issuer_account, Account);
        wasm.mint_asset(retptr, ptr0, len0, amount, issuer_account.ptr);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        var r2 = getInt32Memory0()[retptr / 4 + 2];
        if (r2) {
            throw takeObject(r1);
        }
        return MintAssetOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

function isLikeNone(x) {
    return x === undefined || x === null;
}
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
module.exports.create_transaction = function(seed, amount, sender_account, encrypted_pending_balance, pending_balance, receiver_public_account, mediator_public_key) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(seed, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(sender_account, Account);
        const ptr1 = passArray8ToWasm0(encrypted_pending_balance, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(receiver_public_account, PubAccount);
        var ptr2 = isLikeNone(mediator_public_key) ? 0 : passArray8ToWasm0(mediator_public_key, wasm.__wbindgen_malloc);
        var len2 = WASM_VECTOR_LEN;
        wasm.create_transaction(retptr, ptr0, len0, amount, sender_account.ptr, ptr1, len1, pending_balance, receiver_public_account.ptr, ptr2, len2);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        var r2 = getInt32Memory0()[retptr / 4 + 2];
        if (r2) {
            throw takeObject(r1);
        }
        return CreateTransactionOutput.__wrap(r0);
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

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
module.exports.finalize_transaction = function(amount, init_tx, receiver_account) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(init_tx, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(receiver_account, Account);
        wasm.finalize_transaction(retptr, amount, ptr0, len0, receiver_account.ptr);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

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
module.exports.justify_transaction = function(seed, init_tx, mediator_account, sender_public_account, sender_encrypted_pending_balance, receiver_public_account, amount) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(seed, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        const ptr1 = passArray8ToWasm0(init_tx, wasm.__wbindgen_malloc);
        const len1 = WASM_VECTOR_LEN;
        _assertClass(mediator_account, Account);
        _assertClass(sender_public_account, PubAccount);
        const ptr2 = passArray8ToWasm0(sender_encrypted_pending_balance, wasm.__wbindgen_malloc);
        const len2 = WASM_VECTOR_LEN;
        _assertClass(receiver_public_account, PubAccount);
        wasm.justify_transaction(retptr, ptr0, len0, ptr1, len1, mediator_account.ptr, sender_public_account.ptr, ptr2, len2, receiver_public_account.ptr, !isLikeNone(amount), isLikeNone(amount) ? 0 : amount);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        if (r1) {
            throw takeObject(r0);
        }
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

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
module.exports.decrypt = function(encrypted_value, account) {
    try {
        const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
        const ptr0 = passArray8ToWasm0(encrypted_value, wasm.__wbindgen_malloc);
        const len0 = WASM_VECTOR_LEN;
        _assertClass(account, Account);
        wasm.decrypt(retptr, ptr0, len0, account.ptr);
        var r0 = getInt32Memory0()[retptr / 4 + 0];
        var r1 = getInt32Memory0()[retptr / 4 + 1];
        var r2 = getInt32Memory0()[retptr / 4 + 2];
        if (r2) {
            throw takeObject(r1);
        }
        return r0 >>> 0;
    } finally {
        wasm.__wbindgen_add_to_stack_pointer(16);
    }
};

/**
*/
module.exports.WasmError = Object.freeze({ AccountCreationError:0,"0":"AccountCreationError",AssetIssuanceError:1,"1":"AssetIssuanceError",TransactionCreationError:2,"2":"TransactionCreationError",TransactionFinalizationError:3,"3":"TransactionFinalizationError",TransactionJustificationError:4,"4":"TransactionJustificationError",DeserializationError:5,"5":"DeserializationError",HexDecodingError:6,"6":"HexDecodingError",PlainTickerIdsError:7,"7":"PlainTickerIdsError",DecryptionError:8,"8":"DecryptionError",SeedTooShortError:9,"9":"SeedTooShortError", });
/**
* A wrapper around mercat account.
*/
class Account {

    static __wrap(ptr) {
        const obj = Object.create(Account.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_account_free(ptr);
    }
    /**
    * @param {Uint8Array} secret
    * @param {PubAccount} pub_account
    */
    constructor(secret, pub_account) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(secret, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            _assertClass(pub_account, PubAccount);
            wasm.account_new(retptr, ptr0, len0, pub_account.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return Account.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * The secret account must be kept confidential and not shared with anyone else.
    */
    get secret_account() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.account_secret_account(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * The public account.
    */
    get public_account() {
        const ret = wasm.account_public_account(this.ptr);
        return PubAccount.__wrap(ret);
    }
}
module.exports.Account = Account;
/**
* Contains the secret and public account information of a party.
*/
class CreateAccountOutput {

    static __wrap(ptr) {
        const obj = Object.create(CreateAccountOutput.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_createaccountoutput_free(ptr);
    }
    /**
    * The secret account must be kept confidential and not shared with anyone else.
    */
    get account() {
        const ret = wasm.createaccountoutput_account(this.ptr);
        return Account.__wrap(ret);
    }
    /**
    * The Zero Knowledge proofs of the account creation.
    */
    get account_tx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.createaccountoutput_account_tx(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.CreateAccountOutput = CreateAccountOutput;
/**
* Contains the Zero Knowledge Proof of initializing a confidential transaction by the sender.
*/
class CreateTransactionOutput {

    static __wrap(ptr) {
        const obj = Object.create(CreateTransactionOutput.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_createtransactionoutput_free(ptr);
    }
    /**
    * The Zero Knowledge proofs of the initialized confidential transaction.
    */
    get init_tx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.createtransactionoutput_init_tx(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.CreateTransactionOutput = CreateTransactionOutput;
/**
* Contains the Zero Knowledge Proof of minting an asset by the issuer.
*/
class MintAssetOutput {

    static __wrap(ptr) {
        const obj = Object.create(MintAssetOutput.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_mintassetoutput_free(ptr);
    }
    /**
    * The Zero Knowledge proofs of the asset minting.
    */
    get asset_tx() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.createtransactionoutput_init_tx(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.MintAssetOutput = MintAssetOutput;
/**
* A wrapper around mercat public account.
*/
class PubAccount {

    static __wrap(ptr) {
        const obj = Object.create(PubAccount.prototype);
        obj.ptr = ptr;

        return obj;
    }

    __destroy_into_raw() {
        const ptr = this.ptr;
        this.ptr = 0;

        return ptr;
    }

    free() {
        const ptr = this.__destroy_into_raw();
        wasm.__wbg_pubaccount_free(ptr);
    }
    /**
    * @param {Uint8Array} public_key
    */
    constructor(public_key) {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            const ptr0 = passArray8ToWasm0(public_key, wasm.__wbindgen_malloc);
            const len0 = WASM_VECTOR_LEN;
            wasm.pubaccount_new(retptr, ptr0, len0);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var r2 = getInt32Memory0()[retptr / 4 + 2];
            if (r2) {
                throw takeObject(r1);
            }
            return PubAccount.__wrap(r0);
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
    /**
    * The public key.
    */
    get public_key() {
        try {
            const retptr = wasm.__wbindgen_add_to_stack_pointer(-16);
            wasm.pubaccount_public_key(retptr, this.ptr);
            var r0 = getInt32Memory0()[retptr / 4 + 0];
            var r1 = getInt32Memory0()[retptr / 4 + 1];
            var v0 = getArrayU8FromWasm0(r0, r1).slice();
            wasm.__wbindgen_free(r0, r1 * 1);
            return v0;
        } finally {
            wasm.__wbindgen_add_to_stack_pointer(16);
        }
    }
}
module.exports.PubAccount = PubAccount;

module.exports.__wbindgen_string_new = function(arg0, arg1) {
    const ret = getStringFromWasm0(arg0, arg1);
    return addHeapObject(ret);
};

module.exports.__wbindgen_throw = function(arg0, arg1) {
    throw new Error(getStringFromWasm0(arg0, arg1));
};

const path = require('path').join(__dirname, 'mercat_wasm_bg.wasm');
const bytes = require('fs').readFileSync(path);

const wasmModule = new WebAssembly.Module(bytes);
const wasmInstance = new WebAssembly.Instance(wasmModule, imports);
wasm = wasmInstance.exports;
module.exports.__wasm = wasm;

cachedInt32Memory0 = new Int32Array(wasm.memory.buffer);
cachedUint8Memory0 = new Uint8Array(wasm.memory.buffer);

