#![cfg_attr(not(feature = "std"), no_std)]

pub use ink_core::env::{ Balance, AccountId, Timestamp, BlockNumber, Hash};
use scale::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use ink_core::storage::Flush;
use ink_prelude::{vec, vec::Vec};

pub mod calls;

/// Contract environment types defined in substrate node-runtime
#[cfg_attr(feature = "ink-generate-abi", derive(Metadata))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolymeshRuntimeTypes {}

pub type AccountIndex = u32;

impl ink_core::env::EnvTypes for PolymeshRuntimeTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Timestamp = Timestamp;
    type BlockNumber = BlockNumber;
    type Call = calls::Call;
}

const TICKER_LEN: usize = 12;

#[derive(Decode, Encode, PartialEq, Ord, Eq, PartialOrd, Copy, Hash, Clone, Default, Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IdentityId([u8; 32]);

impl Flush for IdentityId {}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(
    Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord,
)]
pub struct JurisdictionName(pub Vec<u8>);

/// Scope: Almost all claim needs a valid scope identity.
pub type Scope = IdentityId;

/// All possible claims in polymesh
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Claim {
    /// User is Accredited
    Accredited(Scope),
    /// User is Accredited
    Affiliate(Scope),
    /// User has an active BuyLockup (end date defined in claim expiry)
    BuyLockup(Scope),
    /// User has an active SellLockup (date defined in claim expiry)
    SellLockup(Scope),
    /// User has passed CDD
    CustomerDueDiligence,
    /// User is KYC'd
    KnowYourCustomer(Scope),
    /// This claim contains a string that represents the jurisdiction of the user
    Jurisdiction(JurisdictionName, Scope),
    /// User is exempted
    Exempted(Scope),
    /// User is Blocked
    Blocked(Scope),
    /// Empty claim
    NoData,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
/// It defines the type of rule supported, and the filter information we will use to evaluate as a
/// predicate.
pub enum RuleType {
    /// Rule to ensure that claim filter produces one claim.
    IsPresent(Claim),
    /// Rule to ensure that claim filter produces an empty list.
    IsAbsent(Claim),
    /// Rule to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
    /// Rule to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(Vec<Claim>),
}

/// Type of claim requirements that a rule can have
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Rule {
    /// Type of rule.
    pub rule_type: RuleType,
    /// Trusted issuers.
    pub issuers: Vec<IdentityId>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRule {
    pub sender_rules: Vec<Rule>,
    pub receiver_rules: Vec<Rule>,
    /// Unique identifier of the asset rule
    pub rule_id: u32,
}

/// List of rules associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetTransferRules {
    /// This flag indicates if asset transfer rules are active or paused.
    pub is_paused: bool,
    /// List of rules.
    pub rules: Vec<AssetTransferRule>,
}

/// Ticker symbol.
///
/// This type stores fixed-length case-sensitive byte strings. Any value of this type that is
/// received by a Substrate module call method has to be converted to canonical uppercase
/// representation using [`Ticker::canonize`].
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Ticker([u8; TICKER_LEN]);

impl Default for Ticker {
    fn default() -> Self {
        Ticker([0u8; TICKER_LEN])
    }
}