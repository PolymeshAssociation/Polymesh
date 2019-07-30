//! A runtime module providing a unique ticker registry
use crate::{asset, erc20};
use parity_codec::{Decode, Encode};
use rstd::prelude::*;
use crate::utils;
use system::{self, ensure_signed};
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub, Convert, StaticLookup};
use support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap};

/// The module's configuration trait.
pub trait Trait: system::Trait + utils::Trait + asset::Trait + erc20::Trait + timestamp::Trait{
    //type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Instruction<U,V,W> {
    /// Total amount to be sold. This amount is deposited in the settlement module by the instruction creator.
    sell_token_amount_left: U,
    sell_token_ticker: Vec<u8>,
    sell_token_regulated: bool,
    buy_tokens_ticker: Vec<Vec<u8>>, //Array of buy tokens
    buy_tokens_regulated: Vec<bool>,
    prices: Option<Vec<U>>, // sell_token_amount * 10^6 = price * buy_token_amount
    price_contract: Option<V>, // Either fixed price is defined above or variable price is used via this contract
    expiry: Option<W>,
    instruction_owner: V,
}

decl_storage! {
    trait Store for Module<T: Trait> as Settlement {
        Instructions get(instructions): map u64 => Instruction<T::TokenBalance, T::AccountId, T::Moment>;
        InstructionsCount get(instructions_count): u64;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        pub fn add_instructions(
            origin,
            _sell_token_amount: T::TokenBalance,
            _sell_token_ticker: Vec<u8>,
            _sell_token_regulated: bool,
            _buy_tokens_ticker: Vec<Vec<u8>>,
            _buy_tokens_regulated: Vec<bool>,
            _prices: Option<Vec<T::TokenBalance>>,
            _price_contract: Option<T::AccountId>,
            _expiry: Option<T::Moment>
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_sell_token_ticker.as_slice());
            ensure!(
                match _prices.clone() {
                    Some(x) => match _price_contract.clone() {
                        Some(y) => false,
                        None => x.len() == _buy_tokens_ticker.len() && x.len() == _buy_tokens_regulated.len(),
                    },
                    None => match _price_contract.clone() {
                        Some(y) => true,
                        None => false,
                    },
                },
                "Invalid buy token details"
            );
            ensure!(_expiry > Some(<timestamp::Module<T>>::get()), "Instruction expiry must be in future");
            if _sell_token_regulated {
                let balance = <asset::BalanceOf<T>>::get((ticker.clone(), sender.clone()));
                let new_balance = balance.checked_sub(&_sell_token_amount).ok_or("underflow calculating new owner balance")?;
                <asset::BalanceOf<T>>::insert((ticker.clone(), sender.clone()), new_balance);
            } else {
                let balance = <erc20::BalanceOf<T>>::get((ticker.clone(), sender.clone()));
                let new_balance = balance.checked_sub(&_sell_token_amount).ok_or("underflow calculating new owner balance")?;
                <erc20::BalanceOf<T>>::insert((ticker.clone(), sender.clone()), new_balance);
            }
            let new_instruction = Instruction {
                sell_token_amount_left: _sell_token_amount,
                sell_token_ticker: _sell_token_ticker,
                sell_token_regulated: _sell_token_regulated,
                buy_tokens_ticker: _buy_tokens_ticker,
                buy_tokens_regulated: _buy_tokens_regulated,
                prices: _prices,
                price_contract: _price_contract,
                expiry: _expiry,
                instruction_owner: sender.clone(),
            };
            let new_count = Self::instructions_count()
                .checked_add(1)
                .ok_or("Could not add 1 to Instruction count")?;
            <Instructions<T>>::insert(new_count.clone(), new_instruction);
            //<InstructionsCount<T>>::put(new_count.clone());
            Ok(())
        }

        pub fn clear_instruction(origin, _instruction_id: u64) -> Result {
            let sender = ensure_signed(origin)?;
            ensure!(<Instructions<T>>::exists(_instruction_id), "No instruction for supplied ID");
            let instruction = <Instructions<T>>::get(_instruction_id);
            ensure!(sender.clone() == instruction.instruction_owner, "Unauthorized");
            if instruction.sell_token_regulated {
                let balance = <asset::BalanceOf<T>>::get((instruction.sell_token_ticker.clone(), sender.clone()));
                let new_balance = balance.checked_add(&instruction.sell_token_amount_left).ok_or("Overflow calculating new owner balance")?;
                <asset::BalanceOf<T>>::insert((instruction.sell_token_ticker.clone(), sender.clone()), new_balance);
            } else {
                let balance = <erc20::BalanceOf<T>>::get((instruction.sell_token_ticker.clone(), sender.clone()));
                let new_balance = balance.checked_add(&instruction.sell_token_amount_left).ok_or("Overflow calculating new owner balance")?;
                <erc20::BalanceOf<T>>::insert((instruction.sell_token_ticker.clone(), sender.clone()), new_balance);
            }
            <Instructions<T>>::remove(_instruction_id);
            Ok(())
        }

        pub fn settle_instruction(
            origin,
            _instruction_id: u64,
            _sell_token_amount: T::TokenBalance,
            _sell_token_ticker: Vec<u8>,
            _sell_token_regulated: bool
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_sell_token_ticker.as_slice());
            ensure!(<Instructions<T>>::exists(_instruction_id), "No instruction for supplied ticker and ID");
            let instruction = <Instructions<T>>::get(_instruction_id);
            let mut exists = false;
            let mut final_index = 0;
            for (index, temp_ticker) in instruction.buy_tokens_ticker.iter().enumerate() {
                if *temp_ticker == ticker.clone() {
                    if instruction.buy_tokens_regulated[index] == _sell_token_regulated {
                        exists = true;
                        final_index = index;
                        break;
                    }
                }
            }
            ensure!(exists, "Sell token not allowed");
            let buy_amount;
            if instruction.prices == None {
                //fetch price from smart contract
                buy_amount = <T::TokenBalance as As<u64>>::sa(1);
            } else {
                let price = instruction.prices.unwrap()[final_index];
                buy_amount = (_sell_token_amount * price)/<T::TokenBalance as As<u64>>::sa(1000000);
                ensure!((buy_amount * <T::TokenBalance as As<u64>>::sa(1000000))/price == _sell_token_amount, "Error in calculation");
            }
            let new_sell_token_amount_left = instruction.sell_token_amount_left
                    .checked_sub(&buy_amount)
                    .ok_or("Underflow in calculating new sell token amount left")?;

            if instruction.sell_token_regulated {
                let balance = <asset::BalanceOf<T>>::get((instruction.sell_token_ticker.clone(), sender.clone()));
                let new_balance = balance.checked_add(&buy_amount).ok_or("Overflow calculating new owner balance")?;
                <asset::BalanceOf<T>>::insert((instruction.sell_token_ticker.clone(), sender.clone()), new_balance);
            } else {
                let balance = <erc20::BalanceOf<T>>::get((instruction.sell_token_ticker.clone(), sender.clone()));
                let new_balance = balance.checked_add(&buy_amount).ok_or("Overflow calculating new owner balance")?;
                <erc20::BalanceOf<T>>::insert((instruction.sell_token_ticker.clone(), sender.clone()), new_balance);
            }

            if _sell_token_regulated {
                let instruction_onwer_balance = <asset::BalanceOf<T>>::get((ticker.clone(), instruction.instruction_owner.clone()));
                let new_instruction_onwer_balance = instruction_onwer_balance
                    .checked_add(&_sell_token_amount)
                    .ok_or("Overflow calculating new owner balance")?;
                let settler_balance = <asset::BalanceOf<T>>::get((ticker.clone(), sender.clone()));
                let new_settler_balance = settler_balance
                    .checked_sub(&_sell_token_amount)
                    .ok_or("Underflow calculating settler balance")?;
                <asset::BalanceOf<T>>::insert((ticker.clone(), instruction.instruction_owner.clone()), new_instruction_onwer_balance);
                <asset::BalanceOf<T>>::insert((ticker.clone(), sender.clone()), new_settler_balance);
            } else {
                let instruction_onwer_balance = <erc20::BalanceOf<T>>::get((ticker.clone(), instruction.instruction_owner.clone()));
                let new_instruction_onwer_balance = instruction_onwer_balance
                    .checked_add(&_sell_token_amount)
                    .ok_or("Overflow calculating new owner balance")?;
                let settler_balance = <erc20::BalanceOf<T>>::get((ticker.clone(), sender.clone()));
                let new_settler_balance = settler_balance
                    .checked_add(&_sell_token_amount)
                    .ok_or("Underflow calculating settler balance")?;
                <erc20::BalanceOf<T>>::insert((ticker.clone(), instruction.instruction_owner.clone()), new_instruction_onwer_balance);
                <erc20::BalanceOf<T>>::insert((ticker.clone(), sender.clone()), new_settler_balance);
            }

            <Instructions<T>>::mutate(_instruction_id, |inst| -> Result {
                inst.sell_token_amount_left = new_sell_token_amount_left;
                Ok(())
            })?;
            Ok(())
        }
       // T::Lookup::unlookup(_contract.clone())

    }
}

impl<T: Trait> Module<T> {
    // pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
    //     let ticker = utils::bytes_to_upper(_ticker.as_slice());
    //     <asset::Module<T>>::_is_owner(ticker.clone(), sender)
    // }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use runtime_primitives::{
        testing::{Digest, DigestItem, Header},
        traits::{BlakeTwo256, IdentityLookup},
        BuildStorage,
    };
    use support::{assert_ok, impl_outer_origin};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type Digest = Digest;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type Log = DigestItem;
    }
    impl Trait for Test {}

}
