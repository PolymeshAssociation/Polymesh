#![allow(dead_code)]
#![cfg(test)]
// `construct_runtime!` does a lot of recursion and requires us to increase the limit to 256.
#![recursion_limit = "1024"]

pub(crate) use storage::{next_block, TestStorage};

pub(crate) use ext_builder::ExtBuilder;

mod asset_pallet;
#[macro_use]
mod asset_test;
mod asset_metadata_test;
mod balances_test;
mod committee_test;
mod compliance_manager_test;
mod corporate_actions_test;
#[macro_use]
mod external_agents_test;
pub(crate) mod ext_builder;
mod fee_details;
mod group_test;
mod identity_test;
mod multisig;
mod nft;
mod pips_test;
mod portfolio;
mod precompiles;
mod protocol_fee;
mod relayer_test;
mod settlement_pallet;
mod settlement_test;
mod signed_extra;
mod staking_extra_tests;
mod sto_test;
pub(crate) mod storage;
mod transaction_payment_test;
mod transfer_compliance_test;
mod treasury_test;
mod utility_test;
