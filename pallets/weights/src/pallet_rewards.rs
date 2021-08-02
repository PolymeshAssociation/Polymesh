#![allow(unused_parens)]
#![allow(unused_imports)]

use frame_support::{traits::Get, weights::Weight};
use sp_std::marker::PhantomData;

/// Weight functions for pallet_rewards.
pub struct WeightInfo<T>(PhantomData<T>);
impl<T: frame_system::Config> pallet_rewards::WeightInfo for WeightInfo<T> {
    fn claim_itn_reward() -> Weight {
        (102_522_000 as Weight)
            .saturating_add(T::DbWeight::get().reads(13 as Weight))
            .saturating_add(T::DbWeight::get().writes(7 as Weight))
    }
    fn set_itn_reward_status() -> Weight {
        (1_510_000 as Weight)
            .saturating_add(T::DbWeight::get().writes(1 as Weight))
    }
}
