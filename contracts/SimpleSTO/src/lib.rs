#![cfg_attr(not(any(test, feature = "std")), no_std)]

use ink_core::storage;

use ink_lang::contract;

contract! {
    #![env = ink_core::env::DefaultSrmlTypes]

    struct SimpleSTO {
        token_sold: storage::Value<u128>,
        max_limit: storage::Value<u128>,
        reserve_percent: storage::Value<u128>, // enter 1 for 1% i.e. 0.01 ratio
        starting_price: storage::Value<u128>, // price * 10000. Enter 50000 if price is 5
        owner: storage::Value<AccountId>,
    }


    impl Deploy for SimpleSTO {
        fn deploy(&mut self) {
            self.owner.set(env.caller());
            self.token_sold.set(0);
            self.max_limit.set(0);
        }
    }

    impl SimpleSTO {
        /// Sets max token sold
        pub(external) fn configure(&mut self, max_limit: u128, reserve_percent: u128, starting_price: u128) {
            if env.caller() != *self.owner {
                return;
            }
            self.max_limit.set(max_limit);
            self.reserve_percent.set(reserve_percent);
            self.starting_price.set(starting_price);
        }

        /// gets price
        pub(external) fn get_price(&mut self, amount: u128) -> u128 {
            let price = *self.token_sold.get() /
                *self.starting_price.get() + (*self.max_limit.get() - *self.token_sold.get()) * *self.reserve_percent.get();
            let buy_amount = price * amount + *self.token_sold.get();
            if buy_amount > *self.max_limit.get() {
                return 0;
            } else {
                self.token_sold.set(buy_amount);
                return price
            }
        }
    }
}

#[cfg(all(test, feature = "test-env"))]
mod tests {
    use super::*;
    use ink_core::env;
    type Types = ink_core::env::DefaultSrmlTypes;
}
