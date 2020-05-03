#![allow(dead_code)]

pub mod storage;
pub use storage::{
    account_from, add_signing_item, get_identity_id, make_account, make_account_with_balance,
    make_account_without_cdd, register_keyring_account_with_balance,
    register_keyring_account_without_cdd, TestStorage,
};

pub mod ext_builder;
pub use ext_builder::ExtBuilder;

mod asset_test;
mod balances_test;
mod bridge;
mod committee_test;
mod compliance_manager_test;
mod dividend_test;
mod fee_details;
mod group_test;
mod identity_test;
mod multisig;
mod pips_test;
mod protocol_fee;
mod simple_token_test;
mod statistics_test;
mod treasury_test;
mod voting_test;
