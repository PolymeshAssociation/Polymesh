use crate::balances;

use srml_support::{ decl_module, decl_storage };
use rstd::vec::Vec;


type AssetId = Vec<u8>;
type Counter = u64;
type CounterDiff = i64;

pub trait Trait: balances::Trait {
}

decl_storage! {
    trait Store for Module<T: Trait> as statistics {
        pub InvestorCountPerAsset get(investor_count_per_asset): map AssetId => Counter
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}

pub trait StatisticTrait<T: Trait> {
    fn update_transfer_stats(
            ticker: &Vec<u8>,
            updated_from_balance: T::Balance,
            updated_to_balance: T::Balance,
            amount: T::Balance);
}

impl<T: Trait> StatisticTrait<T> for Module<T> {
    fn update_transfer_stats(
            ticker: &Vec<u8>,
            updated_from_balance: T::Balance,
            updated_to_balance: T::Balance,
            amount: T::Balance) {
        if amount != 0u128.into() {
            let counter = Self::investor_count_per_asset(ticker);
            let mut new_counter = counter;

            if updated_from_balance == 0u128.into() {
                new_counter = new_counter.checked_sub(1).unwrap_or( new_counter);
            }
            if updated_to_balance == amount {
                new_counter = new_counter.checked_add(1).unwrap_or( new_counter);
            }

            if new_counter != counter {
                <InvestorCountPerAsset>::insert( ticker, new_counter)
            }
        }
    }
}
