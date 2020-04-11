use crate::asset::*;
use codec::Encode;
use frame_benchmarking::{account, benchmarks};
use frame_support::{traits::Currency, StorageValue};
use frame_system::RawOrigin;
use polymesh_primitives::{AccountKey, AuthorizationData, IdentityId, Signatory, Ticker};
use polymesh_runtime_balances as balances;
use polymesh_runtime_identity as identity;
use sp_std::{convert::TryFrom, prelude::*};

const SEED: u32 = 0;
const MAX_USER_INDEX: u32 = 1_000;
const MAX_TICKER_LENGTH: u8 = 12;

fn create_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    let _ = identity::Module::<T>::register_did(origin.clone().into(), vec![]);
    let did = identity::Module::<T>::get_identity(&AccountKey::try_from(account.encode()).unwrap())
        .unwrap();
    (account, origin, did)
}

benchmarks! {
    _ {
        // User account seed.
        let u in 1 .. MAX_USER_INDEX => ();
    }

    register_ticker {
        let u in ...;
        let l in 1 .. MAX_TICKER_LENGTH as u32;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let origin = create_account::<T>("caller", u).1;
        // Generate a ticker of length `l`.
        let ticker = Ticker::try_from(vec![b'A'; l as usize].as_slice()).unwrap();
    }: _(origin, ticker)

    accept_ticker_transfer {
        let u in ...;
        <TickerConfig<T>>::put(TickerRegistrationConfig {
            max_ticker_length: MAX_TICKER_LENGTH,
            registration_length: None,
        });
        let (_, alice_origin, alice_did) = create_account::<T>("alice", u);
        let (_, bob_origin, bob_did) = create_account::<T>("bob", u);
        let ticker = Ticker::try_from(
            vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()
        ).unwrap();
        Module::<T>::register_ticker(alice_origin.clone().into(), ticker).unwrap();
        let bob_auth_id = identity::Module::<T>::add_auth(
            Signatory::from(alice_did),
            Signatory::from(bob_did),
            AuthorizationData::TransferTicker(ticker),
            None,
        );
    }: _(bob_origin, bob_auth_id)
}
