#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};

use ink_env::call::{DelegateCall, ExecutionInput, Selector};

use polymesh_api::Api;
pub use polymesh_api::{
    ink::extension::PolymeshEnvironment,
    polymesh::types::polymesh_primitives::{
        asset::{AssetName, AssetType},
        ticker::Ticker,
    },
};

/// The contract error types.
#[derive(Debug, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
pub enum PolymeshError {
    /// Polymesh runtime error.
    PolymeshError,
}

impl From<polymesh_api::ink::Error> for PolymeshError {
    fn from(_err: polymesh_api::ink::Error) -> Self {
        Self::PolymeshError
    }
}

impl From<polymesh_api::ink::extension::PolymeshRuntimeErr> for PolymeshError {
    fn from(_err: polymesh_api::ink::extension::PolymeshRuntimeErr) -> Self {
        Self::PolymeshError
    }
}

/// The contract result type.
pub type PolymeshResult<T> = core::result::Result<T, PolymeshError>;

pub type AccountId = <PolymeshEnvironment as ink_env::Environment>::AccountId;
pub type Balance = <PolymeshEnvironment as ink_env::Environment>::Balance;
pub type Hash = <PolymeshEnvironment as ink_env::Environment>::Hash;

/// Contracts would store this a value of this type.
#[derive(Debug, Default, scale::Encode, scale::Decode)]
#[cfg_attr(feature = "std", derive(scale_info::TypeInfo))]
#[derive(ink_storage::traits::SpreadLayout)]
#[derive(ink_storage::traits::PackedLayout)]
#[cfg_attr(feature = "std", derive(ink_storage::traits::StorageLayout))]
pub struct PolymeshInk {
    hash: Option<Hash>,
}

impl PolymeshInk {
    pub fn new(hash: Option<Hash>) -> Self {
        Self { hash }
    }

    /// Update code hash.
    pub fn update_code_hash(&mut self, hash: Option<Hash>) {
        self.hash = hash;
    }

    /// Very simple api call for testing overhead.
    pub fn system_remark(&mut self, remark: Vec<u8>) -> PolymeshResult<()> {
        if let Some(hash) = self.hash {
            self.delegate_system_remark(hash, remark)
        } else {
            self.direct_system_remark(remark)
        }
    }

    // Use a DelegateCall to to the "wrapped" contract.
    fn delegate_system_remark(&self, hash: Hash, remark: Vec<u8>) -> PolymeshResult<()> {
        ink_env::call::build_call::<ink_env::DefaultEnvironment>()
            .call_type(DelegateCall::new().code_hash(hash))
            .exec_input(
                ExecutionInput::new(Selector::new([0x00, 0x00, 0x00, 0x01])).push_arg(remark),
            )
            .returns::<PolymeshResult<()>>()
            .fire()
            .unwrap_or_else(|err| panic!("delegate call to {:?} failed due to {:?}", hash, err))?;
        Ok(())
    }

    fn direct_system_remark(&mut self, remark: Vec<u8>) -> PolymeshResult<()> {
        let api = Api::new();
        api.call().system().remark(remark).submit()?;
        Ok(())
    }

    /// Just an example "high-level" API.
    pub fn create_simple_asset(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
        if let Some(hash) = self.hash {
            self.delegate_create_simple_asset(hash, ticker, amount)
        } else {
            self.direct_create_simple_asset(ticker, amount)
        }
    }

    // Use a DelegateCall to to the "wrapped" contract.
    fn delegate_create_simple_asset(
        &self,
        hash: Hash,
        ticker: Ticker,
        amount: Balance,
    ) -> PolymeshResult<()> {
        ink_env::call::build_call::<ink_env::DefaultEnvironment>()
            .call_type(DelegateCall::new().code_hash(hash))
            .exec_input(
                ExecutionInput::new(Selector::new([0x00, 0x00, 0x1a, 0x01]))
                    .push_arg(ticker)
                    .push_arg(amount),
            )
            .returns::<PolymeshResult<()>>()
            .fire()
            .unwrap_or_else(|err| panic!("delegate call to {:?} failed due to {:?}", hash, err))?;
        Ok(())
    }

    // Directly use the chain extension.
    fn direct_create_simple_asset(&self, ticker: Ticker, amount: Balance) -> PolymeshResult<()> {
        let api = Api::new();
        // Create asset.
        api.call()
            .asset()
            .create_asset(
                AssetName(b"".to_vec()),
                ticker.into(),
                true, // Divisible token.
                AssetType::EquityCommon,
                vec![],
                None,
                true, // Disable Investor uniqueness requirements.
            )
            .submit()?;
        // Mint some tokens.
        api.call().asset().issue(ticker.into(), amount).submit()?;
        // Pause compliance rules to allow transfers.
        api.call()
            .compliance_manager()
            .pause_asset_compliance(ticker.into())
            .submit()?;
        Ok(())
    }
}
