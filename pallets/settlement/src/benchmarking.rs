// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#![cfg(feature = "runtime-benchmarks")]
use crate::*;

use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_primitives::{IdentityId, InvestorUid, Ticker};

pub use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_std::prelude::*;

const SEED: u32 = 0;
const MAX_VENUE_DETAILS_LENGTH: u32 = 200;
const MAX_SIGNERS_ALLOWED: u32 = 50;

fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

fn make_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    let uid = uid_from_name_and_idx(name, u);
    let _ = identity::Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = identity::Module::<T>::get_identity(&account).unwrap_or_default();
    (account, origin, did)
}

// Create venue by assuming worst case scenario.
fn create_venue_<T: Trait>(name: &'static str, u: u32) -> u64 {
    let make_account_data = make_account::<T>(name, u);
    let origin = make_account_data.1;
    let id = make_account_data.2;
    // Worst case length for the venue details.
    let venue_details = VenueDetails::from(vec![b'D'; 200 as usize].as_slice());
    let venue_type = VenueType::Distribution;
    let mut signers = Vec::with_capacity(MAX_SIGNERS_ALLOWED as usize);
    // Create signers vector.
    for signer in 0..MAX_SIGNERS_ALLOWED {
        signers.push(make_account::<T>("signers", signer).0);
    }
    let _ = Module::<T>::create_venue(origin.into(), venue_details, signers, venue_type);
    Module::<T>::user_venues(id)
        .into_iter()
        .last()
        .unwrap_or_default()
}

benchmarks! {
    _{
        // User account seed
        let u in 0 .. 100 => ();
    }

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        // Variations for the no. of signers allowed.
        let s in 0 .. MAX_SIGNERS_ALLOWED;
        let mut signers = Vec::with_capacity(s as usize);
        let origin = make_account::<T>("caller", SEED).1;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Create signers vector.
        for signer in 0 .. s {
            signers.push(make_account::<T>("signers", signer).0);
        }
    }: _(origin, venue_details, signers, venue_type)

    update_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        // Venue type.
        let venue_type = VenueType::Sto;
        // create venue
        let venue_id = create_venue_::<T>("creator", SEED);
        let origin = make_account::<T>("creator", SEED).1;
    }: _(origin, venue_id, Some(venue_details), Some(venue_type))
}
