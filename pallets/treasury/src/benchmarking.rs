// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use crate::*;

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use pallet_identity::benchmarking::UserBuilder;
use sp_std::vec::Vec;

const MAX_BENEFICIARIES: u32 = 128;
const REWARD: u32 = 10;

benchmarks! {
    disbursement {
        let b in 1..MAX_BENEFICIARIES;
        let initial_balance = Pallet::<T>::balance();
        // Refill treasury
        let refiller = UserBuilder::<T>::default().balance(200u32 + REWARD * b).generate_did().build("refiller");
        Pallet::<T>::reimbursement( refiller.origin().into(), (100 + (REWARD * b)).into())
            .expect("Tresury cannot be refill");

        // Create beneficiaries
        let beneficiaries = (0..b).map( |idx| {
            let user = UserBuilder::<T>::default()
                .balance(100u32)
                .seed(idx)
                .generate_did()
                .build("beneficiary")
                .did();

            Beneficiary { id: user, amount: REWARD.into() }
        }).collect::<Vec<_>>();

    }: _(RawOrigin::Root, beneficiaries)
    verify {
        assert_eq!(Pallet::<T>::balance(), (initial_balance + 100u32.into()));
    }

    reimbursement {
        let initial_balance = Pallet::<T>::balance();
        let caller = UserBuilder::<T>::default().balance(1_000u32).generate_did().build("caller");
        let amount = 500u32.into();
    }: _(caller.origin(), amount)
    verify {
        assert_eq!(Pallet::<T>::balance(), (initial_balance + 500u32.into()));
    }
}
