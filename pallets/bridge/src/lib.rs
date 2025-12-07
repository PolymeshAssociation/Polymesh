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
use frame_system::pallet_prelude::*;
use polymesh_primitives::{storage_migration_ver, Balance, IdentityId};
use scale_info::TypeInfo;
use sp_core::H256;
use sp_std::{fmt::Debug, prelude::*, vec};

pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use super::*;
    use frame_support::pallet_prelude::*;

    #[pallet::config]
    pub trait Config: frame_system::Config {}

    #[pallet::pallet]
    pub struct Pallet<T>(_);

    /// The status of a bridge transaction.
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
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
    #[derive(Encode, Decode, TypeInfo, MaxEncodedLen)]
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

    /// The multisig account of the bridge controller. The genesis signers accept their
    /// authorizations and are able to get their proposals delivered. The bridge creator
    /// transfers some POLY to their identity.
    #[pallet::storage]
    pub(super) type Controller<T: Config> = StorageValue<_, T::AccountId>;

    /// Details of bridge transactions identified with pairs of the recipient account and the
    /// bridge transaction nonce.
    #[pallet::storage]
    pub(super) type BridgeTxDetails<T: Config> = StorageDoubleMap<
        _,
        Blake2_128Concat,
        T::AccountId,
        Blake2_128Concat,
        u32,
        BridgeTxDetail<BlockNumberFor<T>>,
        ValueQuery,
    >;

    /// The admin key.
    #[pallet::storage]
    pub(super) type Admin<T: Config> = StorageValue<_, T::AccountId>;

    /// Whether or not the bridge operation is frozen.
    #[pallet::storage]
    pub(super) type Frozen<T: Config> = StorageValue<_, bool, ValueQuery>;

    /// Freeze bridge admins.  These accounts can only freeze the bridge.
    #[pallet::storage]
    pub(super) type FreezeAdmins<T: Config> =
        StorageMap<_, Blake2_128Concat, T::AccountId, bool, ValueQuery>;

    /// The bridge transaction timelock period, in blocks, since the acceptance of the
    /// transaction proposal during which the admin key can freeze the transaction.
    #[pallet::storage]
    pub(super) type Timelock<T: Config> = StorageValue<_, BlockNumberFor<T>, ValueQuery>;

    /// The maximum number of bridged POLYX per identity within a set interval of
    /// blocks. Fields: POLYX amount and the block interval duration.
    #[pallet::storage]
    pub(super) type BridgeLimit<T: Config> =
        StorageValue<_, (Balance, BlockNumberFor<T>), ValueQuery>;

    /// Amount of POLYX bridged by the identity in last block interval. Fields: the bridged
    /// amount and the last interval number.
    #[pallet::storage]
    pub(super) type PolyxBridged<T: Config> =
        StorageMap<_, Identity, IdentityId, (Balance, BlockNumberFor<T>), ValueQuery>;

    /// Identities not constrained by the bridge limit.
    #[pallet::storage]
    pub(super) type BridgeLimitExempted<T: Config> =
        StorageMap<_, Twox64Concat, IdentityId, bool, ValueQuery>;

    /// Storage version.
    #[pallet::storage]
    pub(super) type StorageVersion<T: Config> = StorageValue<_, Version, ValueQuery>;

    #[pallet::genesis_config]
    #[derive(frame_support::DefaultNoBound)]
    pub struct GenesisConfig<T> {
        #[serde(skip)]
        pub _config: sp_std::marker::PhantomData<T>,
    }

    #[pallet::genesis_build]
    impl<T: Config> BuildGenesisConfig for GenesisConfig<T> {
        fn build(&self) {
            StorageVersion::<T>::put(Version::new(0));
        }
    }

    #[pallet::call]
    impl<T: Config> Pallet<T> {}
}
