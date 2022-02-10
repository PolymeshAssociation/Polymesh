#![cfg_attr(not(feature = "std"), no_std)]

use core::{array::TryFromSliceError, convert::TryFrom};
use derive_more::From;
use ink_core::env::Clear;
use ink_core::storage::Flush;
use ink_prelude::vec::Vec;
use scale::{Decode, Encode};
#[cfg(feature = "std")]
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

pub mod calls;

/// Contract environment types defined in substrate node-runtime
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PolymeshRuntimeTypes {}

/// The default balance type.
pub type Balance = u128;

/// The default timestamp type.
pub type Timestamp = u64;

/// The default block number type.
pub type BlockNumber = u64;

/// The default environment `AccountId` type.
///
/// # Note
///
/// This is a mirror of the `AccountId` type used in the default configuration
/// of PALLET contracts.
#[derive(Encode, Decode, From)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct AccountId([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for AccountId {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
    }
}

/// The default environment `Hash` type.
///
/// # Note
///
/// This is a mirror of the `Hash` type used in the default configuration
/// of PALLET contracts.
#[derive(Encode, Decode, From)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Default)]
#[cfg_attr(feature = "std", derive(TypeInfo))]
pub struct Hash([u8; 32]);

impl<'a> TryFrom<&'a [u8]> for Hash {
    type Error = TryFromSliceError;

    fn try_from(bytes: &'a [u8]) -> Result<Self, TryFromSliceError> {
        let address = <[u8; 32]>::try_from(bytes)?;
        Ok(Self(address))
    }
}

impl AsRef<[u8]> for Hash {
    fn as_ref(&self) -> &[u8] {
        &self.0[..]
    }
}

impl AsMut<[u8]> for Hash {
    fn as_mut(&mut self) -> &mut [u8] {
        &mut self.0[..]
    }
}

impl Clear for Hash {
    fn is_clear(&self) -> bool {
        self.as_ref().iter().all(|&byte| byte == 0x00)
    }

    fn clear() -> Self {
        Self([0x00; 32])
    }
}

impl ink_core::env::EnvTypes for PolymeshRuntimeTypes {
    type AccountId = AccountId;
    type Balance = Balance;
    type Hash = Hash;
    type Timestamp = Timestamp;
    type BlockNumber = BlockNumber;
    type Call = calls::Call;
}

const TICKER_LEN: usize = 12;

#[derive(Decode, Encode)]
#[derive(PartialEq, Ord, Eq, PartialOrd, Copy, Hash, Clone, Default, Debug)]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IdentityId([u8; 32]);

impl Flush for IdentityId {}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Decode, Encode)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
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
/// It defines the type of Condition supported, and the filter information we will use to evaluate as a
/// predicate.
pub enum ConditionType {
    /// Condition to ensure that claim filter produces one claim.
    IsPresent(Claim),
    /// Condition to ensure that claim filter produces an empty list.
    IsAbsent(Claim),
    /// Condition to ensure that at least one claim is fetched when filter is applied.
    IsAnyOf(Vec<Claim>),
    /// Condition to ensure that at none of claims is fetched when filter is applied.
    IsNoneOf(Vec<Claim>),
}

/// Type of claim requirements that a condition can have
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct Condition {
    /// Type of condition.
    pub condition_type: ConditionType,
    /// Trusted issuers.
    pub issuers: Vec<IdentityId>,
}

#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct ComplianceRequirement {
    pub sender_conditions: Vec<Condition>,
    pub receiver_conditions: Vec<Condition>,
    /// Unique identifier of the asset condition
    pub id: u32,
}

/// List of conditions associated to an asset.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Encode, Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct AssetCompliance {
    /// This flag indicates if asset transfer conditions are active or paused.
    pub is_paused: bool,
    /// List of conditions.
    pub requirements: Vec<ComplianceRequirement>,
}

/// Ticker symbol.
///
/// This type stores fixed-length case-sensitive byte strings. Any value of this type that is
/// received by a Substrate module call method has to be converted to canonical uppercase
/// representation using [`Ticker::canonize`].
#[cfg_attr(feature = "ink-generate-abi", derive(type_metadata::Metadata))]
#[derive(Decode, Encode)]
#[derive(Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Ticker([u8; TICKER_LEN]);

impl Default for Ticker {
    fn default() -> Self {
        Ticker([0u8; TICKER_LEN])
    }
}
