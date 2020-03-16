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
use sp_runtime::{
    traits::{Convert, Saturating, Zero},
    Fixed64,
};

type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;
type NegativeImbalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::NegativeImbalance;

/// A wrapper for a dispatchable function name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct DispatchableName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for DispatchableName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        DispatchableName(v)
    }
}

pub trait Trait: frame_system::Trait {
    /// The currency type in which fees will be paid.
    type Currency: Currency<Self::AccountId> + Send + Sync + IdentityCurrency<Self::AccountId>;
    /// Handler for the unbalanced reduction when taking protocol fees.
    type OnProtocolFeePayment: OnUnbalanced<NegativeImbalanceOf<Self>>;
    /// Convert a multiplier ratio to the type of fee.
    type MultiplierToFee: Convert<Fixed64, BalanceOf<Self>>;
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        InsufficientBalance,
        AccountIdDecode,
    }
}

decl_storage! {
    trait Store for Module<T: Trait> as Balances {
        BaseFees get(base_fees) config(): map DispatchableName => BalanceOf<T>;
        Multiplier get(multiplier) build(|config: &GenesisConfig<T>| {
            if config.multiplier_numerator > i64::MAX as u64 {
                panic!("The numerator of the fee multiplier should fit be a positive i64");
            }
            if config.multiplier_denominator.is_zero() {
                Fixed64::from_natural(1)
            } else {
                Fixed64::from_rational(
                    config.multiplier_numerator as i64,
                    config.multiplier_denominator
                )
            }
        }): Fixed64;
    }
    add_extra_genesis {
        config(multiplier_numerator): u64;
        config(multiplier_denominator): u64;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        pub fn change_multiplier(origin, multiplier: Fixed64) -> DispatchResult {
            ensure_root(origin)?;
            <Multiplier>::put(multiplier);
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {
    pub fn compute_fee(name: DispatchableName) -> BalanceOf<T> {
        let multiplier = T::MultiplierToFee::convert(Self::multiplier());
        Self::base_fees(name).saturating_mul(multiplier)
    }

    pub fn charge_fee(signatory: Signatory, name: DispatchableName) -> DispatchResult {
        let fee = Self::compute_fee(name);
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
    }
}
