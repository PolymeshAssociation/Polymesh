// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

//! # Polymesh Transaction Payment Module
//!
//! This module provides the TransactionExtension implementation to charge transaction fees
//! in Polymesh.
//! It extends the default functionality provided by `pallet_transaction_payment` to allow
//! subsidised transactions and to enforce tipping rules for Governance Committee and CDD
//! Providers members.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, DecodeWithMemTracking, Encode};
use frame_support::dispatch::PostDispatchInfo;
use frame_support::dispatch::{DispatchClass, DispatchInfo, DispatchResult};
use frame_support::traits::{Imbalance, SuppressedDrop};
use frame_support::weights::Weight;
use frame_support::DebugNoBound;
use frame_support::{pallet_prelude::*, DefaultNoBound};
use frame_system::pallet_prelude::OriginFor;
use scale_info::TypeInfo;
use sp_runtime::traits::{DispatchInfoOf, Dispatchable, PostDispatchInfoOf};
use sp_runtime::traits::{Saturating, TransactionExtension, Zero};
use sp_runtime::transaction_validity::{TransactionValidityError, ValidTransaction};

use polymesh_primitives::traits::group::GroupTrait;
use polymesh_primitives::traits::{CurrentFeePayer, IdentityFnTrait, SubsidiserTrait};
use polymesh_primitives::transaction_payment::CallPaymentInfo;
use polymesh_primitives::TransactionError;

pub use pallet::*;

pub use pallet_transaction_payment::{
    ChargeFeesControl, FeeDetails, InclusionFee, OnChargeTransaction, RuntimeDispatchInfo,
    TxCreditHold, WeightInfo,
};

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

pub type TransactionPallet<T> = pallet_transaction_payment::Pallet<T>;

pub(crate) type BalanceOf<T> = <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::Balance;

/// The credit held by `pallet_transaction_payment` for the current transaction.
///
/// Both the withdrawn transaction fee and the pre-charged (ETH) storage deposit are pooled
/// into this credit, and `pallet_revive` draws the storage deposit it actually uses from it.
pub(crate) type CreditOf<T> = <<<T as pallet_transaction_payment::Config>::OnChargeTransaction as TxCreditHold<T>>::Credit as SuppressedDrop>::Inner;

impl<T: Config> ChargeFeesControl for Pallet<T> {
    fn disabled() -> bool {
        #[cfg(feature = "disable_fees")]
        {
            DisableFees::<T>::get()
        }
        #[cfg(not(feature = "disable_fees"))]
        {
            false
        }
    }
}

#[frame_support::pallet]
pub mod pallet {
    use super::*;

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    #[pallet::config]
    pub trait Config:
        frame_system::Config + pallet_transaction_payment::Config + pallet_timestamp::Config
    {
        /// Fetch the signatory to charge fee from. Also sets fee payer and identity in context.
        type TxFeeHandler: CurrentFeePayer<Self::AccountId, Self::RuntimeCall>;

        /// Used to charge transaction fees to a subsidiser, instead of the payer.
        type Subsidiser: SubsidiserTrait<Self::AccountId, Self::RuntimeCall>;

        type DidRegistrars: GroupTrait<Self::Moment>;

        type GovernanceCommittee: GroupTrait<Self::Moment>;

        type Identity: IdentityFnTrait<Self::AccountId>;
    }

    #[cfg(feature = "disable_fees")]
    #[pallet::storage]
    pub type DisableFees<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// It stores the current gas fee payer for the current transaction.
    #[pallet::storage]
    pub type CurrentPayer<T: Config> = StorageValue<_, T::AccountId, OptionQuery>;

    #[pallet::genesis_config]
    pub struct GenesisConfig<T: Config> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
        #[cfg(feature = "disable_fees")]
        pub disable_fees: bool,
    }

    impl<T: Config> Default for GenesisConfig<T> {
        fn default() -> Self {
            Self {
                _config: Default::default(),
                #[cfg(feature = "disable_fees")]
                disable_fees: false,
            }
        }
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            #[cfg(feature = "disable_fees")]
            DisableFees::<T>::put(self.disable_fees);
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {
        #[pallet::call_index(0)]
        #[pallet::weight(Weight::from_parts(25_000_000, 0))]
        pub fn set_disable_fees(origin: OriginFor<T>, _value: bool) -> DispatchResult {
            frame_system::ensure_root(origin)?;
            #[cfg(feature = "disable_fees")]
            DisableFees::<T>::put(_value);
            Ok(())
        }
    }
}

impl<T: Config> Pallet<T> {
    /// Fetches the fee payer from the context.
    pub fn current_payer() -> Option<T::AccountId> {
        CurrentPayer::<T>::get()
    }

    /// Sets the fee payer in the context.
    pub fn set_current_payer(payer: Option<T::AccountId>) {
        if let Some(payer) = payer {
            CurrentPayer::<T>::put(payer);
        } else {
            // Clear the current payer
            CurrentPayer::<T>::kill();
        }
    }
}

/// Require the transactor pay for themselves and maybe include a tip to gain additional priority
/// in the queue.
///
/// # Transaction Validity
///
/// This extension sets the `priority` field of `TransactionValidity` depending on the amount
/// of tip being paid per weight unit.
///
/// Operational transactions will receive an additional priority bump, so that they are normally
/// considered before regular transactions.
#[derive(
    Encode,
    Decode,
    DecodeWithMemTracking,
    DefaultNoBound,
    Clone,
    Eq,
    PartialEq,
    TypeInfo
)]
#[scale_info(skip_type_params(T))]
pub struct ChargeTransactionPayment<T: Config> {
    #[codec(compact)]
    tip: BalanceOf<T>,
    #[codec(skip)]
    storage_deposit: BalanceOf<T>,
}

impl<T: Config> ChargeTransactionPayment<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync + Into<u128>,
{
    /// utility constructor. Used only in client/factory code.
    pub fn from(tip: BalanceOf<T>) -> Self {
        Self {
            tip,
            ..Default::default()
        }
    }

    /// Returns the tip as being chosen by the transaction sender.
    pub fn tip(&self) -> BalanceOf<T> {
        self.tip
    }

    pub fn set_storage_deposit(&mut self, storage_deposit: BalanceOf<T>) {
        self.storage_deposit = storage_deposit;
    }

    /// Total amount a subsidiser has to cover: the transaction fee plus the storage
    /// deposit already pre-charged for an Ethereum transaction.
    fn total_subsidised(&self, fee_with_tip: BalanceOf<T>) -> BalanceOf<T> {
        fee_with_tip.saturating_add(self.storage_deposit)
    }

    pub(crate) fn can_withdraw_fee(
        &self,
        who: T::AccountId,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
    ) -> Result<(BalanceOf<T>, Option<T::AccountId>), TransactionValidityError> {
        let tip = self.tip;
        let fee_with_tip = TransactionPallet::<T>::compute_fee(len as u32, info, tip);

        // Polymesh change
        // -----------------------------------------------------------------

        if fee_with_tip.is_zero() {
            return Ok((fee_with_tip, None));
        }

        let (call_payment_info, subsidiser) =
            Self::check_subsidy_conditions(&who, call, self.total_subsidised(fee_with_tip))?;

        // key to pay the fee.
        let fee_key = subsidiser
            .as_ref()
            .unwrap_or(call_payment_info.paying_account());

        <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<
            T,
        >>::can_withdraw_fee(fee_key, call, info, fee_with_tip, tip)?;

        T::TxFeeHandler::set_payer_context(Some(call_payment_info.paying_account().clone()));

        // -----------------------------------------------------------------

        Ok((fee_with_tip, subsidiser))
    }

    pub(crate) fn withdraw_fee(
        &self,
        who: &T::AccountId,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        fee_with_tip: BalanceOf<T>,
    ) -> Result<
        (
            BalanceOf<T>,
            Option<T::AccountId>,
            <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
            CallPaymentInfo<T::AccountId>,
        ),
        TransactionValidityError,
    >{
        let tip = self.tip;

        // Polymesh change
        // -----------------------------------------------------------------

        if fee_with_tip.is_zero() {
            let call_payment_info = CallPaymentInfo::new(who.clone(), None, None);
            return Ok((fee_with_tip, None, Default::default(), call_payment_info));
        }

        let (call_payment_info, subsidiser) =
            Self::check_subsidy_conditions(who, call, self.total_subsidised(fee_with_tip))?;

        // key to pay the fee.
        let fee_key = subsidiser
            .as_ref()
            .unwrap_or(call_payment_info.paying_account());

        let liq_info =
            <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::withdraw_fee(
                fee_key,
                call,
                info,
                fee_with_tip,
                tip,
            )?;

        T::TxFeeHandler::set_payer_context(Some(call_payment_info.paying_account().clone()));

        // -----------------------------------------------------------------

        Ok((fee_with_tip, subsidiser, liq_info, call_payment_info))
    }

    pub fn check_subsidy_conditions(
        who: &T::AccountId,
        call: &T::RuntimeCall,
        fee_with_tip: BalanceOf<T>,
    ) -> Result<(CallPaymentInfo<T::AccountId>, Option<T::AccountId>), InvalidTransaction> {
        // Get the payment info for the call
        let call_payment_info = T::TxFeeHandler::call_payment_info(call, who.clone())?;

        // Check if the payer is being subsidised.
        let subsidiser = T::Subsidiser::check_subsidy(
            call_payment_info.paying_account(),
            fee_with_tip.into(),
            Some(call),
        )?;

        Ok((call_payment_info, subsidiser))
    }

    // Polymesh change: Used to allow GC/DID registrar member to include a `tip`.
    // -----------------------------------------------------------------

    /// Returns `true` if `who` is member of `T::GovernanceCommittee` or `T::DidRegistrars`.
    fn is_gc_or_registrar_member(who: &T::AccountId) -> bool {
        T::Identity::get_identity(who)
            .map(|did| T::GovernanceCommittee::is_member(&did) || T::DidRegistrars::is_member(&did))
            .unwrap_or(false)
    }

    /// Ensures that the transaction tip is valid.
    ///
    /// Tipping is allowed for `DispatchClass::Operational` created by a Governance or DID registrar member.
    /// Mandatory transactions are going to be included in the block, so adding a tip does not matter.
    pub(crate) fn ensure_valid_tip(
        &self,
        who: &T::AccountId,
        info: &DispatchInfoOf<T::RuntimeCall>,
    ) -> Result<BalanceOf<T>, TransactionValidityError> {
        match info.class {
            DispatchClass::Normal | DispatchClass::Operational => {
                if self.tip.is_zero() {
                    return Ok(self.tip);
                }

                if info.class == DispatchClass::Operational && Self::is_gc_or_registrar_member(who)
                {
                    return Ok(self.tip);
                }

                Err(TransactionValidityError::Invalid(
                    InvalidTransaction::Custom(TransactionError::ZeroTip as u8),
                ))
            }
            DispatchClass::Mandatory => Ok(self.tip),
        }
    }

    // -----------------------------------------------------------------
}

/// The info passed between the validate and prepare steps for the `ChargeAssetTxPayment` extension.
#[derive(DebugNoBound)]
pub enum Val<T: Config> {
    Charge {
        tip: BalanceOf<T>,
        // who called the transaction
        who: T::AccountId,
        // transaction fee
        fee_with_tip: BalanceOf<T>,
        // Polymesh Subsidiser account (who paid the fee)
        subsidiser: Option<T::AccountId>,
    },
    NoCharge,
}

/// The info passed between the prepare and post-dispatch steps for the `ChargeAssetTxPayment`
/// extension.
pub enum Pre<T: Config> {
    Charge {
        tip: BalanceOf<T>,
        // the max fee reserved from the subsidy in prepare
        fee_with_tip: BalanceOf<T>,
        // storage deposit pre-charged for an ETH transaction (zero otherwise)
        storage_deposit: BalanceOf<T>,
        // imbalance resulting from withdrawing the fee
        imbalance: <<T as pallet_transaction_payment::Config>::OnChargeTransaction as OnChargeTransaction<T>>::LiquidityInfo,
        // Polymesh Subsidiser account (who paid the fee)
        subsidiser: Option<T::AccountId>,
        // Call payment info
        call_payment_info: CallPaymentInfo<T::AccountId>,
    },
    NoCharge {
        // weight initially estimated by the extension, to be refunded
        refund: Weight,
    },
}

impl<T: Config> sp_std::fmt::Debug for ChargeTransactionPayment<T> {
    #[cfg(feature = "std")]
    fn fmt(&self, f: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        write!(f, "ChargeTransactionPayment<{:?}>", self.tip)
    }
    #[cfg(not(feature = "std"))]
    fn fmt(&self, _: &mut sp_std::fmt::Formatter) -> sp_std::fmt::Result {
        Ok(())
    }
}

impl<T: Config> TransactionExtension<T::RuntimeCall> for ChargeTransactionPayment<T>
where
    T::RuntimeCall: Dispatchable<Info = DispatchInfo, PostInfo = PostDispatchInfo>,
    BalanceOf<T>: Send + Sync + Into<u128>,
    CreditOf<T>: Imbalance<BalanceOf<T>>,
{
    const IDENTIFIER: &'static str = "ChargeTransactionPayment";
    type Implicit = ();
    type Val = Val<T>;
    type Pre = Pre<T>;

    fn weight(&self, _call: &T::RuntimeCall) -> Weight {
        // TODO: improve weight based on the call.
        <T as pallet_transaction_payment::Config>::WeightInfo::charge_transaction_payment()
    }

    fn validate(
        &self,
        origin: <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        len: usize,
        _: (),
        _implication: &impl Encode,
        _source: TransactionSource,
    ) -> Result<
        (
            ValidTransaction,
            Self::Val,
            <T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        ),
        TransactionValidityError,
    > {
        let Ok(who) = frame_system::ensure_signed(origin.clone()) else {
            return Ok((ValidTransaction::default(), Val::NoCharge, origin));
        };

        let tip = self.ensure_valid_tip(&who, info)?;

        let (fee_with_tip, subsidiser) = self.can_withdraw_fee(who.clone(), call, info, len)?;

        let valid_transaction = ValidTransaction {
            priority: pallet_transaction_payment::ChargeTransactionPayment::<T>::get_priority(
                info,
                len,
                tip,
                fee_with_tip,
            ),
            ..Default::default()
        };

        let val = Val::Charge {
            tip,
            who,
            fee_with_tip,
            subsidiser,
        };

        Ok((valid_transaction, val, origin))
    }

    fn prepare(
        self,
        val: Self::Val,
        _origin: &<T::RuntimeCall as Dispatchable>::RuntimeOrigin,
        call: &T::RuntimeCall,
        info: &DispatchInfoOf<T::RuntimeCall>,
        _len: usize,
    ) -> Result<Self::Pre, TransactionValidityError> {
        match val {
            Val::NoCharge => Ok(Pre::NoCharge {
                refund: self.weight(call),
            }),
            Val::Charge {
                tip,
                who,
                fee_with_tip,
                subsidiser: _,
            } => {
                let storage_deposit = self.storage_deposit;
                let (_, subsidiser, imbalance, call_payment_info) =
                    self.withdraw_fee(&who, call, info, fee_with_tip)?;

                // Reserve the full tx fee and the pre-charged storage deposit from the subsidy
                // budget so that protocol fees charged during dispatch cannot exhaust
                // `remaining` and break post_dispatch.
                if subsidiser.is_some() {
                    T::Subsidiser::reserve_subsidy(
                        call_payment_info.paying_account(),
                        fee_with_tip.saturating_add(storage_deposit).into(),
                    )?;
                }

                Ok(Pre::Charge {
                    tip,
                    fee_with_tip,
                    storage_deposit,
                    imbalance,
                    subsidiser,
                    call_payment_info,
                })
            }
        }
    }

    fn post_dispatch(
        pre: Self::Pre,
        info: &DispatchInfoOf<T::RuntimeCall>,
        post_info: &mut PostDispatchInfoOf<T::RuntimeCall>,
        len: usize,
        result: &DispatchResult,
    ) -> Result<(), TransactionValidityError> {
        let _ = CurrentPayer::<T>::take();

        let (tip, fee_with_tip, storage_deposit, imbalance, subsidiser, call_payment_info) = {
            match pre {
                Pre::Charge {
                    tip,
                    fee_with_tip,
                    storage_deposit,
                    imbalance,
                    subsidiser,
                    call_payment_info,
                } => (
                    tip,
                    fee_with_tip,
                    storage_deposit,
                    imbalance,
                    subsidiser,
                    call_payment_info,
                ),
                Pre::NoCharge { .. } => return Ok(()),
            }
        };

        // We want to decrease the counter of Authorization.count when the caller is not paying for the fee and the call failed
        if result.is_err() {
            T::TxFeeHandler::decrease_authorization_count(&call_payment_info);
        }

        let actual_fee =
            TransactionPallet::<T>::compute_actual_fee(len as u32, info, post_info, tip);

        let fee_key = {
            if let Some(subsidiser_acc) = subsidiser {
                // The payer is pre-charged `fee_with_tip` and `storage_deposit`.
                // The credit pool `remaining_txfee` doesn't include the `tip`.
                let reserved = storage_deposit.saturating_add(fee_with_tip);
                let unspent: BalanceOf<T> = TransactionPallet::<T>::remaining_txfee();
                // To calculate the actual storage used, we need to subtract the tip and the `remaining_txfee` from the pre-charged amount.
                let storage_used = reserved.saturating_sub(tip).saturating_sub(unspent);
                // The actual fee is the sum of the actual transaction fee and the storage used.
                let actual_fee = actual_fee.saturating_add(storage_used);

                // Settle the subsidy: refund (reserved - actual) and emit the debit event.
                T::Subsidiser::settle_subsidy(
                    call_payment_info.paying_account(),
                    &subsidiser_acc,
                    reserved.into(),
                    actual_fee.into(),
                );
                subsidiser_acc
            } else {
                call_payment_info.paying_account().clone()
            }
        };

        T::OnChargeTransaction::correct_and_deposit_fee(
            &fee_key, info, post_info, actual_fee, tip, imbalance,
        )?;

        TransactionPallet::<T>::deposit_fee_paid_event(fee_key, actual_fee, tip);

        Ok(())
    }
}
