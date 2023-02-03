#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use alloc::{vec, vec::Vec};

use ink_env::call::{DelegateCall, ExecutionInput, Selector};

#[cfg(feature = "tracker")]
pub use upgrade_tracker::UpgradeTrackerRef;

use polymesh_api::Api;
pub use polymesh_api::{
    ink::extension::PolymeshEnvironment,
    polymesh::types::polymesh_primitives::{
        asset::{AssetName, AssetType},
        ticker::Ticker,
    },
};

pub const API_VERSION: u32 = 5;

macro_rules! upgradable_func {
    ($func:ident ($($param:ident: $ty:ty),+) $code:expr) => {
        pub fn $func(&self, $($param: $ty),+) -> PolymeshResult<()> {
            if let Some(hash) = self.hash {
                const FUNC: &str = stringify!($func);
                let selector = ink_lang_ir::Selector::compute(FUNC.as_bytes()).to_bytes();
                ink_env::call::build_call::<ink_env::DefaultEnvironment>()
                    .call_type(DelegateCall::new().code_hash(hash))
                    .exec_input(
                        ExecutionInput::new(Selector::new(selector))
                            .push_arg(($($param),+)),
                    )
                    .returns::<PolymeshResult<()>>()
                    .fire()
                    .unwrap_or_else(|err| panic!("delegate call to {:?} failed due to {:?}", hash, err))?;
            } else {
                $code
            }
            Ok(())
        }
    }
}

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
    #[cfg(feature = "tracker")]
    tracker: Option<UpgradeTrackerRef>,
}

impl PolymeshInk {
    #[cfg(not(feature = "tracker"))]
    pub fn new(hash: Option<Hash>) -> Self {
        Self { hash }
    }

    #[cfg(feature = "tracker")]
    pub fn new(hash: Option<Hash>, tracker: Option<UpgradeTrackerRef>) -> Self {
        Self { hash, tracker }
    }

    /// Update code hash.
    pub fn update_code_hash(&mut self, hash: Option<Hash>) {
        self.hash = hash;
    }

    #[cfg(feature = "tracker")]
    pub fn check_for_upgrade(&mut self) {
        if let Some(tracker) = &self.tracker {
            self.hash = tracker.get_latest_upgrade(API_VERSION);
        }
    }

    upgradable_func!(system_remark (remark: Vec<u8>) {
        let api = Api::new();
        api.call().system().remark(remark).submit()?;
    });

    upgradable_func!(asset_issue (ticker: Ticker, amount: Balance) {
        let api = Api::new();
        // Mint some tokens.
        api.call().asset().issue(ticker, amount).submit()?;
    });

    upgradable_func!(asset_create_and_issue (ticker: Ticker, amount: Balance) {
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
    });
}
