//! # SecurityToken offering Module
//!
//! The STO module provides the way of investing into an asset.
//!
//! ## Overview
//!
//! The STO module provides functions for:
//!
//! - Launching a STO for a given asset
//! - Buy asset from a STO
//! - Pause/Unpause feature of the STO.
//!
//! ### Terminology
//!
//! - **Allowed tokens:** It is a list of tokens allowed as an investment currency for a given STO.
//! - **Simple tokens:** These can be wrapped ETH, BTC, DAI or any other blockchain native currency
//! but it can't be a native token neither an asset.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `launch_sto` - Used to initialize the STO for a given asset
//! - `buy_tokens` - Used to bought asset corresponding to the native currency i.e POLY
//! - `modify_allowed_tokens` - Modify the list of tokens used as investment currency for a given STO
//! - `buy_tokens_by_simple_token` - Used to bought assets corresponds to a simple token
//! - `pause_sto` - Used to pause the STO of a given token
//! - `unpause_sto` - Used to un pause the STO of a given token.

use crate::{
    asset::AssetTrait,
    balances, general_tm, identity,
    simple_token::{self, SimpleTokenTrait},
    utils,
};
use primitives::{IdentityId, Key};

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
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type SimpleTokenTrait: simple_token::SimpleTokenTrait<Self::Balance>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct STO<V, W> {
    beneficiary_did: IdentityId,
    cap: V,
    sold: V,
    rate: u128,
    start_date: W,
    end_date: W,
    active: bool,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Debug)]
pub struct Investment<V, W> {
    investor_did: IdentityId,
    amount_paid: V,
    tokens_purchased: V,
    last_purchase_date: W,
}

decl_storage! {
    trait Store for Module<T: Trait> as STOCapped {
        /// Tokens can have multiple whitelists that (for now) check entries individually within each other
        /// (ticker, sto_id) -> STO
        StosByToken get(stos_by_token): map (Vec<u8>, u32) => STO<T::Balance,T::Moment>;
        /// It returns the sto count corresponds to its ticker
        /// ticker -> sto count
        StoCount get(sto_count): map (Vec<u8>) => u32;
        /// List of SimpleToken tokens which will be accepted as the fund raised type for the STO
        /// (asset_ticker, sto_id, index) -> simple_token_ticker
        AllowedTokens get(allowed_tokens): map(Vec<u8>, u32, u32) => Vec<u8>;
        /// To track the index of the token address for the given STO
        /// (Asset_ticker, sto_id, simple_token_ticker) -> index
        TokenIndexForSTO get(token_index_for_sto): map(Vec<u8>, u32, Vec<u8>) => Option<u32>;
        /// To track the no of different tokens allowed as fund raised type for the given STO
        /// (asset_ticker, sto_id) -> count
        TokensCountForSto get(tokens_count_for_sto): map(Vec<u8>, u32) => u32;
        /// To track the investment data of the investor corresponds to ticker
        /// (asset_ticker, sto_id, DID) -> Investment structure
        InvestmentData get(investment_data): map(Vec<u8>, u32, IdentityId) => Investment<T::Balance, T::Moment>;
        /// To track the investment amount of the investor corresponds to ticker using SimpleToken
        /// (asset_ticker, simple_token_ticker, sto_id, accountId) -> Invested balance
        SimpleTokenSpent get(simple_token_token_spent): map(Vec<u8>, Vec<u8>, u32, IdentityId) => T::Balance;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        /// Initializing events
        /// this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Used to initialize the STO for a given asset
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner who wants to initialize the sto
        /// * `did` DID of the token owner
        /// * `_ticker` Ticker of the token
        /// * `beneficiary_did` DID which holds all the funds collected
        /// * `cap` Total amount of tokens allowed for sale
        /// * `rate` Rate of asset in terms of native currency
        /// * `start_date` Unix timestamp at when STO starts
        /// * `end_date` Unix timestamp at when STO ends
        /// * `simple_token_ticker` Ticker of the simple token
        pub fn launch_sto(
            origin,
            did: IdentityId,
            _ticker: Vec<u8>,
            beneficiary_did: IdentityId,
            cap: T::Balance,
            rate: u128,
            start_date: T::Moment,
            end_date: T::Moment,
            simple_token_ticker: Vec<u8>
        ) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let sold:T::Balance = 0.into();
            ensure!(Self::is_owner(&ticker, did),"Sender must be the token owner");

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

        /// Used to buy tokens
        ///
        /// # Arguments
        /// * `origin` Signing key of the investor
        /// * `did` DID of the investor
        /// * `_ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in
        /// * `value` Amount of POLY wants to invest in
        pub fn buy_tokens(origin, did: IdentityId,  _ticker: Vec<u8>, sto_id: u32, value: T::Balance ) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let mut selected_sto = Self::stos_by_token((ticker.clone(), sto_id));
            // Pre validation checks
            ensure!(Self::_pre_validation(&_ticker, did, selected_sto.clone()).is_ok(), "Invalidate investment");
            // Make sure sender has enough balance
            let sender_balance = <balances::Module<T> as Currency<_>>::free_balance(&sender);
            ensure!(sender_balance >= value,"Insufficient funds");

            // Get the invested amount of investment currency and amount of ST tokens minted as a return of investment
            let token_amount_value = Self::_get_invested_amount_and_tokens(
                value,
                selected_sto.clone()
            )?;
            let _allowed_value = token_amount_value.1;

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_amount_value.0)
                .ok_or("overflow while calculating tokens sold")?;

            // Mint tokens and update STO
            T::Asset::_mint_from_sto(&ticker, did, token_amount_value.0)?;

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
                0.into(),
                selected_sto.clone()
            )?;

            Ok(())
        }


        /// Modify the list of allowed tokens (stable coins) corresponds to given token/asset
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `did` DID of the token owner
        /// * `_ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in.
        /// * `simple_token_ticker` Ticker of the stable coin
        /// * `modify_status` Boolean to know whether the provided simple token ticker will be used or not.
        pub fn modify_allowed_tokens(origin, did: IdentityId, _ticker: Vec<u8>, sto_id: u32, simple_token_ticker: Vec<u8>, modify_status: bool) -> Result {
            let sender = ensure_signed(origin)?;

            /// Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            let selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            let now = <timestamp::Module<T>>::get();
            // Right now we are only allowing the issuer to change the configuration only before the STO start not after the start
            // or STO should be in non-active stage
            ensure!(now < selected_sto.start_date || !selected_sto.active, "STO is already started");

            ensure!(Self::is_owner(&ticker,did), "Not authorised to execute this function");

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

        /// Used to buy tokens using stable coins
        ///
        /// # Arguments
        /// * `origin` Signing key of the investor
        /// * `did` DID of the investor
        /// * `_ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in
        /// * `value` Amount of POLY wants to invest in
        /// * `simple_token_ticker` Ticker of the simple token
        pub fn buy_tokens_by_simple_token(origin, did: IdentityId, _ticker: Vec<u8>, sto_id: u32, value: T::Balance, simple_token_ticker: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            let ticker = utils::bytes_to_upper(_ticker.as_slice());

            // Check whether given token is allowed as investment currency or not
            ensure!(Self::token_index_for_sto((ticker.clone(), sto_id, simple_token_ticker.clone())) != None, "Given token is not a permitted investment currency");
            let mut selected_sto = Self::stos_by_token((ticker.clone(),sto_id));
            // Pre validation checks
            ensure!(Self::_pre_validation(&_ticker, did, selected_sto.clone()).is_ok(), "Invalidate investment");
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
            let _minted_tokes = T::Asset::_mint_from_sto(&ticker, did, token_amount_value.0);
            // Transfer the simple_token invested token to beneficiary account
            T::SimpleTokenTrait::transfer(did, &simple_token_ticker, selected_sto.beneficiary_did, token_amount_value.1)?;

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

        /// Pause the STO, Can only be called by the token owner
        /// By doing this every operations on given sto_id would get freezed like buy_tokens
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `did` DID of the token owner
        /// * `_ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to paused
        pub fn pause_sto(origin, did: IdentityId, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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

        /// Un-pause the STO, Can only be called by the token owner
        /// By doing this every operations on given sto_id would get un freezed.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `did` DID of the token owner
        /// * `_ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to un paused
        pub fn unpause_sto(origin, did: IdentityId, _ticker: Vec<u8>, sto_id: u32) -> Result {
            let sender = ensure_signed(origin)?;

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

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
        Balance = <T as balances::Trait>::Balance,
    {
        ModifyAllowedTokens(Vec<u8>, Vec<u8>, u32, bool),
        /// Emit when Asset get purchased by the investor
        /// Ticker, SimpleToken token, sto_id, investor DID, amount invested, amount of token purchased
        AssetPurchase(Vec<u8>, Vec<u8>, u32, IdentityId, Balance, Balance),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker.as_slice());
        T::Asset::is_owner(&upper_ticker, did)
    }

    fn _pre_validation(
        _ticker: &Vec<u8>,
        _did: IdentityId,
        selected_sto: STO<T::Balance, T::Moment>,
    ) -> Result {
        // TODO: Validate that buyer is whitelisted for primary issuance.
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
        invested_amount: T::Balance,
        selected_sto: STO<T::Balance, T::Moment>,
    ) -> core::result::Result<(T::Balance, T::Balance), &'static str> {
        // Calculate tokens to mint
        let mut token_conversion = invested_amount
            .checked_mul(&selected_sto.rate.into())
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
                .checked_div(&selected_sto.rate.into())
                .ok_or("incorrect division")?;
        }
        Ok((token_conversion, allowed_value))
    }

    fn _update_storage(
        ticker: Vec<u8>,
        sto_id: u32,
        did: IdentityId,
        investment_amount: T::Balance,
        new_tokens_minted: T::Balance,
        simple_token_ticker: Vec<u8>,
        simple_token_investment: T::Balance,
        selected_sto: STO<T::Balance, T::Moment>,
    ) -> Result {
        // Store Investment DATA
        let mut investor_holder = Self::investment_data((ticker.clone(), sto_id, did));
        if investor_holder.investor_did == IdentityId::default() {
            investor_holder.investor_did = did.clone();
        }
        investor_holder.tokens_purchased = investor_holder
            .tokens_purchased
            .checked_add(&new_tokens_minted)
            .ok_or("overflow while updating the invested amount")?;
        investor_holder.last_purchase_date = <timestamp::Module<T>>::get();

        if simple_token_ticker != vec![0] {
            <SimpleTokenSpent<T>>::insert(
                (ticker.clone(), simple_token_ticker.clone(), sto_id, did),
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
