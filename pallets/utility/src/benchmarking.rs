use crate::*;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::{
    benchs::{AccountIdOf, User, UserBuilder},
    traits::TestUtilsFn,
};
use sp_core::sr25519::Signature;
use sp_runtime::MultiSignature;

const MAX_CALLS: u32 = 30;

/// Generate `c` no-op system remark calls.
fn make_calls<T: Config>(c: u32) -> Vec<<T as Config>::Call> {
    let call: <T as Config>::Call = frame_system::Call::<T>::remark { remark: vec![] }.into();
    vec![call; c as usize]
}

fn make_relay_tx_users<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, User<T>) {
    let alice = UserBuilder::<T>::default()
        .balance(1_000_000u32)
        .generate_did()
        .build("Caller");
    let bob = UserBuilder::<T>::default()
        .balance(1_000_000u32)
        .generate_did()
        .build("Target");

    (alice, bob)
}

fn remark_call_builder<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    signer: &User<T>,
    _: T::AccountId,
) -> (UniqueCall<<T as Config>::Call>, Vec<u8>) {
    let call = make_calls::<T>(1).pop().unwrap();
    let nonce: AuthorizationNonce = Module::<T>::nonce(signer.account());
    let call = UniqueCall::new(nonce, call);

    // Signer signs the relay call.
    // NB: Decode as T::OffChainSignature because there is not type constraints in
    // `T::OffChainSignature` to limit it.
    let raw_signature: [u8; 64] = signer
        .sign(&call.encode())
        .expect("Data cannot be signed")
        .0;
    let encoded = MultiSignature::from(Signature::from_raw(raw_signature)).encode();

    (call, encoded)
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    batch {
        let c in 0..MAX_CALLS;

        let u = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)
    verify {
        // NB In this case we are using `frame_system::Call::<T>::remark` which makes *no DB
        // operations*. This helps us to fetch the DB read/write ops only from `batch` instead of
        // its batched calls.
        // So there is no way to verify it.
        // The following cases use `balances::transfer` to be able to verify their outputs.
    }

    batch_atomic {
        let c in 0..MAX_CALLS;

        let alice = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);
    }: _(alice.origin, calls)
    verify {
        // NB see comment at `batch` verify section.
    }

    batch_optimistic {
        let c in 0..MAX_CALLS;

        let alice = UserBuilder::<T>::default().generate_did().build("ALICE");
        let calls = make_calls::<T>(c);

    }: _(alice.origin, calls)
    verify {
        // NB see comment at `batch` verify section.
    }

    relay_tx {
        let (caller, target) = make_relay_tx_users::<T>();
        let (call, encoded) = remark_call_builder( &target, caller.account());

        // Rebuild signature from `encoded`.
        let signature = T::OffChainSignature::decode(&mut &encoded[..])
            .expect("OffChainSignature cannot be decoded from a MultiSignature");

    }: _(caller.origin.clone(), target.account(), signature, call)
    verify {
        // NB see comment at `batch` verify section.
    }

}
