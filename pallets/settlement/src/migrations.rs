use frame_support::pallet_prelude::*;
use frame_support::storage::migration::move_prefix;
use frame_support::StoragePrefixedMap;

use polymesh_primitives::asset::AssetHolder;
use polymesh_primitives::settlement::Leg;
use polymesh_primitives::{PortfolioId, PortfolioKind};

use crate::Config;

pub mod v3 {
    use frame_support::pallet_prelude::*;

    use serde::{Deserialize, Serialize};

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::settlement::{AffirmationStatus, InstructionId, LegId};
    use polymesh_primitives::{AccountId, Balance, IdentityId, NFTs, PortfolioNumber, Ticker};

    use crate::{Config, Pallet};

    #[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
    #[derive(Clone, PartialEq, Eq)]
    #[derive(Serialize, Deserialize)]
    pub struct PortfolioId {
        pub did: IdentityId,
        pub kind: PortfolioKind,
    }

    #[derive(Encode, Decode, DecodeWithMemTracking, TypeInfo, MaxEncodedLen)]
    #[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
    pub enum PortfolioKind {
        Default,
        User(PortfolioNumber),
        AccountId(AccountId),
    }

    #[derive(Decode, DecodeWithMemTracking, Encode, Eq, PartialEq, TypeInfo)]
    #[derive(Deserialize, Serialize)]
    pub enum Leg {
        Fungible {
            sender: PortfolioId,
            receiver: PortfolioId,
            asset_id: AssetId,
            amount: Balance,
        },
        NonFungible {
            sender: PortfolioId,
            receiver: PortfolioId,
            nfts: NFTs,
        },
        OffChain {
            sender_identity: IdentityId,
            receiver_identity: IdentityId,
            ticker: Ticker,
            amount: Balance,
        },
    }

    #[frame_support::storage_alias]
    pub type OldAffirmsReceived<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        PortfolioId,
        AffirmationStatus,
        ValueQuery,
    >;

    #[frame_support::storage_alias]
    pub type OldUserAffirmations<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        PortfolioId,
        Twox64Concat,
        InstructionId,
        AffirmationStatus,
        ValueQuery,
    >;

    #[frame_support::storage_alias]
    pub type OldInstructionLegs<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        LegId,
        Leg,
        OptionQuery,
    >;
}

pub fn migrate_affirms_received<T: Config>() -> Weight {
    let mut count = 0;

    move_prefix(
        &crate::AffirmsReceived::<T>::final_prefix(),
        &v3::OldAffirmsReceived::<T>::final_prefix(),
    );

    for (instruction_id, portfolio_id, status) in v3::OldAffirmsReceived::<T>::drain() {
        crate::AffirmsReceived::<T>::insert(
            instruction_id,
            AssetHolder::from(portfolio_id),
            status,
        );
        count += 1;
    }

    T::DbWeight::get().reads_writes(count, count)
}

pub fn migrate_user_affirmations<T: Config>() -> Weight {
    let mut count = 0;

    move_prefix(
        &crate::UserAffirmations::<T>::final_prefix(),
        &v3::OldUserAffirmations::<T>::final_prefix(),
    );

    for (portfolio_id, instruction_id, status) in v3::OldUserAffirmations::<T>::drain() {
        crate::UserAffirmations::<T>::insert(
            AssetHolder::from(portfolio_id),
            instruction_id,
            status,
        );
        count += 1;
    }

    log::info!("UserAffirmations migrated: {count}");

    T::DbWeight::get().reads_writes(count, count)
}

pub fn migrate_instruction_legs<T: Config>() -> Weight {
    let mut count = 0;

    move_prefix(
        &crate::InstructionLegs::<T>::final_prefix(),
        &v3::OldInstructionLegs::<T>::final_prefix(),
    );

    for (instruction_id, leg_id, leg) in v3::OldInstructionLegs::<T>::drain() {
        crate::InstructionLegs::<T>::insert(instruction_id, leg_id, Leg::from(leg));
        count += 1;
    }

    log::info!("InstructionLegs migrated: {count}");

    T::DbWeight::get().reads_writes(count, count)
}

pub fn migrate_to_v4<T: Config>() -> Weight {
    migrate_affirms_received::<T>()
        .saturating_add(migrate_user_affirmations::<T>())
        .saturating_add(migrate_instruction_legs::<T>())
}

impl From<v3::Leg> for Leg {
    fn from(leg: v3::Leg) -> Self {
        match leg {
            v3::Leg::Fungible {
                sender,
                receiver,
                asset_id,
                amount,
            } => Leg::Fungible {
                sender: sender.into(),
                receiver: receiver.into(),
                asset_id,
                amount,
            },
            v3::Leg::NonFungible {
                sender,
                receiver,
                nfts,
            } => Leg::NonFungible {
                sender: sender.into(),
                receiver: receiver.into(),
                nfts,
            },
            v3::Leg::OffChain {
                sender_identity,
                receiver_identity,
                ticker,
                amount,
            } => Leg::OffChain {
                sender_identity,
                receiver_identity,
                ticker,
                amount,
            },
        }
    }
}

impl From<v3::PortfolioId> for AssetHolder {
    fn from(portfolio_id: v3::PortfolioId) -> Self {
        match portfolio_id.kind {
            v3::PortfolioKind::Default => AssetHolder::Portfolio(PortfolioId {
                did: portfolio_id.did,
                kind: PortfolioKind::Default,
            }),
            v3::PortfolioKind::User(portfolio_number) => AssetHolder::Portfolio(PortfolioId {
                did: portfolio_id.did,
                kind: PortfolioKind::User(portfolio_number),
            }),
            v3::PortfolioKind::AccountId(account_id) => AssetHolder::Account(account_id),
        }
    }
}
