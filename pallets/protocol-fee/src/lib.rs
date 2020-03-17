//! # Protocol Fee Module

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::DispatchResult,
    traits::{Currency, ExistenceRequirement, OnUnbalanced, WithdrawReason},
};
use frame_system::ensure_root;
use primitives::{traits::IdentityCurrency, Signatory};
use sp_runtime::traits::{CheckedDiv, Saturating};

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// A wrapper for a dispatchable function name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct ExtrinsicName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for ExtrinsicName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        ExtrinsicName(v)
    }
}

pub trait Trait: frame_system::Trait {
    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId> + Send + Sync + IdentityCurrency<Self::AccountId>;
    /// Handler for the unbalanced reduction when taking protocol fees.
    type OnProtocolFeePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Insufficient balance to pay the fee.
        InsufficientBalance,
        /// Account ID decoding failed.
        AccountIdDecode,
        /// Division in `compute_fee` failed.
        ComputeFee,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Balances {
        /// The mapping of extrinsic names to the base fees of those extrinsics.
        BaseFees get(base_fees) config(): map ExtrinsicName => BalanceOf<T>;
        /// The fee multiplier as a positive rational (numerator, denominator).
        Multiplier get(multiplier) config() build(|config: &GenesisConfig<T>| {
            if config.multiplier.1 == 0 {
                (1, 1)
            } else {
                config.multiplier
            }
        }): (u32, u32);
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        /// Changes the fee multiplier for the root origin.
        pub fn change_multiplier(origin, multiplier: (u32, u32)) -> DispatchResult {
            ensure_root(origin)?;
            <Multiplier>::put(multiplier);
            Ok(())
        }

        /// Changes the a base fee for the root origin.
        pub fn change_base_fee(origin, name: ExtrinsicName, base_fee: BalanceOf<T>) ->
            DispatchResult
        {
            ensure_root(origin)?;
            <BaseFees<T>>::insert(name, base_fee);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    /// Computes the fee of the extrinsic.
    pub fn compute_fee(name: ExtrinsicName) -> Option<BalanceOf<T>> {
        let (numerator, denominator) = Self::multiplier();
        Self::base_fees(name)
            .saturating_mul(<BalanceOf<T>>::from(numerator))
            .checked_div(&<BalanceOf<T>>::from(denominator))
    }

    /// Computes the fee of the extrinsic and charges it to the given signatory.
    pub fn charge_fee(signatory: Signatory, name: ExtrinsicName) -> DispatchResult {
        if let Some(fee) = Self::compute_fee(name) {
            let imbalance = match signatory {
                Signatory::Identity(did) => T::Currency::withdraw_identity_balance(&did, fee)
                    .map_err(|_| Error::<T>::InsufficientBalance),
                Signatory::AccountKey(account) => T::Currency::withdraw(
                    &T::AccountId::decode(&mut &account.encode()[..])
                        .map_err(|_| Error::<T>::AccountIdDecode)?,
                    fee,
                    WithdrawReason::Fee.into(),
                    ExistenceRequirement::KeepAlive,
                )
                    .map_err(|_| Error::<T>::InsufficientBalance),
            }?;
            T::OnProtocolFeePayment::on_unbalanced(imbalance);
            Ok(())
        } else {
            Err(Error::<T>::ComputeFee.into())
        }
    }
}
