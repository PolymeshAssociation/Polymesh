use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v2 {
    use scale_info::TypeInfo;

    use super::*;

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
            pub(crate) InstructionDetails get(fn instruction_details):
                map hasher(twox_64_concat) InstructionId => Instruction<T::Moment, T::BlockNumber>;
        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
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

#[allow(dead_code)]
pub(crate) fn migrate_to_v3<T: Config>() {
    RuntimeLogger::init();

    let mut count = 0;
    log::info!("Updating types for the InstructionDetails storage");
    v2::InstructionDetails::<T>::drain().for_each(|(id, inst)| {
        count += 1;
        InstructionDetails::<T>::insert(id, Instruction::from(inst));
    });
    log::info!("{:?} items migrated", count);
}
