use crate::{
    asset::AssetTrait,
    balances, general_tm, identity,
    simple_token::{self, SimpleTokenTrait},
    utils,
};
use primitives::Key;

use codec::Encode;
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use srml_support::traits::Currency;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait:
    timestamp::Trait + system::Trait + utils::Trait + balances::Trait + general_tm::Trait
{
    // TODO: Add other types and constants required configure this module.

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type SimpleTokenTrait: simple_token::SimpleTokenTrait<Self::TokenBalance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct STO<V, W> {
    beneficiary_did: Vec<u8>,
    cap: V,
    sold: V,
    rate: u64,
    start_date: W,
    end_date: W,
    active: bool,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Investment<V, W> {
    investor_did: Vec<u8>,
    amount_paid: V,
    tokens_purchased: V,
    last_purchase_date: W,
}

decl_storage! {
    trait Store for Module<T: Trait> as STOCapped {

        // Tokens can have multiple whitelists that (for now) check entries individually within each other
        StosByToken get(stos_by_token): map (Vec<u8>, u32) => STO<T::TokenBalance,T::Moment>;

        StoCount get(sto_count): map (Vec<u8>) => u32;

        // List of SimpleToken tokens which will be accepted as the fund raised type for the STO
        // [asset_ticker][sto_id][index] => simple_token_ticker
        AllowedTokens get(allowed_tokens): map(Vec<u8>, u32, u32) => Vec<u8>;
        // To track the index of the token address for the given STO
        // [Asset_ticker][sto_id][simple_token_ticker] => index
        TokenIndexForSTO get(token_index_for_sto): map(Vec<u8>, u32, Vec<u8>) => Option<u32>;
        // To track the no of different tokens allowed as fund raised type for the given STO
        // [asset_ticker][sto_id] => count
        TokensCountForSto get(tokens_count_for_sto): map(Vec<u8>, u32) => u32;
        // To track the investment data of the investor corresponds to ticker
        //[asset_ticker][sto_id][DID] => Investment structure
        InvestmentData get(investment_data): map(Vec<u8>, u32, Vec<u8>) => Investment<T::TokenBalance, T::Moment>;
        // To track the investment amount of the investor corresponds to ticker using SimpleToken
        // [asset_ticker][simple_token_ticker][sto_id][accountId] => Invested balance
        SimpleTokenSpent get(simple_token_token_spent): map(Vec<u8>, Vec<u8>, u32, Vec<u8>) => T::TokenBalance;

    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        pub fn launch_sto(
            origin,
            did: Vec<u8>,
            _ticker: Vec<u8>,
            beneficiary_did: Vec<u8>,
            cap: T::TokenBalance,
            rate: u64,
            start_date: T::Moment,
            end_date: T::Moment,
            simple_token_ticker: Vec<u8>
        ) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sold:T::TokenBalance = 0.into();
            ensure!(Self::is_owner(&ticker, &did),"Sender must be the token owner");

            let sto = STO {
                beneficiary_did,
                cap,
                sold,
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
            <StoCount>::insert(ticker.clone(),new_sto_count);

            if simple_token_ticker.len() > 0 {
                // Addition of the SimpleToken token as the fund raised type.
                <TokenIndexForSTO>::insert((ticker.clone(), sto_count, simple_token_ticker.clone()), new_token_count);
                <AllowedTokens>::insert((ticker.clone(), sto_count, new_token_count), simple_token_ticker.clone());
                <TokensCountForSto>::insert((ticker.clone(), sto_count), new_token_count);

                Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, simple_token_ticker, sto_count, true));
            }
            sr_primitives::print("Capped STOlaunched!!!");

            Ok(())
        }

        pub fn buy_tokens(origin, did: Vec<u8>,  _ticker: Vec<u8>, sto_id: u32, value: T::Balance ) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let mut selected_sto = Self::stos_by_token((ticker.clone(), sto_id));
            // Pre validation checks
            ensure!(Self::_pre_validation(&_ticker, &did, selected_sto.clone()).is_ok(), "Invalidate investment");
            // Make sure sender has enough balance
            let sender_balance = <balances::Module<T> as Currency<_>>::free_balance(&sender);
            ensure!(sender_balance >= value,"Insufficient funds");

            // Get the invested amount of investment currency and amount of ST tokens minted as a return of investment
            let token_amount_value = Self::_get_invested_amount_and_tokens(
                <T as utils::Trait>::balance_to_token_balance(value),
                selected_sto.clone()
            )?;
            let _allowed_value = <T as utils::Trait>::token_balance_to_balance(token_amount_value.1);

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_amount_value.0)
                .ok_or("overflow while calculating tokens sold")?;

            // Mint tokens and update STO
            T::Asset::_mint_from_sto(&ticker, &did, token_amount_value.0)?;

            // Transfer poly to token owner
            // TODO: transfer between DIDs
            //<balances::Module<T> as Currency<_>>::transfer(
                //&sender,
                //&selected_sto.beneficiary_did,
                //allowed_value
                //)?;

            // Update storage values
            Self::_update_storage(
                ticker.clone(),
                sto_id.clone(),
                did.clone(),
                token_amount_value.1,
                token_amount_value.0,
                vec![0],
                <T as utils::Trait>::as_tb(0_u128),
                selected_sto.clone()
            )?;

            Ok(())
        }

        pub fn modify_allowed_tokens(origin, did: Vec<u8>, _ticker: Vec<u8>, sto_id: u32, simple_token_ticker: Vec<u8>, modify_status: bool) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            let selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            let now = <timestamp::Module<T>>::get();
            // Right now we are only allowing the issuer to change the configuration only before the STO start not after the start
            // or STO should be in non-active stage
            ensure!(now < selected_sto.start_date || !selected_sto.active, "STO is already started");

            ensure!(Self::is_owner(&ticker,&did), "Not authorised to execute this function");

            let token_index = Self::token_index_for_sto((ticker.clone(), sto_id, simple_token_ticker.clone()));
            let token_count = Self::tokens_count_for_sto((ticker.clone(), sto_id));

            let current_status = match token_index == None {
                true => false,
                false => true,
            };

            ensure!(current_status != modify_status, "Already in that state");

            if modify_status {
                let new_count = token_count.checked_add(1).ok_or("overflow new token count value")?;
                <TokenIndexForSTO>::insert((ticker.clone(), sto_id, simple_token_ticker.clone()), new_count);
                <AllowedTokens>::insert((ticker.clone(), sto_id, new_count), simple_token_ticker.clone());
                <TokensCountForSto>::insert((ticker.clone(), sto_id), new_count);
            } else {
                let new_count = token_count.checked_sub(1).ok_or("underflow new token count value")?;
                <TokenIndexForSTO>::insert((ticker.clone(), sto_id, simple_token_ticker.clone()), new_count);
                <AllowedTokens>::insert((ticker.clone(), sto_id, new_count), vec![]);
                <TokensCountForSto>::insert((ticker.clone(), sto_id), new_count);
            }

            Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, simple_token_ticker, sto_id, modify_status));

            Ok(())

        }

        pub fn buy_tokens_by_simple_token(origin, did: Vec<u8>, _ticker: Vec<u8>, sto_id: u32, value: T::TokenBalance, simple_token_ticker: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            // Check whether given token is allowed as investment currency or not
            ensure!(Self::token_index_for_sto((ticker.clone(), sto_id, simple_token_ticker.clone())) != None, "Given token is not a permitted investment currency");
            let mut selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            // Pre validation checks
            ensure!(Self::_pre_validation(&_ticker, &did, selected_sto.clone()).is_ok(), "Invalidate investment");
            // Make sure sender has enough balance
            ensure!(T::SimpleTokenTrait::balance_of(simple_token_ticker.clone(), did.clone()) >= value, "Insufficient balance");

            // Get the invested amount of investment currency and amount of ST tokens minted as a return of investment
            let token_amount_value = Self::_get_invested_amount_and_tokens(
                value,
                selected_sto.clone()
            )?;

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_amount_value.0)
                .ok_or("overflow while calculating tokens sold")?;

            let simple_token_investment = (Self::simple_token_token_spent((ticker.clone(), simple_token_ticker.clone(), sto_id, did.clone())))
                                    .checked_add(&token_amount_value.1)
                                    .ok_or("overflow while updating the simple_token investment value")?;

            // Mint tokens and update STO
            let _minted_tokes = T::Asset::_mint_from_sto(&ticker, &did, token_amount_value.0);
            // Transfer the simple_token invested token to beneficiary account
            T::SimpleTokenTrait::transfer(&did, &simple_token_ticker, &selected_sto.beneficiary_did, token_amount_value.1)?;

            // Update storage values
            Self::_update_storage(
                ticker.clone(),
                sto_id.clone(),
                did.clone(),
                token_amount_value.1,
                token_amount_value.0,
                simple_token_ticker.clone(),
                simple_token_investment,
                selected_sto.clone()
            )?;
            Ok(())
        }

        pub fn pause_sto(origin, did: Vec<u8>, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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

        pub fn unpause_sto(origin, did: Vec<u8>, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(&did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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
        Balance = <T as utils::Trait>::TokenBalance,
    {
        ModifyAllowedTokens(Vec<u8>, Vec<u8>, u32, bool),
        //Emit when Asset get purchased by the investor
        // Ticker, SimpleToken token, sto_id, investor DID, amount invested, amount of token purchased
        AssetPurchase(Vec<u8>, Vec<u8>, u32, Vec<u8>, Balance, Balance),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Vec<u8>, did: &Vec<u8>) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker.as_slice());
        T::Asset::is_owner(&upper_ticker, did)
    }

    fn _pre_validation(
        ticker: &Vec<u8>,
        did: &Vec<u8>,
        selected_sto: STO<T::TokenBalance, T::Moment>,
    ) -> Result {
        // Validate that buyer is whitelisted for primary issuance.
        ensure!(
            <general_tm::Module<T>>::is_whitelisted(ticker, did).is_ok(),
            "sender is not allowed to invest"
        );
        // Check whether the sto is unpaused or not
        ensure!(selected_sto.active, "sto is paused");
        // Check whether the sto is already ended
        let now = <timestamp::Module<T>>::get();
        ensure!(
            now >= selected_sto.start_date && now <= selected_sto.end_date,
            "STO has not started or already ended"
        );
        Ok(())
    }

    fn _get_invested_amount_and_tokens(
        invested_amount: T::TokenBalance,
        selected_sto: STO<T::TokenBalance, T::Moment>,
    ) -> core::result::Result<(T::TokenBalance, T::TokenBalance), &'static str> {
        // Calculate tokens to mint
        let mut token_conversion = invested_amount
            .checked_mul(&<T as utils::Trait>::as_tb(selected_sto.rate.into()))
            .ok_or("overflow in calculating tokens")?;
        let allowed_token_sold = selected_sto
            .cap
            .checked_sub(&selected_sto.sold)
            .ok_or("underflow while calculating the amount of token sold")?;
        let mut allowed_value = invested_amount;
        // Make sure there's still an allocation
        // Instead of reverting, buy up to the max and refund excess amount of investment currency.
        if token_conversion > allowed_token_sold {
            token_conversion = allowed_token_sold;
            allowed_value = token_conversion
                .checked_div(&<T as utils::Trait>::as_tb(selected_sto.rate.into()))
                .ok_or("incorrect division")?;
        }
        Ok((token_conversion, allowed_value))
    }

    fn _update_storage(
        ticker: Vec<u8>,
        sto_id: u32,
        did: Vec<u8>,
        investment_amount: T::TokenBalance,
        new_tokens_minted: T::TokenBalance,
        simple_token_ticker: Vec<u8>,
        simple_token_investment: T::TokenBalance,
        selected_sto: STO<T::TokenBalance, T::Moment>,
    ) -> Result {
        // Store Investment DATA
        let mut investor_holder = Self::investment_data((ticker.clone(), sto_id, did.clone()));
        if investor_holder.investor_did == Vec::<u8>::default() {
            investor_holder.investor_did = did.clone();
        }
        investor_holder.tokens_purchased = investor_holder
            .tokens_purchased
            .checked_add(&new_tokens_minted)
            .ok_or("overflow while updating the invested amount")?;
        investor_holder.last_purchase_date = <timestamp::Module<T>>::get();

        if simple_token_ticker != vec![0] {
            <SimpleTokenSpent<T>>::insert(
                (
                    ticker.clone(),
                    simple_token_ticker.clone(),
                    sto_id,
                    did.clone(),
                ),
                simple_token_investment,
            );
        } else {
            investor_holder.amount_paid = investor_holder
                .amount_paid
                .checked_add(&investment_amount)
                .ok_or("overflow while updating the invested amount")?;
        }
        <StosByToken<T>>::insert((ticker.clone(), sto_id), selected_sto);
        // Emit Event
        Self::deposit_event(RawEvent::AssetPurchase(
            ticker,
            simple_token_ticker,
            sto_id,
            did,
            investment_amount,
            new_tokens_minted,
        ));
        sr_primitives::print("Invested in STO");
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    /*
     *    use super::*;
     *
     *    use substrate_primitives::{Blake2Hasher, H256};
     *    use sr_io::with_externalities;
     *    use sr_primitives::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use srml_support::{assert_ok, impl_outer_origin};
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
     *        type Digest = H256;
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
     *    fn new_test_ext() -> sr_io::TestExternalities<Blake2Hasher> {
     *        system::GenesisConfig::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     */
}
