// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Shareable types.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{
    format,
    string::{String, ToString},
};
use codec::{CompactAs, Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use core::ops::Add;
use frame_support::parameter_types;
use frame_support::traits::Get;
use polymesh_primitives_derive::{SliceU8StrongTyped, StringStrongTyped, VecU8StrongTyped};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_runtime::{generic, traits::BlakeTwo256, MultiSignature};
use sp_std::prelude::Vec;

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to 512-bit hash when used in the context of a signature on the relay chain.
/// Equipped with logic for possibly "unsigned" messages.
pub type Signature = MultiSignature;

/// Alias for `EnsureRoot<AccountId>`.
pub type EnsureRoot = frame_system::EnsureRoot<AccountId>;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Identifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// A hash of some data used by the relay chain.
pub type Hash = sp_core::H256;

/// Nonce of a transaction. 32-bit should be plenty.
pub type Nonce = u32;

/// Alias for Gas.
pub type Gas = Weight;

/// Use `GetExtra` as the trait bounds for pallet `Config` parameters
/// that will be used for bounded collections.
pub trait GetExtra<T>: Get<T> + Clone + core::fmt::Debug + Default + PartialEq + Eq {}

/// ConstSize type wrapper.
///
/// This allows the use of Bounded collections in extrinsic parameters.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstSize<const T: u32>;

impl<const T: u32> Get<u32> for ConstSize<T> {
    fn get() -> u32 {
        T
    }
}

impl<const T: u32> GetExtra<u32> for ConstSize<T> {}

/// Either a block number, or nothing.
#[derive(Clone, Copy, Debug, Default, Deserialize, MaxEncodedLen, Serialize)]
#[derive(Decode, DecodeWithMemTracking, Encode, Eq, PartialEq, TypeInfo)]
pub enum MaybeBlock<BlockNumber> {
    /// Has a block number.
    Some(BlockNumber),
    /// No block number.
    #[default]
    None,
}

impl<T: Add<Output = T>> Add<T> for MaybeBlock<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        match self {
            MaybeBlock::Some(lhs) => MaybeBlock::Some(lhs + rhs),
            MaybeBlock::None => MaybeBlock::None,
        }
    }
}

/// A positive coefficient: a pair of a numerator and a denominator. Defaults to `(1, 1)`.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct PosRatio(pub u32, pub u32);

impl Default for PosRatio {
    fn default() -> Self {
        PosRatio(1, 1)
    }
}

impl From<(u32, u32)> for PosRatio {
    fn from((n, d): (u32, u32)) -> Self {
        PosRatio(n, d)
    }
}

/// The balance of an account.
/// 128-bits (or 38 significant decimal figures) will allow for 10m currency (10^7) at a resolution
/// to all for one second's worth of an annualised 50% reward be paid to a unit holder (10^11 unit
/// denomination), or 10^18 total atomic units, to grow at 50%/year for 51 years (10^9 multiplier)
/// for an eventual total of 10^27 units (27 significant decimal figures).
/// We round denomination to 10^12 (12 sdf), and leave the other redundancy at the upper end so
/// that 32 bits may be multiplied with a balance in 128 bits without worrying about overflow.
pub type Balance = u128;

/// Header type.
pub type Header = generic::Header<BlockNumber, BlakeTwo256>;
/// Block type.
pub type Block = generic::Block<Header, UncheckedExtrinsic>;
/// Block ID.
pub type BlockId = generic::BlockId<Block>;

/// Opaque, encoded, unchecked extrinsic.
pub use sp_runtime::OpaqueExtrinsic as UncheckedExtrinsic;

/// Utility byte container where equality comparison are ignored case.
pub mod ignored_case_string;
pub use ignored_case_string::IgnoredCaseString;

/// Account types.
pub mod account;
pub use account::{AccountId, HexAccountId};

/// External agents.
pub mod agent;

/// Asset identifiers.
pub mod asset_identifier;
pub use asset_identifier::AssetIdentifier;

pub mod event_only;
pub use event_only::EventOnly;

/// Polymesh Distributed Identity.
pub mod identity_id;
pub use identity_id::{
    EventDid, IdentityId, PortfolioId, PortfolioKind, PortfolioName, PortfolioNumber,
};

/// Identity information.
/// Each DID is associated with this kind of record.
pub mod identity;
pub use identity::DidRecord;

/// Provides the `CheckedInc` trait.
pub mod checked_inc;

/// CDD Id.
pub mod cdd_id;
pub use cdd_id::CddId;

/// Claim information.
/// Each claim is associated with this kind of record.
pub mod identity_claim;
pub use identity_claim::{Claim, ClaimType, CustomClaimTypeId, IdentityClaim, Scope};

// Defining and enumerating jurisdictions.
pub mod jurisdiction;
pub use jurisdiction::CountryCode;

/// Utilities for storage migration.
pub mod migrate;

/// This module contains entities related with secondary keys.
pub mod secondary_key;
pub use secondary_key::{
    AssetPermissions, ExtrinsicPermissions, KeyRecord, PalletPermissions, Permissions,
    PortfolioPermissions, SecondaryKey, Signatory,
};

/// Subset type.
pub mod subset;
pub use subset::SubsetRestriction;

/// Generic authorization data types for all two step processes
pub mod authorization;
pub use authorization::{Authorization, AuthorizationData, AuthorizationType};

/// Pub Traits
pub mod traits;

/// Benchmarking helpers.
#[cfg(feature = "runtime-benchmarks")]
pub mod bench;

pub mod ticker;
pub use ticker::Ticker;

/// Document hash
pub mod document_hash;
pub use document_hash::DocumentHash;

/// Document types
pub mod document;
pub use document::{Document, DocumentId, DocumentName, DocumentUri};

/// Rules for claims.
pub mod condition;
pub use condition::{Condition, ConditionType, TargetIdentity, TrustedFor, TrustedIssuer};

/// Predicate calculation for Claims.
pub mod proposition;
pub use proposition::{AndProposition, Context, NotProposition, OrProposition, Proposition};

/// Protocol fee types and traits.
pub mod protocol_fee;

/// Timekeeping and checkpoints.
pub mod calendar;

/// Checkpoint types and traits.
pub mod checkpoint;

/// Runtime crypto tools.
pub mod crypto;

/// Asset type definitions.
pub mod asset;
pub use asset::{AssetHolder, AssetHolderKind, HoldingsUpdateReason};

/// Asset Metadata type definitions.
pub mod asset_metadata;

/// Statistics type definitions.
pub mod statistics;

/// Compliance manager type definitions.
pub mod compliance_manager;

/// Transfer compliance type definitions.
pub mod transfer_compliance;

/// Committee type definitions.
pub mod committee;

/// NFT type definitions.
pub mod nft;
pub use nft::{NFTCollectionId, NFTCollectionKeys, NFTId, NFTMetadataAttribute, NFTs};

/// Portfolio type definitions.
pub mod portfolio;
pub use portfolio::{Fund, FundDescription};

/// Custom WeightMeter definitions.
pub mod weight_meter;
pub use weight_meter::WeightMeter;

/// Re-exports of some common types.
pub use frame_support::weights::{
    constants::{WEIGHT_REF_TIME_PER_MICROS, WEIGHT_REF_TIME_PER_NANOS},
    RuntimeDbWeight, Weight,
};
parameter_types! {
    /// Polymesh custom db weights.
    /// When the read/writes are cached/buffered, they take 25/100 microseconds on NVMe disks.
    /// When they are uncached, they take 250/450 microseconds on NVMe disks.
    /// Most read will be cached and writes will be buffered in production.
    /// We are taking a number slightly higher than what cached suggest to allow for some extra breathing room.
    pub const RocksDbWeight: RuntimeDbWeight = RuntimeDbWeight {
        read: 50 * WEIGHT_REF_TIME_PER_MICROS, // ~50 µs @ 100,000 items
        write: 200 * WEIGHT_REF_TIME_PER_MICROS, // ~200 µs @ 100,000 items
    };
}

/// Settlement type definitions.
pub mod settlement;

/// STO type definitions.
pub mod sto;

/// Constants definitions.
pub mod constants;
pub use constants::{SystematicIssuers, GC_DID, SYSTEMATIC_ISSUERS, TECHNICAL_DID, UPGRADE_DID};

/// Multisig type definitions.
pub mod multisig;

/// Represents custom transaction errors.
#[repr(u8)]
pub enum TransactionError {
    /// 0-6 are used by substrate. Skipping them to avoid confusion
    ZeroTip = 0,
    /// Transaction needs an Identity associated to an account.
    MissingIdentity = 1,
    /// CDD is required
    CddRequired = 2,
    /// Invalid auth id
    InvalidAuthorization = 3,
    /// Subsidy is not available for this pallet.
    PalletNotSubsidised = 4,
}

/// Represents the target identity and the amount requested by a beneficiary.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Beneficiary<Balance> {
    /// Beneficiary identity.
    pub id: IdentityId,
    /// Amount requested to this beneficiary.
    pub amount: Balance,
}

/// A short on-chain memo for POLYX transfer, asset transfer and portfolio moves.
#[derive(Decode, DecodeWithMemTracking, Encode, MaxEncodedLen, TypeInfo)]
#[derive(Hash, PartialEq, Eq, PartialOrd, Ord, SliceU8StrongTyped)]
#[derive(Clone, Default)]
pub struct Memo(pub [u8; 32]);

/// Url for linking to off-chain resources.
#[derive(Decode, DecodeWithMemTracking, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct Url(#[serde(with = "serde_bytes")] pub Vec<u8>);

/// The name of a pallet.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, StringStrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct PalletName(pub String);

impl PalletName {
    /// Helper method to generate a PalletName.  Used for tests and benchmarks.
    pub fn generate(n: u64) -> Self {
        Self(format!("P{n}"))
    }
}

/// The name of an extrinsic within a pallet.
#[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, StringStrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[derive(Serialize, Deserialize)]
pub struct ExtrinsicName(pub String);

impl ExtrinsicName {
    /// Helper method to generate a PalletName.  Used for tests and benchmarks.
    pub fn generate(n: u64) -> Self {
        Self(format!("E{n}"))
    }
}

/// The old weight type.
///
/// NOTE: This type exists purely for compatibility purposes! Use [`weight_v2::Weight`] in all other
/// cases.
#[derive(
    Decode,
    Encode,
    CompactAs,
    PartialEq,
    Eq,
    Clone,
    Copy,
    Debug,
    Default,
    MaxEncodedLen,
    TypeInfo
)]
#[derive(Serialize, Deserialize)]
#[serde(transparent)]
pub struct OldWeight(pub u64);

/// Execute the supplied function in a new storage transaction,
/// committing on `Ok(_)` and rolling back on `Err(_)`, returning the result.
///
/// Transactions can be arbitrarily nested with commits happening to the parent.
pub fn with_transaction<T, E: From<frame_support::pallet_prelude::DispatchError>>(
    tx: impl FnOnce() -> Result<T, E>,
) -> Result<T, E> {
    use frame_support::storage::{with_transaction, TransactionOutcome};
    with_transaction(|| match tx() {
        r @ Ok(_) => TransactionOutcome::Commit(r),
        r @ Err(_) => TransactionOutcome::Rollback(r),
    })
}

/// Compile time assert.
#[macro_export]
macro_rules! const_assert {
    ($x:expr $(,)?) => {
        #[allow(unknown_lints, eq_op)]
        const _: [(); 0 - !{
            const ASSERT: bool = $x;
            ASSERT
        } as usize] = [];
    };
}

/// Create a `Version` struct with an upper limit.
#[macro_export]
macro_rules! storage_migration_ver {
    ($ver:literal) => {
        #[derive(frame_support::pallet_prelude::MaxEncodedLen)]
        #[derive(Encode, Decode, scale_info::TypeInfo)]
        #[derive(Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
        pub struct Version(u8);

        impl Version {
            const MAX: u8 = $ver;

            /// Build const version and do compile-time maximum version check.
            pub const fn new(ver: u8) -> Self {
                Self(ver)
            }

            const fn check_version(&self) -> bool {
                self.0 <= Self::MAX
            }
        }

        impl Default for Version {
            fn default() -> Self {
                Version(0)
            }
        }
    };
}

/// Helper macro which execute the `$body` if `$storage` is less than version `$ver`.
/// It also updates `$storage` in the current pallet to `$ver`.
#[macro_export]
macro_rules! storage_migrate_on {
    ($storage: ty, $ver:literal, $body: block) => {{
        const TARGET_VERSION: Version = Version::new($ver);
        polymesh_primitives::const_assert!(TARGET_VERSION.check_version());
        if <$storage>::get() < TARGET_VERSION {
            $body;
            <$storage>::put(TARGET_VERSION);
        }
    }};
}

#[cfg(test)]
mod tests {
    use polymesh_primitives_derive::{SliceU8StrongTyped, VecU8StrongTyped};

    use codec::{Decode, Encode};

    #[derive(VecU8StrongTyped)]
    struct A(Vec<u8>);
    #[derive(VecU8StrongTyped)]
    struct B(Vec<u8>);

    #[derive(Default, SliceU8StrongTyped)]
    struct C([u8; 16]);
    #[derive(Default, SliceU8StrongTyped)]
    struct D([u8; 16]);

    #[test]
    fn vec_strong_typed() {
        let text1 = &b"Lorem Ipsum ";
        let text2 = &b"dolor sit amet";
        // let text3 = &b"Lorem Ipsum dolor sit amet";

        // From traits.
        let mut a1 = A::from(text1);
        let a2: A = text1.into();
        let _b1 = B::from(text2.to_vec());
        let _b2: B = text2.to_vec().into();

        // Deref & DerefMut
        assert_eq!(*a1, text1[..]);
        a1[6] = b'i';
        assert_eq!(a1.as_slice(), &b"Lorem ipsum "[..]);

        // Other methods.
        assert_eq!(a2.len(), text1.len());
        assert_eq!(a2.as_slice(), &text1[..]);
        assert_eq!(a2.as_vec().clone(), text1.to_vec());

        // Strong types are not equal.
        // The below line does NOT compile.
        // let a3 :A = _b1;
    }

    #[test]
    fn slice_strong_typed() {
        let text1 = &b"lorem Ipsum ";
        let text2 = &b"dolor sit amet";
        // let text3 = &b"Lorem Ipsum dolor sit amet";

        // From traits.
        let mut c1 = C::from(text1.as_ref());
        let c2: C = C::from(text1.as_ref());
        let _d1 = D::from(text2.as_ref());

        // Deref & DerefMut
        let text1_with_zeros = &b"lorem Ipsum \0\0\0\0";
        assert_eq!(*c1, text1_with_zeros[..]);
        c1[6] = b'i';
        assert_eq!(c1.as_slice(), &b"lorem ipsum \0\0\0\0"[..]);

        // Other methods.
        assert_eq!(c2.len(), text1_with_zeros.len());
        assert_eq!(c2.as_slice(), &text1_with_zeros[..]);

        // Strong types are not equal.
        // The below line does NOT compile.
        // let c3 :C = _d1;
    }

    #[test]
    fn storage_migration_ver_test_1() {
        storage_migration_ver!(3);

        assert!(Version::new(2).check_version());
        assert!(!Version::new(4).check_version());
    }
}
