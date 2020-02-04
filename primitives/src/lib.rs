//! Shareable types.

#![warn(missing_docs)]
#![cfg_attr(not(feature = "std"), no_std)]

use sp_runtime::{generic, MultiSignature};

pub use sp_runtime::traits::{BlakeTwo256, Hash as HashT, IdentifyAccount, Verify};

pub use codec::Compact;

/// An index to a block.
/// 32-bits will allow for 136 years of blocks assuming 1 block per second.
pub type BlockNumber = u32;

/// An instant or duration in time.
pub type Moment = u64;

/// Alias to 512-bit hash when used in the context of a signature on the relay chain.
/// Equipped with logic for possibly "unsigned" messages.
pub type Signature = MultiSignature;

/// Alias to an sr25519 or ed25519 key.
pub type AccountId = <<Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// The type for looking up accounts. We don't expect more than 4 billion of them.
pub type AccountIndex = u32;

/// Identifier for a chain. 32-bit should be plenty.
pub type ChainId = u32;

/// A hash of some data used by the relay chain.
pub type Hash = sp_core::H256;

/// Index of a transaction in the relay chain. 32-bit should be plenty.
pub type Index = u32;

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

/// Utility byte container where equality comparision are ignored case.
pub mod ignored_case_string;
pub use ignored_case_string::IgnoredCaseString;

/// Role for identities.
pub mod identity_role;
pub use identity_role::IdentityRole;

/// Polymesh Distributed Identity.
pub mod identity_id;
pub use identity_id::IdentityId;

/// Identity information.
/// Each DID is associated with this kind of record.
pub mod identity;
pub use identity::Identity;

/// Key is strong type which stores bytes representing the key.
pub mod account_key;
pub use account_key::AccountKey;

/// This module contains entities related with signing keys.
pub mod signing_item;
pub use signing_item::{Permission, Signer, SignerType, SigningItem};

/// This module defines the needed information to add a pre-authorized key into an identity.
pub mod pre_authorized_key_info;
pub use pre_authorized_key_info::PreAuthorizedKeyInfo;

/// Generic authorization data types for all two step processes
pub mod authorization;
/// Pub Traits
pub mod traits;
pub use authorization::Authorization;
pub use authorization::AuthorizationData;
pub use authorization::AuthorizationError;

/// Generic links that contains information about a key/identity for example ownership of a ticker
pub mod link;
pub use link::Link;
pub use link::LinkData;

pub mod ticker;
pub use ticker::Ticker;

pub mod document;
pub use document::Document;

/// Represents custom transaction errors.
#[repr(u8)]
pub enum TransactionError {
    /// 0-6 are used by substrate. Skipping them to avoid confusion
    ZeroTip = 0,
    /// Transaction needs an Identity associated to an account.
    MissingIdentity = 1,
    /// KYC is required
    RequiredKYC = 2,
}
