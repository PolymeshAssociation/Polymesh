use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v0 {
    use super::*;

    decl_storage! {
        trait Store for Module<T: Config> as CorporateBallot {
            // The CAId type has changed.
            pub Metas get(fn metas):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Option<BallotMeta>;

            // The CAId type has changed.
            pub TimeRanges get(fn time_ranges):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Option<BallotTimeRange>;

            // The CAId type has changed.
            pub MotionNumChoices get(fn motion_choices):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Vec<u16>;

            // The CAId type has changed.
            pub RCV get(fn rcv): map hasher(blake2_128_concat) crate::migrations::v0::CAId => bool;

            // The CAId type has changed.
            pub Results get(fn results): map hasher(blake2_128_concat) crate::migrations::v0::CAId => Vec<Balance>;

            // The CAId type has changed.
            pub Votes get(fn votes):
                double_map hasher(blake2_128_concat) crate::migrations::v0::CAId, hasher(identity) IdentityId => Vec<BallotVote>;

        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();

    log::info!("Updating types for the Metas storage");
    v0::Metas::drain().for_each(|(ca_id, ballot)| {
        Metas::insert(CAId::from(ca_id), ballot);
    });

    log::info!("Updating types for the TimeRanges storage");
    v0::TimeRanges::drain().for_each(|(ca_id, range)| {
        TimeRanges::insert(CAId::from(ca_id), range);
    });

    log::info!("Updating types for the MotionNumChoices storage");
    v0::MotionNumChoices::drain().for_each(|(ca_id, choices)| {
        MotionNumChoices::insert(CAId::from(ca_id), choices);
    });

    log::info!("Updating types for the RCV storage");
    v0::RCV::drain().for_each(|(ca_id, rcv)| {
        RCV::insert(CAId::from(ca_id), rcv);
    });

    log::info!("Updating types for the Results storage");
    v0::Results::drain().for_each(|(ca_id, balances)| {
        Results::insert(CAId::from(ca_id), balances);
    });

    log::info!("Updating types for the Votes storage");
    v0::Votes::drain().for_each(|(ca_id, did, vote)| {
        Votes::insert(CAId::from(ca_id), did, vote);
    });
}
