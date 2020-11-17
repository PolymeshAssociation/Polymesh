use crate::*;
use pallet_balances::{self as balances, Call as BalancesCall};
use pallet_identity::benchmarking::User;

use frame_benchmarking::benchmarks;
use sp_runtime::{traits::StaticLookup, MultiSignature};

#[cfg(not(feature = "std"))]
use hex_literal::hex;
#[cfg(feature = "std")]
use sp_core::sr25519::Signature;

const MAX_CALLS: u32 = 30;

fn make_calls<T: Trait>(c: u32) -> Vec<<T as Trait>::Call> {
    let call: <T as Trait>::Call = frame_system::Call::<T>::remark(vec![]).into();
    vec![call; c as usize]
}

fn make_transfer_calls<T: Trait>(
    c: u32,
    to: T::AccountId,
    amount: u128,
) -> Vec<<T as Trait>::Call> {
    let idx = <T as frame_system::Trait>::Lookup::unlookup(to);
    let call: <T as Trait>::Call = BalancesCall::transfer(idx, amount.into()).into();
    vec![call; c as usize]
}

fn verify_free_balance<T: Trait>(account: &T::AccountId, expected_balance: u128) {
    let acc_balance = balances::Module::<T>::free_balance(account);
    assert_eq!(acc_balance, expected_balance.into())
}

struct RelayTxSetup<T: Trait> {
    pub caller: T::AccountId,
    pub target: T::AccountId,
    pub signature: T::OffChainSignature,
    pub call: UniqueCall<<T as Trait>::Call>,
}

/// Relay transaction setup for native code.
/// It prepares a relayed transfer from `bob` to `alice` which is going to be called by `alice`.
/// In order to do that, `bob` has to sign the call. We can sign only in native execution, because
/// WASM does not enable `full_crypto` feature.
#[cfg(feature = "std")]
fn relay_tx_setup<T: Trait>() -> RelayTxSetup<T> {
    let alice = User::<T>::new("Caller", 1);
    let bob = User::<T>::new("Target", 1);

    // Alice sends 1 POLY on behalf of bob from bob's account.
    let call = make_transfer_calls::<T>(1, alice.account.clone(), 1)
        .pop()
        .unwrap();
    let nonce: AuthorizationNonce = Module::<T>::nonce(bob.account.clone());
    let call = UniqueCall::new(nonce, call);

    // Bob signs the relay call.
    // NB: Decode as T::OffChainSignature because there is not type constraints in
    // `T::OffChainSignature` to limit it.
    let raw_signature: [u8; 64] = bob.sign(&call.encode()).to_bytes();
    let encoded = MultiSignature::from(Signature::from_raw(raw_signature)).encode();
    // Native execution can generate a hard-coded signature using the following code:
    // ```ignore
    //  let hex_encoded = hex::encode(&encoded);
    //  frame_support::debug::info!("Bob nonce:{} encoded:{:?}", nonce, &hex_encoded);
    //  ```

    let signature = T::OffChainSignature::decode(&mut &encoded[..])
        .expect("OffChainSignature cannot be decoded from a MultiSignature");

    RelayTxSetup {
        caller: alice.account,
        target: bob.account,
        signature,
        call,
    }
}

/// Relay transaction setup for WASM code.
/// Public keys of `alice` and `bob` and the signature of the relayed call are generated in the native part, so WASM does not need to use the crypto stuff.
///
#[cfg(not(feature = "std"))]
fn relay_tx_setup<T: Trait>() -> RelayTxSetup<T> {
    // Keys generated
    let alice_pk = hex!("6a4f597d1a0004ee6fd08622baf93fc350c048aeb8a6bf253208b1a536539333");
    let bob_pk = hex!("8a2f30f00294ca72f2e9572263c8cf96695a1d9fffff3f8b0d49171a917d9f31");

    // Create account from generated keys.
    let alice = User::<T>::new_from_public(alice_pk);
    let bob = User::<T>::new_from_public(bob_pk);

    // Create the relayed call and its signature.
    let call = make_transfer_calls::<T>(1, alice.account.clone(), 1)
        .pop()
        .unwrap();
    let nonce: AuthorizationNonce = Module::<T>::nonce(bob.account.clone());
    let call = UniqueCall::new(nonce, call);

    //
    let data = hex!("01aa3fb75dddaa9d1c058097aa10814a46a411192124a1970e21fc9547a075045090a318672bd44baa8f2069c4484ffc1b0e133011a80a1aaf2a484970bcffd987");
    let signature = T::OffChainSignature::decode(&mut &data[..])
        .expect("OffChainSignature cannot be decoded from a MultiSignature");

    RelayTxSetup {
        caller: alice.account,
        target: bob.account,
        signature,
        call,
    }
}

benchmarks! {
    _ {}

    batch {
        let c in 1..MAX_CALLS;

        let u = User::<T>::new("ALICE", 1);
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)
    verify {
        // NB In this case we are using `frame_system::Call::<T>::remark` which makes *no DB
        // operations*. This helps us to fetch the DB read/write ops only from `batch` instead of
        // its batched calls.
        // So there is no way to verify it.
        // The following cases use `balances::transfer` to be able to verify their outputs.
    }

    batch_transfer {
        let c in 1..MAX_CALLS;

        let sender = User::<T>::new("SENDER", 1);
        let receiver = User::<T>::new("RECEIVER", 1);

        let transfer_calls = make_transfer_calls::<T>(c, receiver.account.clone(), 500);
    }: batch(sender.origin, transfer_calls)
    verify {
        verify_free_balance::<T>( &sender.account, (1_000_000 - (500 * c)) as u128);
        verify_free_balance::<T>( &receiver.account, (1_000_000 + (500 * c)) as u128);
    }

    batch_atomic {
        let c in 1..MAX_CALLS;

        let alice = User::<T>::new("ALICE", 1);
        let bob = User::<T>::new("BOB", 1);
        let calls = make_transfer_calls::<T>(c, bob.account.clone(), 100);

    }: _(alice.origin, calls)
    verify {
        verify_free_balance::<T>( &alice.account, (1_000_000 - (100 * c)) as u128);
        verify_free_balance::<T>( &bob.account, (1_000_000 + (100 * c)) as u128);
    }

    batch_optimistic {
        let c in 1..MAX_CALLS;

        let alice = User::<T>::new("ALICE", 1);
        let bob = User::<T>::new("BOB", 1);
        let calls = make_transfer_calls::<T>(c, bob.account.clone(), 100);

    }: _(alice.origin, calls)
    verify {
        verify_free_balance::<T>( &alice.account, (1_000_000 - (100 * c)) as u128);
        verify_free_balance::<T>( &bob.account, (1_000_000 + (100 * c)) as u128);
    }

    relay_tx {
        let setup = relay_tx_setup::<T>();
        let origin = RawOrigin::Signed(setup.caller.clone());
    }: _(origin, setup.target.clone(), setup.signature, setup.call)
    verify {
        verify_free_balance::<T>( &setup.caller, 1_000_001u128);
        verify_free_balance::<T>( &setup.target, 999_999u128);
    }
}
