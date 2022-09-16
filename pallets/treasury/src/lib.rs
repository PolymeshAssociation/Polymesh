// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
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

//! # Treasury Module
//!
//! Treasury module contains a couple of functions to manage the treasury through the governance
//! module.
//!
//! ## Overview
//!
//! Treasury balance is filled by fees of each operation, but it also accepts donations
//! through [reimbursement](Module::reimbursement) method.
//!
//! The disbursement mechanism is designed to incentivize Polymesh Improvement Proposals.
//!
//! ## Dispatchable Functions
//!
//! - [disbursement](Module::disbursement) - Transfers from the treasury to the given benericiaries.
//! - [reimbursement](Module::reimbursement) - Transfers to the treasury.
//!
#![cfg_attr(not(feature = "std"), no_std)]
#![recursion_limit = "256"]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::traits::{StorageInfo, StorageInfoTrait};
use frame_support::{
    decl_error, decl_event, decl_module,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    traits::{Currency, ExistenceRequirement, Imbalance, OnUnbalanced},
    weights::Weight,
};
use frame_system::ensure_root;
use pallet_identity as identity;
use polymesh_common_utilities::{
    constants::TREASURY_PALLET_ID, traits::balances::Config as BalancesConfig, Context, GC_DID,
};
use polymesh_primitives::{Beneficiary, IdentityId};
use sp_runtime::traits::{AccountIdConversion, Saturating};
use sp_std::prelude::*;

pub type ProposalIndex = u32;

type Identity<T> = identity::Module<T>;
type BalanceOf<T> =
    <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
type NegativeImbalanceOf<T> = <<T as Config>::Currency as Currency<
    <T as frame_system::Config>::AccountId,
>>::NegativeImbalance;

pub trait Config: frame_system::Config + BalancesConfig {
    // The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// The native currency.
    type Currency: Currency<Self::AccountId>;
    /// Weight information for extrinsics in the identity pallet.
    type WeightInfo: WeightInfo;
}

pub trait WeightInfo {
    fn reimbursement() -> Weight;
    fn disbursement(beneficiary_count: u32) -> Weight;
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// Disbursement to a target Identity.
        ///
        /// (treasury identity, target identity, target primary key, amount)
        TreasuryDisbursement(IdentityId, IdentityId, AccountId, Balance),

        /// Disbursement to a target Identity failed.
        ///
        /// (treasury identity, target identity, target primary key, amount)
        TreasuryDisbursementFailed(IdentityId, IdentityId, AccountId, Balance),

        /// Treasury reimbursement.
        ///
        /// (source identity, amount)
        TreasuryReimbursement(IdentityId, Balance),
    }
);

decl_error! {
    /// Error for the treasury module.
    pub enum Error for Module<T: Config> {
        /// Proposer's balance is too low.
        InsufficientBalance,
        /// Invalid identity for disbursement.
        InvalidIdentity,
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// It transfers balances from treasury to each of beneficiaries and the specific amount
        /// for each of them.
        ///
        /// # Error
        /// * `BadOrigin`: Only root can execute transaction.
        /// * `InsufficientBalance`: If treasury balances is not enough to cover all beneficiaries.
        /// * `InvalidIdentity`: If one of the beneficiaries has an invalid identity.
        #[weight = <T as Config>::WeightInfo::disbursement(beneficiaries.len() as u32)]
        pub fn disbursement(origin, beneficiaries: Vec<Beneficiary<BalanceOf<T>>>) {
            Self::base_disbursement(origin, beneficiaries)?;
        }

        /// It transfers the specific `amount` from `origin` account into treasury.
        ///
        /// Only accounts which are associated to an identity can make a donation to treasury.
        #[weight = <T as Config>::WeightInfo::reimbursement()]
        pub fn reimbursement(origin, amount: BalanceOf<T>) {
            Self::base_reimbursement(origin, amount)?;
        }
    }
}

impl<T: Config> Module<T> {
    fn base_disbursement(
        origin: T::Origin,
        beneficiaries: Vec<Beneficiary<BalanceOf<T>>>,
    ) -> DispatchResult {
        ensure_root(origin)?;

        // Get the primary key for each Beneficiary.
        let mut total_amount: BalanceOf<T> = 0u32.into();
        let beneficiaries = beneficiaries
            .iter()
            .map(|b| -> Result<_, DispatchError> {
                total_amount = total_amount.saturating_add(b.amount);
                // Ensure the identity exists and get its primary key.
                let primary_key =
                    Identity::<T>::get_primary_key(b.id).ok_or(Error::<T>::InvalidIdentity)?;
                Ok((primary_key, b.id, b.amount))
            })
            .collect::<Result<Vec<_>, DispatchError>>()?;

        // Ensure treasury has enough balance.
        ensure!(
            Self::balance() >= total_amount,
            Error::<T>::InsufficientBalance
        );

        // Do disbursement.
        for (primary_key, id, amount) in beneficiaries {
            Self::unsafe_disbursement(primary_key, id, amount);
        }

        Ok(())
    }

    fn base_reimbursement(origin: T::Origin, amount: BalanceOf<T>) -> DispatchResult {
        let identity::PermissionedCallOriginData {
            sender,
            primary_did,
            ..
        } = Identity::<T>::ensure_origin_call_permissions(origin)?;

        // Not checking the cdd for the treasury account as it is assumed
        // that treasury account posses a valid CDD check during the genesis phase
        T::Currency::transfer(
            &sender,
            &Self::account_id(),
            amount,
            ExistenceRequirement::AllowDeath,
        )?;

        Self::deposit_event(RawEvent::TreasuryReimbursement(primary_did, amount));

        Ok(())
    }

    /// The account ID of the treasury pot.
    ///
    /// This actually does computation. If you need to keep using it, then make sure you cache the
    /// value and only call this once.
    fn account_id() -> T::AccountId {
        TREASURY_PALLET_ID.into_account()
    }

    fn unsafe_disbursement(primary_key: T::AccountId, target: IdentityId, amount: BalanceOf<T>) {
        // The transfer failure cases are:
        // 1. `target` not having a valid CDD.
        // 2. The Treasury not having enough POLYX.  This shouldn't happen here,
        //   since the balance is check before the disbursement.
        // 3. `primary_key` balance overflow.
        // 4. The Treasury's balance is frozen (staking).
        let res = T::Currency::transfer(
            &Self::account_id(),
            &primary_key,
            amount,
            ExistenceRequirement::AllowDeath,
        );

        // Emit event based on transfer results.
        let event = if res.is_ok() {
            RawEvent::TreasuryDisbursement
        } else {
            RawEvent::TreasuryDisbursementFailed
        };
        Self::deposit_event(event(GC_DID, target, primary_key, amount));
    }

    /// Returns the current balance of the treasury.
    pub fn balance() -> BalanceOf<T> {
        T::Currency::free_balance(&Self::account_id())
    }
}

/// That trait implementation is needed to receive a portion of the fees from transactions.
impl<T: Config> OnUnbalanced<NegativeImbalanceOf<T>> for Module<T> {
    fn on_nonzero_unbalanced(amount: NegativeImbalanceOf<T>) {
        let numeric_amount = amount.peek();

        let _ = T::Currency::resolve_creating(&Self::account_id(), amount);
        let current_did = Context::current_identity::<Identity<T>>().unwrap_or_default();
        Self::deposit_event(RawEvent::TreasuryReimbursement(current_did, numeric_amount));
    }
}

impl<T: Config> StorageInfoTrait for Module<T> {
    fn storage_info() -> Vec<StorageInfo> {
        Vec::new()
    }
}
