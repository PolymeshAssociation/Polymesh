use sp_runtime::runtime_logger::RuntimeLogger;
use sp_std::collections::btree_map::BTreeMap;

use super::*;

mod v2 {
    use scale_info::TypeInfo;

    use super::*;
    use polymesh_primitives::{NFTId, Ticker};

    #[derive(Clone, Debug, Decode, Encode, Eq, PartialEq, TypeInfo)]
    pub enum Leg {
        Fungible {
            sender: PortfolioId,
            receiver: PortfolioId,
            ticker: Ticker,
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

    #[derive(Clone, Debug, Decode, Default, Encode, Eq, PartialEq, TypeInfo)]
    pub struct NFTs {
        pub ticker: Ticker,
        pub ids: Vec<NFTId>,
    }

    #[derive(Encode, Decode, TypeInfo)]
    #[derive(Default, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
    pub struct Instruction<Moment, BlockNumber> {
        pub instruction_id: InstructionId,
        pub venue_id: VenueId,
        pub settlement_type: SettlementType<BlockNumber>,
        pub created_at: Option<Moment>,
        pub trade_date: Option<Moment>,
        pub value_date: Option<Moment>,
    }

    decl_storage! {
        trait Store for Module<T: Config> as Settlement {
            // This storage changed the Ticker key to AssetID.
            pub(crate) VenueFiltering get(fn venue_filtering):
                map hasher(blake2_128_concat) Ticker => bool;

            // This storage changed the Ticker key to AssetID.
            pub(crate) VenueAllowList get(fn venue_allow_list):
                double_map hasher(blake2_128_concat) Ticker, hasher(twox_64_concat) VenueId => bool;

            // This storage changed the Leg type.
            pub(crate) InstructionLegs get(fn instruction_legs):
                double_map hasher(twox_64_concat) InstructionId, hasher(twox_64_concat) LegId => Option<Leg>;

            pub(crate) InstructionDetails get(fn instruction_details):
                map hasher(twox_64_concat) InstructionId => Instruction<T::Moment, T::BlockNumber>;

        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

impl From<v2::NFTs> for NFTs {
    fn from(v2_nfts: v2::NFTs) -> NFTs {
        NFTs::new_unverified(v2_nfts.ticker.into(), v2_nfts.ids)
    }
}

#[rustfmt::skip]
impl From<v2::Leg> for Leg {
    fn from(v2_leg: v2::Leg) -> Leg {
        match v2_leg {
            v2::Leg::Fungible { sender, receiver, ticker, amount } => {
                Leg::Fungible {
                    sender,
                    receiver,
                    asset_id: ticker.into(),
                    amount,
                }
            },
            v2::Leg::NonFungible { sender, receiver, nfts } => {
                Leg::NonFungible {
                    sender,
                    receiver,
                    nfts: nfts.into(),
                }
            },
            v2::Leg::OffChain { sender_identity, receiver_identity, ticker, amount } => {
                Leg::OffChain {
                    sender_identity,
                    receiver_identity,
                    ticker,
                    amount,
                }
            }
        }
    }
}

impl<T, S> From<v2::Instruction<T, S>> for Instruction<T, S> {
    fn from(v2_instruction: v2::Instruction<T, S>) -> Instruction<T, S> {
        Instruction {
            instruction_id: v2_instruction.instruction_id,
            venue_id: Some(v2_instruction.venue_id),
            settlement_type: v2_instruction.settlement_type,
            created_at: v2_instruction.created_at,
            trade_date: v2_instruction.trade_date,
            value_date: v2_instruction.value_date,
        }
    }
}

pub(crate) fn migrate_to_v3<T: Config>() {
    RuntimeLogger::init();
    let mut ticker_to_asset_id = BTreeMap::new();

    // Removes all elements in the old storage and inserts it in the new storage

    let mut count = 0;
    log::info!("Updating types for the VenueFiltering storage");
    v2::VenueFiltering::drain().for_each(|(ticker, v)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        VenueFiltering::insert(asset_id, v);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the VenueAllowList storage");
    v2::VenueAllowList::drain().for_each(|(ticker, id, v)| {
        count += 1;
        let asset_id = ticker_to_asset_id
            .entry(ticker)
            .or_insert(AssetID::from(ticker));
        VenueAllowList::insert(asset_id, id, v);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the InstructionLegs storage");
    v2::InstructionLegs::drain().for_each(|(instruction_id, leg_id, leg)| {
        count += 1;
        InstructionLegs::insert(instruction_id, leg_id, Leg::from(leg));
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the InstructionDetails storage");
    v2::InstructionDetails::<T>::drain().for_each(|(id, inst)| {
        count += 1;
        InstructionDetails::<T>::insert(id, Instruction::from(inst));
    });
    log::info!("{:?} items migrated", count);
}
