use crate::*;
use pallet_balances::{self as balances, Call as BalancesCall};
use pallet_identity::benchmarking::User;

use frame_benchmarking::benchmarks;
use sp_runtime::traits::StaticLookup;

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

benchmarks! {
    _ {}

    batch {
        let c in 1..MAX_CALLS;

        let u = User::<T>::new("ALICE", 1);
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)

    batch_transfer {
        let c in 1..MAX_CALLS;

        let sender = User::<T>::new("SENDER", 1);
        let receiver = User::<T>::new("RECEIVER", 1);

        let transfer_calls = make_transfer_calls::<T>(c, receiver.account.clone(), 500);
    }: batch(sender.origin, transfer_calls)
    verify {
        let exp_balance = (1_000_000 - (500 * c)) as u128;
        let sender_balance = balances::Module::<T>::free_balance(&sender.account);
        assert!( sender_balance <= exp_balance.into());
        let exp_balance = (1_000_000 + (500 * c)) as u128;
        let receiver_balance = balances::Module::<T>::free_balance(&receiver.account);
        assert_eq!( receiver_balance, exp_balance.into());
    }

    batch_atomic {
        let c in 1..MAX_CALLS;

        let u = User::<T>::new("ALICE", 1);
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)


    batch_optimistic {
        let c in 1..MAX_CALLS;

        let u = User::<T>::new("ALICE", 1);
        let calls = make_calls::<T>(c);

    }: _(u.origin, calls)

    /*
    relay_tx {
        let caller = User::<T>::new("Caller", 1);
        let target = User::<T>::new("Target", 1);

        let call: <T as Trait>::Call = frame_system::Call::<T>::remark(vec![]).into();
        let nonce: AuthorizationNonce = Module::<T>::nonce(target.account);
        let relay_call = UniqueCall::new(nonce, call);
        let signature =  target.sign(&relay_call.encode());
        let multi_signature = MultiSignature::Sr25519(signature.into());

    }: _(caller.origin, target.account, multi_signature, relay_call)
    */
}
