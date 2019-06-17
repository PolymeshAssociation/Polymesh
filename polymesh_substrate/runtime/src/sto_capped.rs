use crate::asset::AssetTrait;
use crate::erc20::{self, ERC20Trait};
use crate::general_tm;
use crate::utils;
use support::traits::Currency;

use rstd::prelude::*;
use runtime_primitives::traits::{As, CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure, StorageMap};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait:
    timestamp::Trait + system::Trait + utils::Trait + balances::Trait + general_tm::Trait
{
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type ERC20Trait: erc20::ERC20Trait<Self::AccountId, Self::TokenBalance>;
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct STO<U, V, W> {
    beneficiary: U,
    cap: V,
    sold: V,
    rate: u64,
    start_date: W,
    end_date: W,
    active: bool,
}

#[derive(parity_codec::Encode, parity_codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Investment<U, V, W> {
    investor: U,
    amount_payed: V,
    tokens_purchased: V,
    last_purchase_date: W,
}

decl_storage! {
    trait Store for Module<T: Trait> as STOCapped {

        // Tokens can have multiple whitelists that (for now) check entries individually within each other
        StosByToken get(stos_by_token): map (Vec<u8>, u32) => STO<T::AccountId,T::TokenBalance,T::Moment>;

        StoCount get(sto_count): map (Vec<u8>) => u32;

        // List of ERC20 tokens which will be accepted as the fund raised type for the STO
        // [asset_ticker][sto_id][index] => erc20_ticker
        AllowedTokens get(allowed_tokens): map(Vec<u8>, u32, u32) => Vec<u8>;
        // To track the index of the token address for the given STO
        // [Asset_ticker][sto_id][erc20_ticker] => index
        TokenIndexForSTO get(token_index_for_sto): map(Vec<u8>, u32, Vec<u8>) => Option<u32>;
        // To track the no of different tokens allowed as fund raised type for the given STO
        // [asset_ticker][sto_id] => count
        TokensCountForSto get(tokens_count_for_sto): map(Vec<u8>, u32) => u32;
        // To track the investment data of the investor corresponds to ticker
        //[asset_ticker][erc20_ticker][sto_id][accountId] => Investment structure
        InvestmentData get(investment_data): map(Vec<u8>, u32, T::AccountId) => Investment<T::AccountId, T::TokenBalance, T::Moment>;
        // To track the investment amount of the investor corresponds to ticker using ERC20
        // [asset_ticker][erc20_ticker][sto_id][accountId] => Invested balance
        Erc20TokenSpent get(erc20_token_spent): map(Vec<u8>, Vec<u8>, u32, T::AccountId) => T::TokenBalance;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event<T>() = default;

        pub fn launch_sto(
            origin, _ticker: Vec<u8>,
            beneficiary: T::AccountId,
            cap: T::TokenBalance,
            rate: u64,
            start_date: T::Moment,
            end_date: T::Moment,
            erc20_ticker: Vec<u8>
        ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");

            let sto = STO {
                beneficiary,
                cap,
                sold:<T::TokenBalance as As<u64>>::sa(0),
                rate,
                start_date,
                end_date,
                active: true
            };

            let sto_count = Self::sto_count(ticker.clone());
            let new_sto_count = sto_count
                .checked_add(1)
                .ok_or("overflow in calculating next sto count")?;

            let token_count = Self::tokens_count_for_sto((ticker.clone(), sto_count));
            let new_token_count = token_count.checked_add(1).ok_or("overflow new token count value")?;

            <StosByToken<T>>::insert((ticker.clone(),sto_count), sto);
            <StoCount<T>>::insert(ticker.clone(),new_sto_count);

            if erc20_ticker.len() > 0 {
                // Addition of the ERC20 token as the fund raised type.
                <TokenIndexForSTO<T>>::insert((ticker.clone(), sto_count, erc20_ticker.clone()), new_token_count);
                <AllowedTokens<T>>::insert((ticker.clone(), sto_count, new_token_count), erc20_ticker.clone());
                <TokensCountForSto<T>>::insert((ticker.clone(), sto_count), new_token_count);

                Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, erc20_ticker, sto_count, true));
            }
            runtime_io::print("Capped STOlaunched!!!");

            Ok(())
        }

        pub fn buy_tokens(origin, _ticker: Vec<u8>, sto_id: u32, value: T::Balance ) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            // Validate that buyer is whitelisted for primary issuance.
            ensure!(Self::is_whitelisted(_ticker.clone(), sender.clone()),"sender is not allowed to invest");

            let mut selected_sto = Self::stos_by_token((ticker.clone(), sto_id));
            // Check whether the sto is unpaused or not
            ensure!(selected_sto.active, "sto is paused");
            // Check whether the sto is already ended
            let now = <timestamp::Module<T>>::get();
            ensure!(now >= selected_sto.start_date && now <= selected_sto.end_date,"STO has not started or already ended");
            // Make sure sender has enough balance
            let sender_balance = <balances::Module<T> as Currency<_>>::free_balance(&sender);
            ensure!(sender_balance >= value,"Insufficient funds");
            // Calculate tokens to mint
            let mut token_conversion = <T::TokenBalance as As<T::Balance>>::sa(value).checked_mul(&<T::TokenBalance as As<u64>>::sa(selected_sto.rate))
                .ok_or("overflow in calculating tokens")?;
            let allowed_token_sold = selected_sto.cap
                .checked_sub(&selected_sto.sold)
                .ok_or("underflow while calculating the amount of token sold")?;
            let mut allowed_value = value;
            // Make sure there's still an allocation
            // Instead of reverting, buy up to the max and refund excess of poly.
            if token_conversion > allowed_token_sold {
                token_conversion = allowed_token_sold;
                allowed_value = <T::Balance as As<_>>::sa(
                        (
                            <T as utils::Trait>::as_u128
                            (
                            token_conversion
                            .checked_div(&<T::TokenBalance as As<u64>>::sa(selected_sto.rate))
                            .ok_or("incorrect division")?
                            )
                        ) as u64
                    );
            }

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_conversion)
                .ok_or("overflow while calculating tokens sold")?;

            // Mint tokens and update STO
            T::Asset::_mint_from_sto(ticker.clone(), sender.clone(), token_conversion)?;

            // Transfer poly to token owner
            <balances::Module<T> as Currency<_>>::transfer(
                &sender,
                &selected_sto.beneficiary,
                allowed_value
                )?;

            <StosByToken<T>>::insert((ticker.clone(),sto_id), selected_sto);
            // Store Investment DATA
            let mut investor_holder = Self::investment_data((ticker.clone(), sto_id, sender.clone()));
            if investor_holder.investor != sender {
                investor_holder.investor = sender.clone();
            }
            investor_holder.amount_payed = investor_holder.amount_payed
                .checked_add(&<T::TokenBalance as As<T::Balance>>::sa(allowed_value))
                .ok_or("overflow while updating the invested amount")?;
            investor_holder.tokens_purchased = investor_holder.tokens_purchased
                .checked_add(&token_conversion)
                .ok_or("overflow while updating the invested amount")?;
            investor_holder.last_purchase_date = <timestamp::Module<T>>::get();
            // Emit Event
            Self::deposit_event(RawEvent::AssetPurchase(ticker, vec![0], sto_id, sender, <T::TokenBalance as As<T::Balance>>::sa(allowed_value), token_conversion));
            runtime_io::print("Invested in STO");
            Ok(())
        }

        pub fn modify_allowed_tokens(origin, _ticker: Vec<u8>, sto_id: u32, erc20_ticker: Vec<u8>, modify_status: bool) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            let selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            let now = <timestamp::Module<T>>::get();
            // Right now we are only allowing the issuer to change the configuration only before the STO start not after the start
            // or STO should be in non-active stage
            ensure!(now < selected_sto.start_date || !selected_sto.active, "STO is already started");

            ensure!(Self::is_owner(ticker.clone(),sender), "Not authorised to execute this function");

            let token_index = Self::token_index_for_sto((ticker.clone(), sto_id, erc20_ticker.clone()));
            let token_count = Self::tokens_count_for_sto((ticker.clone(), sto_id));

            let current_status = match token_index == None {
                true => false,
                false => true,
            };

            ensure!(current_status != modify_status, "Already in that state");

            if modify_status {
                let new_count = token_count.checked_add(1).ok_or("overflow new token count value")?;
                <TokenIndexForSTO<T>>::insert((ticker.clone(), sto_id, erc20_ticker.clone()), new_count);
                <AllowedTokens<T>>::insert((ticker.clone(), sto_id, new_count), erc20_ticker.clone());
                <TokensCountForSto<T>>::insert((ticker.clone(), sto_id), new_count);
            } else {
                let new_count = token_count.checked_sub(1).ok_or("underflow new token count value")?;
                <TokenIndexForSTO<T>>::insert((ticker.clone(), sto_id, erc20_ticker.clone()), new_count);
                <AllowedTokens<T>>::insert((ticker.clone(), sto_id, new_count), vec![]);
                <TokensCountForSto<T>>::insert((ticker.clone(), sto_id), new_count);
            }

            Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, erc20_ticker, sto_id, modify_status));

            Ok(())

        }

        pub fn buy_tokens_by_erc20(origin, _ticker: Vec<u8>, sto_id: u32, value: T::TokenBalance, erc20_ticker: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            // Check whether given token is allowed as investment currency or not
            ensure!(Self::token_index_for_sto((ticker.clone(), sto_id, erc20_ticker.clone())) != None, "Given token is not a permitted investment currency");
            let mut selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            // Validate that buyer is whitelisted for primary issuance.
            ensure!(Self::is_whitelisted(_ticker.clone(), sender.clone()),"sender is not allowed to invest");
            let now = <timestamp::Module<T>>::get();
            ensure!(now >= selected_sto.start_date && now <= selected_sto.end_date, "STO has not started or already ended");
            // Check whether the sto is unpaused or not
            ensure!(selected_sto.active, "STO is not active at the moment");
            ensure!(T::ERC20Trait::balanceOf(erc20_ticker.clone(), sender.clone()) >= value, "Insufficient balance");

            //  Calculate tokens to mint
            let mut token_conversion = value.checked_mul(&<T::TokenBalance as As<u64>>::sa(selected_sto.rate))
                .ok_or("overflow in calculating tokens")?;
            let allowed_token_sold = selected_sto.cap
                .checked_sub(&selected_sto.sold)
                .ok_or("underflow while calculating the amount of token sold")?;
            let mut allowed_value = value;

            // Make sure there's still an allocation
            // Instead of reverting, buy up to the max and refund excess of Erc20.
            if token_conversion > allowed_token_sold {
                token_conversion = allowed_token_sold;
                allowed_value = token_conversion
                            .checked_div(&<T::TokenBalance as As<u64>>::sa(selected_sto.rate))
                            .ok_or("incorrect division")?;
            }

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_conversion)
                .ok_or("overflow while calculating tokens sold")?;

            let erc20_investment = (Self::erc20_token_spent((ticker.clone(), erc20_ticker.clone(), sto_id, sender.clone())))
                                    .checked_add(&allowed_value)
                                    .ok_or("overflow while updating the erc20 investment value")?;

            // Mint tokens and update STO
            T::Asset::_mint_from_sto(ticker.clone(), sender.clone(), token_conversion);

            T::ERC20Trait::transfer(sender.clone(), erc20_ticker.clone(), selected_sto.beneficiary.clone(), allowed_value)?;

            // Store Investment DATA
            let mut investor_holder = Self::investment_data((ticker.clone(), sto_id, sender.clone()));
            if investor_holder.investor != sender {
                investor_holder.investor = sender.clone();
            }
            investor_holder.tokens_purchased = investor_holder.tokens_purchased
                .checked_add(&token_conversion)
                .ok_or("overflow while updating the invested amount")?;
            investor_holder.last_purchase_date = <timestamp::Module<T>>::get();

            <Erc20TokenSpent<T>>::insert((ticker.clone(), erc20_ticker.clone(), sto_id, sender.clone()), erc20_investment);
            <StosByToken<T>>::insert((ticker.clone(),sto_id), selected_sto);
            // Emit Event
            Self::deposit_event(RawEvent::AssetPurchase(ticker, erc20_ticker, sto_id, sender, allowed_value, token_conversion));
            runtime_io::print("Invested in STO");

            Ok(())
        }

        pub fn pause_sto(origin, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            // Check valid STO id
            ensure!(Self::sto_count(ticker.clone()) >= sto_id, "Invalid sto id");
            // Access the STO data
            let mut selected_sto = Self::stos_by_token((ticker.clone(), sto_id));
            // Check the flag
            ensure!(selected_sto.active, "Already paused");
            // Change the flag
            selected_sto.active = false;
            // Update the storage
            <StosByToken<T>>::insert((ticker.clone(),sto_id), selected_sto);
            Ok(())
        }

        pub fn unpause_sto(origin, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            // Check valid STO id
            ensure!(Self::sto_count(ticker.clone()) >= sto_id, "Invalid sto id");
            // Access the STO data
            let mut selected_sto = Self::stos_by_token((ticker.clone(), sto_id));
            // Check the flag
            ensure!(!selected_sto.active, "Already in the active state");
            // Change the flag
            selected_sto.active = true;
            // Update the storage
            <StosByToken<T>>::insert((ticker.clone(),sto_id), selected_sto);
            Ok(())
        }

    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as utils::Trait>::TokenBalance,
    {
        Example(u32, AccountId, AccountId),
        ModifyAllowedTokens(Vec<u8>, Vec<u8>, u32, bool),
        //Emit when Asset get purchased by the investor
        // Ticker, Erc20 token, sto_id, investor address, amount invested, amount of token purchased
        AssetPurchase(Vec<u8>, Vec<u8>, u32, AccountId, Balance, Balance),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        T::Asset::is_owner(ticker.clone(), sender)
    }

    pub fn is_whitelisted(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        <general_tm::Module<T>>::is_whitelisted(_ticker, sender)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use primitives::{Blake2Hasher, H256};
     *    use runtime_io::with_externalities;
     *    use runtime_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use support::{assert_ok, impl_outer_origin};
     *
     *    impl_outer_origin! {
     *        pub enum Origin for Test {}
     *    }
     *
     *    // For testing the module, we construct most of a mock runtime. This means
     *    // first constructing a configuration type (`Test`) which `impl`s each of the
     *    // configuration traits of modules we want to use.
     *    #[derive(Clone, Eq, PartialEq)]
     *    pub struct Test;
     *    impl system::Trait for Test {
     *        type Origin = Origin;
     *        type Index = u64;
     *        type BlockNumber = u64;
     *        type Hash = H256;
     *        type Hashing = BlakeTwo256;
     *        type Digest = Digest;
     *        type AccountId = u64;
     *        type Lookup = IdentityLookup<Self::AccountId>;
     *        type Header = Header;
     *        type Event = ();
     *        type Log = DigestItem;
     *    }
     *    impl Trait for Test {
     *        type Event = ();
     *    }
     *    type TransferValidationModule = Module<Test>;
     *
     *    // This function basically just builds a genesis storage key/value store according to
     *    // our desired mockup.
     *    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::<Test>::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     */
}
