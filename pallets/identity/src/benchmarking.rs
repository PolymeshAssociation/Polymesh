use crate::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use polymesh_primitives::SigningItem;
use polymesh_runtime_balances as balances;
use sp_std::{iter, prelude::*};

const SEED: u32 = 0;
const MAX_USER_INDEX: u32 = 1_000;

fn make_account<T: Trait>(name: &'static str, u: u32) -> (T::AccountId, RawOrigin<T::AccountId>) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    (account, origin)
}

benchmarks! {
    _ {
        // User account seed.
        let u in 1 .. MAX_USER_INDEX => ();
    }

    register_did {
        let u in ...;
        // Number of signing items.
        let i in 0 .. 50;
        let origin = make_account::<T>("caller", u).1;
        let signing_items: Vec<SigningItem> = iter::repeat(Default::default())
            .take(i as usize)
            .collect();
    }: _(origin, signing_items)
}
