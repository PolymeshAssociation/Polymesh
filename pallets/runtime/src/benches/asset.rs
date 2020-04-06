use crate::asset::*;
use frame_benchmarking::{account, benchmarks};
use frame_system::RawOrigin;
use polymesh_primitives::Ticker;
use sp_std::prelude::*;

use Module as Benchmark;

const SEED: u32 = 0;

benchmarks! {
    _ {
        let m in 1 .. 1000 => {
            // let origin = RawOrigin::Signed(account("member", m, SEED));
            // Benchmark::<T>::register_ticker(origin.into(), Default::default())?
        };
    }

    register_ticker {
        let m in ...;
        let l in 1 .. 12;
        // Generate a ticker of length `l`.
        let ticker = Ticker::from(vec![b'A'; l as usize].as_slice());
    }: _(RawOrigin::Signed(account("member", m + 1, SEED)), ticker)
}
