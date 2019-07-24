//! A runtime module providing a unique ticker registry
use crate::{asset, erc20};
use parity_codec::{Decode, Encode};
use rstd::prelude::*;
use crate::utils;
use system::{self, ensure_signed};
use runtime_primitives::traits::{As, CheckedAdd, CheckedSub, Convert, StaticLookup};
use support::{decl_module, decl_storage, dispatch::Result, ensure, StorageMap};

/// The module's configuration trait.
pub trait Trait: system::Trait + utils::Trait + asset::Trait + erc20::Trait{

}

decl_storage! {
    trait Store for Module<T: Trait> as Settlement {
        pub SettlementContracts get(settlement_contracts): map (Vec<u8>, u64) => T::AccountId;
        pub NumberOfContracts get(number_of_contracts): map Vec<u8> => u64;
        pub ContractPosition get(contract_position): map (Vec<u8>, T::AccountId) => u64;
        pub DepositedSecurityBalance get(deposited_security_balance): map (Vec<u8>, T::AccountId) => T::TokenBalance;
        pub DepositedERC20Balance get(deposited_erc20_balance): map (Vec<u8>, T::AccountId) => T::TokenBalance;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        pub fn add_settlement_contract(origin, _ticker: Vec<u8>, _contract: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");
            ensure!(!<ContractPosition<T>>::exists((_ticker.clone(), _contract.clone())), "Contract already added");
            let number_of_settlement_contracts = Self::number_of_contracts(_ticker.clone()).checked_add(1).ok_or("overflow in increasing number of contracts")?;
            <SettlementContracts<T>>::insert((_ticker.clone(), number_of_settlement_contracts.clone()), _contract.clone());
            <NumberOfContracts<T>>::insert(
                _ticker.clone(),
                number_of_settlement_contracts
            );
            <ContractPosition<T>>::insert((_ticker.clone(), _contract.clone()), number_of_settlement_contracts);
            Ok(())
        }

        pub fn remove_settlement_contract(origin, _ticker: Vec<u8>, _contract: T::AccountId) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            ensure!(Self::is_owner(ticker.clone(),sender.clone()),"Sender must be the token owner");
            ensure!(<ContractPosition<T>>::exists((_ticker.clone(), _contract.clone())), "Contract not added");
            let contract_pos = Self::contract_position((_ticker.clone(), _contract.clone()));
            let last_contract = Self::settlement_contracts((_ticker.clone(), Self::number_of_contracts(_ticker.clone())));
            <SettlementContracts<T>>::insert((_ticker.clone(), contract_pos), last_contract.clone());
            <NumberOfContracts<T>>::insert(
                _ticker.clone(),
                Self::number_of_contracts(_ticker.clone()).checked_sub(1).ok_or("underflow number of contracts")?
            );
            <ContractPosition<T>>::remove((_ticker.clone(), _contract.clone()));
            <ContractPosition<T>>::insert((_ticker.clone(), last_contract.clone()), contract_pos);
            Ok(())
        }

        pub fn deposit_security_tokens(origin, _ticker: Vec<u8>, _amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            //<asset::Module<T>>::_transfer(_ticker.clone(), sender.clone(), 0, _amount.clone());
            let balance = <asset::BalanceOf<T>>::get((_ticker.clone(), sender.clone()));
            let new_balance = balance.checked_sub(&_amount).ok_or("Overflow calculating new owner balance")?;
            let new_deposit_balance = Self::deposited_security_balance((_ticker.clone(), sender.clone())).checked_add(&_amount).ok_or("Overflow in deposit balance")?;
            <asset::BalanceOf<T>>::insert((_ticker.clone(), sender.clone()), new_balance);
            <DepositedSecurityBalance<T>>::insert((_ticker.clone(), sender.clone()), new_deposit_balance);
            Ok(())
        }

        pub fn withdraw_security_tokens(origin, _ticker: Vec<u8>, _amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let new_deposit_balance = Self::deposited_security_balance((_ticker.clone(), sender.clone())).checked_sub(&_amount).ok_or("Underflow in deposit balance")?;
            let new_balance = <asset::BalanceOf<T>>::get((_ticker.clone(), sender.clone())).checked_add(&_amount).ok_or("Overflow calculating new owner balance")?;
            <DepositedSecurityBalance<T>>::insert((_ticker.clone(), sender.clone()), new_deposit_balance);
            <asset::BalanceOf<T>>::insert((_ticker.clone(), sender.clone()), new_balance);
            Ok(())
        }

        pub fn deposit_erc20_tokens(origin, _ticker: Vec<u8>, _amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            //<asset::Module<T>>::_transfer(_ticker.clone(), sender.clone(), 0, _amount.clone());
            let balance = <erc20::BalanceOf<T>>::get((_ticker.clone(), sender.clone()));
            let new_balance = balance.checked_sub(&_amount).ok_or("Overflow calculating new owner balance")?;
            let new_deposit_balance = Self::deposited_erc20_balance((_ticker.clone(), sender.clone())).checked_add(&_amount).ok_or("Overflow in deposit balance")?;
            <erc20::BalanceOf<T>>::insert((_ticker.clone(), sender.clone()), new_balance);
            <DepositedERC20Balance<T>>::insert((_ticker.clone(), sender.clone()), new_deposit_balance);
            Ok(())
        }

        pub fn withdraw_erc20_tokens(origin, _ticker: Vec<u8>, _amount: T::TokenBalance) -> Result {
            let sender = ensure_signed(origin)?;
            let ticker = utils::bytes_to_upper(_ticker.as_slice());
            let new_deposit_balance = Self::deposited_erc20_balance((_ticker.clone(), sender.clone())).checked_sub(&_amount).ok_or("Underflow in deposit balance")?;
            let new_balance = <erc20::BalanceOf<T>>::get((_ticker.clone(), sender.clone())).checked_add(&_amount).ok_or("Overflow calculating new owner balance")?;
            <DepositedERC20Balance<T>>::insert((_ticker.clone(), sender.clone()), new_deposit_balance);
            <erc20::BalanceOf<T>>::insert((_ticker.clone(), sender.clone()), new_balance);
            Ok(())
        }
       // T::Lookup::unlookup(_contract.clone())
    }
}

impl<T: Trait> Module<T> {
    pub fn is_owner(_ticker: Vec<u8>, sender: T::AccountId) -> bool {
        let ticker = utils::bytes_to_upper(_ticker.as_slice());
        <asset::Module<T>>::_is_owner(ticker.clone(), sender)
    }
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
