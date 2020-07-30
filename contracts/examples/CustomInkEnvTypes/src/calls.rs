use ink_core::env::EnvTypes;
use scale::{Codec, Decode, Encode};
use pallet_indices::address::Address;
use sp_runtime::traits::Member;
use crate::{AccountId, AccountIndex, Balance, PolymeshRuntimeTypes, Rule, AssetTransferRule, AssetTransferRules, Ticker};
use ink_prelude::{vec, vec::Vec};

/// Default runtime Call type, a subset of the runtime Call module variants
///
/// The codec indices of the  modules *MUST* match those in the concrete runtime.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub enum Call {
    #[codec(index = "27")]
    ComplianceManager(ComplianceManager<NodeRuntimeTypes, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>),
}

impl From<ComplianceManager<NodeRuntimeTypes, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>> for Call {
    fn from(compliance_manager_call: ComplianceManager<NodeRuntimeTypes, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>) -> Call {
        Call::ComplianceManager(compliance_manager_call)
    }
}
/// Generic ComplianceManager Call, could be used with other runtimes
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum ComplianceManager<T, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>
where
    T: EnvTypes,
    T::AccountId: Member + Codec,
    AccountIndex: Member + Codec,
    Rule: Member + Codec,
    AssetTransferRules: Member + Codec,
    AssetTransferRule: Member + Codec,
    Ticker: Member + Codec
{
    #[allow(non_camel_case_types)]
    add_active_rule(
        Ticker,
        Vec<Rule>,
        Vec<Rule>
    ),
    #[allow(non_camel_case_types)]
    remove_active_rule(Ticker, u32),
}

/// Construct a `ComplianceManager::remove_active_rule` call
pub fn cm_remove_active_rule(ticker: Ticker, active_rule_id: u32) -> Call {
    ComplianceManager::ComplianceManager<NodeRuntimeTypes, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>::remove_active_rule(ticker, active_rule_id).into()
}

/// Construct a `ComplianceManager::add_active_rule` call
pub fn cm_add_active_rule(ticker: Ticker, sender_rules: Vec<Rule>, receiver_rules: Vec<Rule>) -> Call {
    ComplianceManager::ComplianceManager<NodeRuntimeTypes, AccountIndex, Rule, AssetTransferRule, AssetTransferRules, Ticker>::add_active_rule(ticker, sender_rules, receiver_rules).into()
}