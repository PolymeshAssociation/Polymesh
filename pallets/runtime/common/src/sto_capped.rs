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

use frame_support::traits::Currency;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::ensure_signed;
use pallet_balances as balances;
use pallet_compliance_manager as compliance_manager;
use pallet_identity as identity;
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, balances::Trait as BalancesTrait, CommonTrait, Context,
};
use polymesh_primitives::{IdentityId, Signatory, Ticker};
use sp_runtime::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use sp_std::prelude::*;

/// The module's configuration trait.
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + BalancesTrait + compliance_manager::Trait
{
    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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
        /// To track the investment data of the investor corresponds to ticker
        /// (asset_ticker, sto_id, DID) -> Investment structure
        InvestmentData get(fn investment_data): map hasher(blake2_128_concat) (Ticker, u32, IdentityId) => Investment<T::Balance, T::Moment>;
        /// To track the investment amount of the investor corresponds to ticker
        /// (asset_ticker, sto_id, accountId) -> Invested balance
        SimpleTokenSpent get(fn simple_token_token_spent): map hasher(blake2_128_concat) (Ticker, u32, IdentityId) => T::Balance;
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// The sender must be a secondary key for the DID.
        SenderMustBeSecondaryKeyForDid,
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
        /// * `origin` Secondary key of the token owner who wants to initialize the sto
        /// * `ticker` Ticker of the token
        /// * `beneficiary_did` DID which holds all the funds collected
        /// * `cap` Total amount of tokens allowed for sale
        /// * `rate` Rate of asset in terms of native currency
        /// * `start_date` Unix timestamp at when STO starts
        /// * `end_date` Unix timestamp at when STO ends
        #[weight = 70_000_000_000]
        pub fn launch_sto(
            origin,
            ticker: Ticker,
            beneficiary_did: IdentityId,
            cap: T::Balance,
            rate: u128,
            start_date: T::Moment,
            end_date: T::Moment,
        ) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
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

            <StosByToken<T>>::insert((ticker, sto_count), sto);
            <StoCount>::insert(ticker, new_sto_count);

            sp_runtime::print("Capped STO launched!!!");

            Ok(())
        }

        /// Used to buy tokens
        ///
        /// # Arguments
        /// * `origin` Secondary key of the investor
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO investor wants to invest in
        /// * `value` Amount of POLYX wants to invest in
        #[weight = 1_000_000_000]
        pub fn buy_tokens(origin, ticker: Ticker, sto_id: u32, value: T::Balance) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender_signer = Signatory::Account(sender.clone());

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender_signer),
                Error::<T>::SenderMustBeSecondaryKeyForDid
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
                selected_sto
            )?;

            Ok(())
        }

        /// Pause the STO, Can only be called by the token owner
        /// By doing this every operations on given sto_id would get freezed like buy_tokens
        ///
        /// # Arguments
        /// * `origin` Secondary key of the token owner
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to paused
        #[weight = 500_000_000]
        pub fn pause_sto(origin, ticker: Ticker, sto_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
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
        /// * `origin` Secondary key of the token owner
        /// * `ticker` Ticker of the token
        /// * `sto_id` A unique identifier to know which STO needs to un paused
        #[weight = 500_000_000]
        pub fn unpause_sto(origin, ticker: Ticker, sto_id: u32) -> DispatchResult {
            let sender = ensure_signed(origin)?;
            let did = Context::current_identity_or::<Identity<T>>(&sender)?;
            let sender = Signatory::Account(sender);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(
                <identity::Module<T>>::is_signer_authorized(did, &sender),
                Error::<T>::SenderMustBeSecondaryKeyForDid
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
        /// Emit when Asset get purchased by the investor
        /// caller DID/investor DID, Ticker, sto_id, amount invested, amount of token purchased
        AssetPurchased(IdentityId, Ticker, u32, Balance, Balance),
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

        investor_holder.amount_paid = investor_holder
            .amount_paid
            .checked_add(&investment_amount)
            .ok_or(Error::<T>::InvestmentOverflow)?;
        <StosByToken<T>>::insert((ticker, sto_id), selected_sto);
        // Emit Event
        Self::deposit_event(RawEvent::AssetPurchased(
            did,
            ticker,
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
