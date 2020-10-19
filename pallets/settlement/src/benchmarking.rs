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

use super::*;
use crate::Module as Settlement;

use pallet_balances as balances;
use pallet_identity as identity;
use polymesh_primitives::{IdentityId, InvestorUid, Ticker};

pub use frame_benchmarking::{account, benchmarks, whitelisted_caller};
use frame_system::RawOrigin;
use sp_std::{convert::TryFrom, iter, prelude::*};

const SEED: u32 = 0;
const MAX_VENUE_DETAILS_LENGTH: u32 = 1024;
const MAX_SIGNERS_ALLOWED: u32 = 100; 

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

benchmarks! {
    _{
        // User account seed
        let u in 0 .. 100 => ();
    }

    create_venue {
        // Variations for the venue_details length.
        let d in 1 .. MAX_VENUE_DETAILS_LENGTH;
        let mut signers = Vec::new();
        let origin = make_account::<T>("caller", 1).1;
        let venue_details = VenueDetails::from(vec![b'D'; d as usize].as_slice());
        let venue_type = VenueType::Distribution;
        // Variations for the no. of signers allowed.
        for s in 0 .. MAX_SIGNERS_ALLOWED {
            signers.push(s);
        }
    }: _(origin, venue_details, signers, venue_type)
}