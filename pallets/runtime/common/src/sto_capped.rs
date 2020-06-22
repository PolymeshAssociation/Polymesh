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
//! - Pause/Un-pause feature of the STO.
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

use crate::simple_token::{self, SimpleTokenTrait};

use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, balances::Trait as BalancesTrait, CommonTrait, Context,
};
use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};

use codec::Encode;
use frame_support::traits::Currency;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_signed};
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use sp_std::{convert::TryFrom, prelude::*};

/// The module's configuration trait.
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + BalancesTrait + compliance_manager::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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
    assets_purchased: V,
    last_purchase_date: W,
}

decl_storage! {
    trait Store for Module<T: Trait> as StoCapped {
        /// Tokens can have multiple exemptions that (for now) check entries individually within each other
        /// (ticker, sto_id) -> STO
        StosByToken get(fn stos_by_token): map hasher(blake2_128_concat) (Ticker, u32) => STO<T::Balance,T::Moment>;
        /// It returns the sto count corresponds to its ticker
        /// ticker -> sto count
        StoCount get(fn sto_count): map hasher(blake2_128_concat) Ticker => u32;
        /// List of SimpleToken tokens which will be accepted as the fund raised type for the STO
        /// (asset_ticker, sto_id, index) -> simple_token_ticker
        AllowedTokens get(fn allowed_tokens): map hasher(blake2_128_concat) (Ticker, u32, u32) => Ticker;
        /// To track the index of the token address for the given STO
        /// (Asset_ticker, sto_id, simple_token_ticker) -> index
        TokenIndexForSTO get(fn token_index_for_sto): map hasher(blake2_128_concat) (Ticker, u32, Ticker) => Option<u32>;
        /// To track the no of different tokens allowed as fund raised type for the given STO
        /// (asset_ticker, sto_id) -> count
        TokensCountForSto get(fn tokens_count_for_sto): map hasher(blake2_128_concat) (Ticker, u32) => u32;
        /// To track the investment data of the investor corresponds to ticker
        /// (asset_ticker, sto_id, DID) -> Investment structure
        InvestmentData get(fn investment_data): map hasher(blake2_128_concat) (Ticker, u32, IdentityId) => Investment<T::Balance, T::Moment>;
        /// To track the investment amount of the investor corresponds to ticker using SimpleToken
        /// (asset_ticker, simple_token_ticker, sto_id, accountId) -> Invested balance
        SimpleTokenSpent get(fn simple_token_token_spent): map hasher(blake2_128_concat) (Ticker, Ticker, u32, IdentityId) => T::Balance;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a signing key for the DID.
        SenderMustBeSigningKeyForDid,
        /// The sender is not a token owner.
        NotAnOwner,
        /// Pre-validation checks failed.
        PrevalidationFailed,
        /// Insufficient sender balance.
        InsufficientBalance,
        /// The STO has already started.
        StoAlreadyStarted,
        /// The sender is not authorized to perform the given operation.
        Unauthorized,
        /// The STO has already reached the given state.
        AlreadyInThatState,
        /// An overflow in the new token count.
        StoCountOverflow,
        /// An overflow in the new token count.
        TokenCountOverflow,
        /// An underflow in the new token count.
        TokenCountUnderflow,
        /// The given token is not a permitted investment currency.
        TokenIsNotPermitted,
        /// An overflow while calculating sold tokens.
        SoldTokensOverflow,
        /// An overflow while updating the simple token investment.
        InvestmentOverflow,
        /// An invalid STO ID.
        InvalidStoId,
        /// The STO has already been paused.
        AlreadyPaused,
        /// The STO is already active.
        AlreadyActive,
        /// The STO is paused.
        Paused,
        /// The STO has not started or has already ended.
        NotStartedOrAlreadyEnded,
        /// Division failed.
        DivisionFailed,
    }
}

type Identity<T> = identity::Module<T>;

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
        /// * `ticker` Ticker of the token
        /// * `beneficiary_did` DID which holds all the funds collected
        /// * `cap` Total amount of tokens allowed for sale
        /// * `rate` Rate of asset in terms of native currency
        /// * `start_date` Unix timestamp at when STO starts
        /// * `end_date` Unix timestamp at when STO ends
        /// * `simple_token_ticker` Ticker of the simple token
        #[weight = SimpleDispatchInfo::FixedNormal(300_000)]
        pub fn launch_sto(
            origin,
            ticker: Ticker,
            beneficiary_did: IdentityId,
            cap: T::Balance,
            rate: u128,
            start_date: T::Moment,
            end_date: T::Moment,
            simple_token_ticker: Ticker
        ) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );

            let sold:T::Balance = 0.into();
            ensure!(Self::is_owner(&ticker, did), Error::<T>::NotAnOwner);

            let sto = STO {
                beneficiary_did,
                cap,
                sold,
                rate,
                start_date,
                end_date,
                active: true
            };

            let sto_count = Self::sto_count(ticker);
            let new_sto_count = sto_count
                .checked_add(1)
                .ok_or(Error::<T>::StoCountOverflow)?;

            let token_count = Self::tokens_count_for_sto((ticker, sto_count));
            let new_token_count = token_count.checked_add(1).ok_or(Error::<T>::TokenCountOverflow)?;

            <StosByToken<T>>::insert((ticker, sto_count), sto);
            <StoCount>::insert(ticker, new_sto_count);

            if simple_token_ticker.len() > 0 {
                // Addition of the SimpleToken token as the fund raised type.
                <TokenIndexForSTO>::insert((ticker, sto_count, simple_token_ticker), new_token_count);
                <AllowedTokens>::insert((ticker, sto_count, new_token_count), simple_token_ticker);
                <TokensCountForSto>::insert((ticker, sto_count), new_token_count);

                Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, simple_token_ticker, sto_count, true));
            }
            sp_runtime::print("Capped STO launched!!!");

            Ok(())
        }

        /// Used to buy tokens
        ///
        /// # Arguments
        /// * `origin` Signing key of the investor
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in
        /// * `value` Amount of POLYX wants to invest in
        #[weight = SimpleDispatchInfo::FixedNormal(500_000)]
        pub fn buy_tokens(origin, ticker: Ticker, sto_id: u32, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender_signer = Signatory::AccountKey(sender_key);


            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender_signer),
                Error::<T>::SenderMustBeSigningKeyForDid
            );

            let mut selected_sto = Self::stos_by_token((ticker, sto_id));
            // Pre validation checks
            ensure!(
                Self::_pre_validation(&ticker, did, selected_sto.clone()).is_ok(),
                Error::<T>::PrevalidationFailed
            );
            // Make sure sender has enough balance
            let sender_balance = <balances::Module<T> as Currency<_>>::free_balance(&sender);
            ensure!(sender_balance >= value, Error::<T>::InsufficientBalance);

            // Get the invested amount of investment currency and amount of ST tokens minted as a return of investment
            let token_amount_value = Self::_get_invested_amount_and_tokens(
                value,
                selected_sto.clone()
            )?;
            let _allowed_value = token_amount_value.1;

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_amount_value.0)
                .ok_or(Error::<T>::SoldTokensOverflow)?;

            // Mint tokens and update STO
            T::Asset::_mint_from_sto(&ticker, sender, did, token_amount_value.0)?;

            // Transfer POLYX to token owner
            // TODO: transfer between DIDs
            //<balances::Module<T> as Currency<_>>::transfer(
                //&sender,
                //&selected_sto.beneficiary_did,
                //allowed_value
                //)?;

            // Update storage values
            Self::_update_storage(
                ticker,
                sto_id,
                did,
                token_amount_value.1,
                token_amount_value.0,
                Ticker::default(),
                0.into(),
                selected_sto
            )?;

            Ok(())
        }


        /// Modify the list of allowed tokens (stable coins) corresponds to given token/asset
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in.
        /// * `simple_token_ticker` Ticker of the stable coin
        /// * `modify_status` Boolean to know whether the provided simple token ticker will be used or not.
        #[weight = SimpleDispatchInfo::FixedNormal(400_000)]
        pub fn modify_allowed_tokens(origin, ticker: Ticker, sto_id: u32, simple_token_ticker: Ticker, modify_status: bool) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            let selected_sto = Self::stos_by_token((ticker, sto_id));
            let now = <pallet_timestamp::Module<T>>::get();
            // Right now we are only allowing the issuer to change the configuration only before the STO start not after the start
            // or STO should be in non-active stage
            ensure!(
                now < selected_sto.start_date || !selected_sto.active,
                Error::<T>::StoAlreadyStarted
            );
            ensure!(Self::is_owner(&ticker, did), Error::<T>::Unauthorized);

            let token_index = Self::token_index_for_sto((ticker, sto_id, simple_token_ticker));
            let token_count = Self::tokens_count_for_sto((ticker, sto_id));

            let current_status = match token_index == None {
                true => false,
                false => true,
            };

            ensure!(current_status != modify_status, Error::<T>::AlreadyInThatState);

            if modify_status {
                let new_count = token_count.checked_add(1).ok_or(Error::<T>::TokenCountOverflow)?;
                <TokenIndexForSTO>::insert((ticker, sto_id, simple_token_ticker), new_count);
                <AllowedTokens>::insert((ticker, sto_id, new_count), simple_token_ticker);
                <TokensCountForSto>::insert((ticker, sto_id), new_count);
            } else {
                let new_count = token_count.checked_sub(1).ok_or(Error::<T>::TokenCountUnderflow)?;
                <TokenIndexForSTO>::insert((ticker, sto_id, simple_token_ticker), new_count);
                <AllowedTokens>::insert((ticker, sto_id, new_count), Ticker::default());
                <TokensCountForSto>::insert((ticker, sto_id), new_count);
            }

            Self::deposit_event(RawEvent::ModifyAllowedTokens(ticker, simple_token_ticker, sto_id, modify_status));

            Ok(())

        }

        /// Used to buy tokens using stable coins
        ///
        /// # Arguments
        /// * `origin` Signing key of the investor
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in
        /// * `value` Amount of POLYX wants to invest in
        /// * `simple_token_ticker` Ticker of the simple token
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn buy_tokens_by_simple_token(origin, ticker: Ticker, sto_id: u32, value: T::Balance, simple_token_ticker: Ticker) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let sender_key = AccountKey::try_from(sender.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let spender = Signatory::AccountKey(sender_key);

            // Check that spender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &spender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check whether given token is allowed as investment currency or not
            ensure!(
                Self::token_index_for_sto((ticker, sto_id, simple_token_ticker)) != None,
                Error::<T>::TokenIsNotPermitted
            );
            let mut selected_sto = Self::stos_by_token((ticker, sto_id));
            // Pre validation checks
            ensure!(
                Self::_pre_validation(&ticker, did, selected_sto.clone()).is_ok(),
                Error::<T>::PrevalidationFailed
            );
            // Make sure spender has enough balance
            ensure!(
                T::SimpleTokenTrait::balance_of(simple_token_ticker, did) >= value,
                Error::<T>::InsufficientBalance
            );

            // Get the invested amount of investment currency and amount of ST tokens minted as a return of investment
            let token_amount_value = Self::_get_invested_amount_and_tokens(
                value,
                selected_sto.clone()
            )?;

            selected_sto.sold = selected_sto.sold
                .checked_add(&token_amount_value.0)
                .ok_or(Error::<T>::SoldTokensOverflow)?;

            let simple_token_investment =
                Self::simple_token_token_spent((ticker, simple_token_ticker, sto_id, did))
                .checked_add(&token_amount_value.1)
                .ok_or(Error::<T>::InvestmentOverflow)?;

            // Mint tokens and update STO
            let _minted_tokes = T::Asset::_mint_from_sto(&ticker, sender, did, token_amount_value.0);
            // Transfer the simple_token invested token to beneficiary account
            T::SimpleTokenTrait::transfer(did, &simple_token_ticker, selected_sto.beneficiary_did, token_amount_value.1)?;

            // Update storage values
            Self::_update_storage(
                ticker,
                sto_id,
                did,
                token_amount_value.1,
                token_amount_value.0,
                simple_token_ticker,
                simple_token_investment,
                selected_sto
            )?;
            Ok(())
        }

        /// Pause the STO, Can only be called by the token owner
        /// By doing this every operations on given sto_id would get freezed like buy_tokens
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to paused
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn pause_sto(origin, ticker: Ticker, sto_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check valid STO id
            ensure!(Self::sto_count(ticker) >= sto_id, Error::<T>::InvalidStoId);
            // Access the STO data
            let mut selected_sto = Self::stos_by_token((ticker, sto_id));
            // Check the flag
            ensure!(selected_sto.active, Error::<T>::AlreadyPaused);
            // Change the flag
            selected_sto.active = false;
            // Update the storage
            <StosByToken<T>>::insert((ticker, sto_id), selected_sto);
            Ok(())
        }

        /// Un-pause the STO, Can only be called by the token owner
        /// By doing this every operations on given sto_id would get un freezed.
        ///
        /// # Arguments
        /// * `origin` Signing key of the token owner
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to un paused
        #[weight = SimpleDispatchInfo::FixedNormal(150_000)]
        pub fn unpause_sto(origin, ticker: Ticker, sto_id: u32) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;
            let sender = Signatory::AccountKey(sender_key);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSigningKeyForDid
            );
            // Check valid STO id
            ensure!(Self::sto_count(ticker) >= sto_id, Error::<T>::InvalidStoId);
            // Access the STO data
            let mut selected_sto = Self::stos_by_token((ticker, sto_id));
            // Check the flag
            ensure!(!selected_sto.active, Error::<T>::AlreadyActive);
            // Change the flag
            selected_sto.active = true;
            // Update the storage
            <StosByToken<T>>::insert((ticker, sto_id), selected_sto);
            Ok(())
        }

    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
    {
        ModifyAllowedTokens(Ticker, Ticker, u32, bool),
        /// Emit when Asset get purchased by the investor
        /// caller DID/investor DID, Ticker, SimpleToken token, sto_id, amount invested, amount of token purchased
        AssetPurchased(IdentityId, Ticker, Ticker, u32, Balance, Balance),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, did)
    }

    fn _pre_validation(
        _ticker: &Ticker,
        _did: IdentityId,
        selected_sto: STO<T::Balance, T::Moment>,
    ) -> DispatchResult {
        // TODO: Validate that buyer is exempted for primary issuance.
        // Check whether the sto is un-paused or not
        ensure!(selected_sto.active, Error::<T>::Paused);
        // Check whether the sto is already ended
        let now = <pallet_timestamp::Module<T>>::get();
        ensure!(
            now >= selected_sto.start_date && now <= selected_sto.end_date,
            Error::<T>::NotStartedOrAlreadyEnded
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
            .ok_or(Error::<T>::InvestmentOverflow)?;
        let allowed_token_sold = selected_sto
            .cap
            .checked_sub(&selected_sto.sold)
            .ok_or(Error::<T>::SoldTokensOverflow)?;
        let mut allowed_value = invested_amount;
        // Make sure there's still an allocation
        // Instead of reverting, buy up to the max and refund excess amount of investment currency.
        if token_conversion > allowed_token_sold {
            token_conversion = allowed_token_sold;
            allowed_value = token_conversion
                .checked_div(&selected_sto.rate.into())
                .ok_or(Error::<T>::DivisionFailed)?;
        }
        Ok((token_conversion, allowed_value))
    }

    fn _update_storage(
        ticker: Ticker,
        sto_id: u32,
        did: IdentityId,
        investment_amount: T::Balance,
        new_tokens_minted: T::Balance,
        simple_token_ticker: Ticker,
        simple_token_investment: T::Balance,
        selected_sto: STO<T::Balance, T::Moment>,
    ) -> DispatchResult {
        // Store Investment DATA
        let mut investor_holder = Self::investment_data((ticker, sto_id, did));
        if investor_holder.investor_did == IdentityId::default() {
            investor_holder.investor_did = did;
        }
        investor_holder.assets_purchased = investor_holder
            .assets_purchased
            .checked_add(&new_tokens_minted)
            .ok_or(Error::<T>::InvestmentOverflow)?;
        investor_holder.last_purchase_date = <pallet_timestamp::Module<T>>::get();

        if simple_token_ticker != Ticker::default() {
            <SimpleTokenSpent<T>>::insert(
                (ticker, simple_token_ticker, sto_id, did),
                simple_token_investment,
            );
        } else {
            investor_holder.amount_paid = investor_holder
                .amount_paid
                .checked_add(&investment_amount)
                .ok_or(Error::<T>::InvestmentOverflow)?;
        }
        <StosByToken<T>>::insert((ticker, sto_id), selected_sto);
        // Emit Event
        Self::deposit_event(RawEvent::AssetPurchased(
            did,
            ticker,
            simple_token_ticker,
            sto_id,
            investment_amount,
            new_tokens_minted,
        ));
        sp_runtime::print("Invested in STO");
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
     *    use sp_io::with_externalities;
     *    use sp_runtime::{
     *        testing::{Digest, DigestItem, Header},
     *        traits::{BlakeTwo256, IdentityLookup},
     *        BuildStorage,
     *    };
     *    use frame_support::{assert_ok, impl_outer_origin};
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
     *    impl frame_system::Trait for Test {
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
     *    fn new_test_ext() -> sp_io::TestExternalities<Blake2Hasher> {
     *        frame_system::GenesisConfig::default()
     *            .build_storage()
     *            .unwrap()
     *            .0
     *            .into()
     *    }
     */
}
