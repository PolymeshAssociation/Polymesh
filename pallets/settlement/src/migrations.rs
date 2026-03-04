use frame_support::pallet_prelude::*;

use polymesh_primitives::asset::AssetHolder;
use polymesh_primitives::settlement::Leg;

use crate::Config;

pub mod v3 {
    use frame_support::pallet_prelude::*;

    use serde::{Deserialize, Serialize};

    use polymesh_primitives::asset::AssetId;
    use polymesh_primitives::settlement::{AffirmationStatus, InstructionId, LegId};
    use polymesh_primitives::{Balance, IdentityId, NFTs, PortfolioId, Ticker};

    use crate::{Config, Pallet};

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
    pub type AffirmsReceived<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        InstructionId,
        Twox64Concat,
        PortfolioId,
        AffirmationStatus,
        ValueQuery,
    >;

    #[frame_support::storage_alias]
    pub type UserAffirmations<T: Config> = StorageDoubleMap<
        Pallet<T>,
        Twox64Concat,
        PortfolioId,
        Twox64Concat,
        InstructionId,
        AffirmationStatus,
        ValueQuery,
    >;

    #[frame_support::storage_alias]
    pub type InstructionLegs<T: Config> = StorageDoubleMap<
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

    for (instruction_id, portfolio_id, status) in v3::AffirmsReceived::<T>::drain() {
        crate::AffirmsReceived::<T>::insert(
            instruction_id,
            AssetHolder::Portfolio(portfolio_id),
            status,
        );
        count += 1;
    }

    T::DbWeight::get().reads_writes(count, count)
}

pub fn migrate_user_affirmations<T: Config>() -> Weight {
    let mut count = 0;

    for (portfolio_id, instruction_id, status) in v3::UserAffirmations::<T>::drain() {
        crate::UserAffirmations::<T>::insert(
            AssetHolder::Portfolio(portfolio_id),
            instruction_id,
            status,
        );
        count += 1;
    }

    T::DbWeight::get().reads_writes(count, count)
}

pub fn migrate_instruction_legs<T: Config>() -> Weight {
    let mut count = 0;

    for (instruction_id, leg_id, leg) in v3::InstructionLegs::<T>::drain() {
        crate::InstructionLegs::<T>::insert(instruction_id, leg_id, Leg::from(leg));
        count += 1;
    }

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
                sender: AssetHolder::Portfolio(sender),
                receiver: AssetHolder::Portfolio(receiver),
                asset_id,
                amount,
            },
            v3::Leg::NonFungible {
                sender,
                receiver,
                nfts,
            } => Leg::NonFungible {
                sender: AssetHolder::Portfolio(sender),
                receiver: AssetHolder::Portfolio(receiver),
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
