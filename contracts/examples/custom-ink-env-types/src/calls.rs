use crate::{Condition, Ticker};
use ink_prelude::vec::Vec;
use scale::{Codec, Decode, Encode};
use sp_runtime::traits::Member;

/// Default runtime Call type, a subset of the runtime Call module variants
///
/// The codec indices of the  modules *MUST* match those in the concrete runtime.
#[derive(Encode, Decode)]
#[cfg_attr(feature = "std", derive(Clone, PartialEq, Eq))]
pub enum Call {
    #[codec(index = "27")]
    ComplianceManager(ComplianceManager<Condition, Ticker>),
}

impl From<ComplianceManager<Condition, Ticker>> for Call {
    fn from(compliance_manager_call: ComplianceManager<Condition, Ticker>) -> Call {
        Call::ComplianceManager(compliance_manager_call)
    }
}
/// Generic ComplianceManager Call, could be used with other runtimes
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub enum ComplianceManager<Condition, Ticker>
where
    Condition: Member + Codec,
    Ticker: Member + Codec,
{
    #[allow(non_camel_case_types)]
    add_compliance_requirement(Ticker, Vec<Condition>, Vec<Condition>),
    #[allow(non_camel_case_types)]
    remove_compliance_requirement(Ticker, u32),
}

/// Construct a `ComplianceManager::remove_compliance_requirement` call
pub fn cm_remove_compliance_requirement(ticker: Ticker, id: u32) -> Call {
    ComplianceManager::<Condition, Ticker>::remove_compliance_requirement(ticker, id).into()
}

/// Construct a `ComplianceManager::add_compliance_requirement` call
pub fn cm_add_compliance_requirement(
    ticker: Ticker,
    sender_conditions: Vec<Condition>,
    receiver_conditions: Vec<Condition>,
) -> Call {
    ComplianceManager::<Condition, Ticker>::add_compliance_requirement(ticker, sender_conditions, receiver_conditions).into()
}
