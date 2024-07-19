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

//! # Bridge from Ethereum to Polymesh
//! Removed.

#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{decl_module, decl_storage};
use polymesh_primitives::{storage_migration_ver, Balance, IdentityId};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::{fmt::Debug, prelude::*, vec};

pub trait Config: frame_system::Config {}

/// The status of a bridge transaction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BridgeTxStatus {
    /// No such transaction in the system.
    Absent,
    /// The transaction is missing a CDD or the bridge module is frozen.  The `u8` parameter is the
    /// capped number of times the module tried processing this transaction.  It will be retried
    /// automatically. Anyone can retry these manually.
    Pending(u8),
    /// The transaction is frozen by the admin. It will not be retried automatically.
    Frozen,
    /// The transaction is pending its first execution. These can not be manually triggered by
    /// normal accounts.
    Timelocked,
    /// The transaction has been successfully credited.
    Handled,
}

impl Default for BridgeTxStatus {
    fn default() -> Self {
        BridgeTxStatus::Absent
    }
}

/// A unique lock-and-mint bridge transaction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTx<Account> {
    /// A single transaction hash can have multiple locks. This nonce differentiates between them.
    pub nonce: u32,
    /// The recipient account of POLYX on Polymesh.
    pub recipient: Account,
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Ethereum token lock transaction hash. It is not used internally in the bridge and is kept
    /// here for compatibility reasons only.
    pub tx_hash: H256,
}

/// Additional details of a bridge transaction.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct BridgeTxDetail<BlockNumber> {
    /// Amount of POLYX tokens to credit.
    pub amount: Balance,
    /// Status of the bridge transaction.
    pub status: BridgeTxStatus,
    /// Block number at which this transaction was executed or is planned to be executed.
    pub execution_block: BlockNumber,
    /// Ethereum token lock transaction hash. It is not used internally in the bridge and is kept
    /// here for compatibility reasons only.
    pub tx_hash: H256,
}

storage_migration_ver!(0);

decl_storage! {
    trait Store for Module<T: Config> as Bridge {
        /// The multisig account of the bridge controller. The genesis signers accept their
        /// authorizations and are able to get their proposals delivered. The bridge creator
        /// transfers some POLY to their identity.
        pub Controller get(fn controller): Option<T::AccountId>;

        /// Details of bridge transactions identified with pairs of the recipient account and the
        /// bridge transaction nonce.
        pub BridgeTxDetails get(fn bridge_tx_details): double_map
                hasher(blake2_128_concat) T::AccountId,
                hasher(blake2_128_concat) u32
            =>
                BridgeTxDetail<T::BlockNumber>;

        /// The admin key.
        Admin get(fn admin): Option<T::AccountId>;

        /// Whether or not the bridge operation is frozen.
        Frozen get(fn frozen): bool;

        /// Freeze bridge admins.  These accounts can only freeze the bridge.
        FreezeAdmins get(fn freeze_admins): map hasher(blake2_128_concat) T::AccountId => bool;

        /// The bridge transaction timelock period, in blocks, since the acceptance of the
        /// transaction proposal during which the admin key can freeze the transaction.
        Timelock get(fn timelock) config(): T::BlockNumber;

        /// The maximum number of bridged POLYX per identity within a set interval of
        /// blocks. Fields: POLYX amount and the block interval duration.
        BridgeLimit get(fn bridge_limit) config(): (Balance, T::BlockNumber);

        /// Amount of POLYX bridged by the identity in last block interval. Fields: the bridged
        /// amount and the last interval number.
        PolyxBridged get(fn polyx_bridged): map hasher(identity) IdentityId => (Balance, T::BlockNumber);

        /// Identities not constrained by the bridge limit.
        BridgeLimitExempted get(fn bridge_exempted): map hasher(twox_64_concat) IdentityId => bool;

        /// Storage version.
        StorageVersion get(fn storage_version) build(|_| Version::new(0)): Version;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
    }
}
