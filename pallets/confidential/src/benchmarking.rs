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
use pallet_asset as asset;
use pallet_identity::benchmarking::{User, UserBuilder};
use polymesh_common_utilities::traits::asset::AssetType;

use frame_benchmarking::benchmarks;
use sp_std::convert::TryFrom;

const SEED: u32 = 0;
const MAX_TICKER_LENGTH: u8 = 12;

benchmarks! {
    _ {}

    add_range_proof {
        let owner = UserBuilder::<T>::default().build_with_did("owner", SEED);
        let prover = UserBuilder::<T>::default().build_with_did("prover", SEED);
        let verifier = UserBuilder::<T>::default().build_with_did("verifier", SEED);

        let ticker = Ticker::try_from(vec![b'A'; MAX_TICKER_LENGTH as usize].as_slice()).unwrap();
        let sc_name = b"TIC".into();
        asset::Module::<T>::create_asset( owner.origin().into(), sc_name, ticker.clone(), 1_000.into(), true, AssetType::default(), vec![], None)
            .expect("Asset cannot be created");

        let secret_value = 42;

    }: _(prover.origin(), owner.did(), ticker.clone(), secret_value)
    verify {
        let prover_ticker_key = ProverTickerKey { prover: prover.did(), ticker };
        assert_eq!(RangeProofs::contains_key(owner.did(), prover_ticker_key), true);
    }

    /*
    add_verify_range_proof {
    } _(origin,)
    verify {
    }*/
}
