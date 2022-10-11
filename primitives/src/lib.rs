// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymath

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

use blake2::{Blake2b, Digest};
use codec::{Decode, Encode};
use confidential_identity_v1::Scalar as ScalarV1;
use frame_support::weights::Weight;
use polymesh_primitives_derive::VecU8StrongTyped;
use scale_info::TypeInfo;
use sp_runtime::{generic, traits::BlakeTwo256, MultiSignature};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
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

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Index = u32;

/// Alias for Gas.
pub type Gas = Weight;

/// A positive coefficient: a pair of a numerator and a denominator. Defaults to `(1, 1)`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
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

/// It creates a scalar from the blake2_512 hash of `data` parameter.
pub fn scalar_blake2_from_bytes(data: impl AsRef<[u8]>) -> ScalarV1 {
    let hash = Blake2b::default()
        .chain_update(data.as_ref())
        .finalize()
        .into();
    ScalarV1::from_bytes_mod_order_wide(&hash)
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

/// CDD Identity is an ID to link the encrypted investor UID with one Identity ID.
/// That keeps the privacy of a real investor and its global portfolio split in several Polymesh
/// Identities.
pub mod cdd_id;
pub use cdd_id::{CddId, InvestorUid};

/// Investor Zero Knowledge Proof data
pub mod investor_zkproof_data;
pub use investor_zkproof_data::InvestorZKProofData;

/// Claim information.
/// Each claim is associated with this kind of record.
pub mod identity_claim;
pub use identity_claim::{Claim, ClaimType, CustomClaimTypeId, IdentityClaim, Scope, ScopeId};

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
pub use subset::{LatticeOrd, LatticeOrdering, SubsetRestriction};

/// Generic authorization data types for all two step processes
pub mod authorization;
pub use authorization::{Authorization, AuthorizationData, AuthorizationError, AuthorizationType};

/// Pub Traits
pub mod traits;

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

/// For confidential stuff.
pub mod valid_proof_of_investor;

/// Timekeeping and checkpoints.
pub mod calendar;

/// Runtime crypto tools.
pub mod crypto;

/// Asset type definitions.
pub mod asset;

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

/// Host functions.
pub mod host_functions;

pub mod ethereum;

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
#[derive(Encode, Decode, TypeInfo, Clone, PartialEq, Eq, Debug)]
pub struct Beneficiary<Balance> {
    /// Beneficiary identity.
    pub id: IdentityId,
    /// Amount requested to this beneficiary.
    pub amount: Balance,
}

/// Url for linking to off-chain resources.
#[derive(Decode, Encode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url(pub Vec<u8>);

/// The name of a pallet.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct PalletName(pub Vec<u8>);

/// The name of a function within a pallet.
#[derive(Encode, Decode, TypeInfo, VecU8StrongTyped)]
#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct DispatchableName(pub Vec<u8>);

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
        #[derive(Encode, Decode, scale_info::TypeInfo)]
        #[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub struct Version(u8);

        impl Version {
            const MAX: u8 = $ver;

            /// Build const version and do compile-time maximum version check.
            const fn new(ver: u8) -> Self {
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
