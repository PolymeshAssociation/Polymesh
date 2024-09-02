use frame_support::storage::migration::move_prefix;
use sp_runtime::runtime_logger::RuntimeLogger;

use super::*;

mod v0 {
    use super::*;

    decl_storage! {
        trait Store for Module<T: Config> as CorporateBallot {
            // The CAId type has changed.
            pub OldMetas get(fn metas):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Option<BallotMeta>;

            // The CAId type has changed.
            pub OldTimeRanges get(fn time_ranges):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Option<BallotTimeRange>;

            // The CAId type has changed.
            pub OldMotionNumChoices get(fn motion_choices):
                map hasher(blake2_128_concat) crate::migrations::v0::CAId => Vec<u16>;

            // The CAId type has changed.
            pub OldRCV get(fn rcv): map hasher(blake2_128_concat) crate::migrations::v0::CAId => bool;

            // The CAId type has changed.
            pub OldResults get(fn results): map hasher(blake2_128_concat) crate::migrations::v0::CAId => Vec<Balance>;

            // The CAId type has changed.
            pub OldVotes get(fn votes):
                double_map hasher(blake2_128_concat) crate::migrations::v0::CAId, hasher(identity) IdentityId => Vec<BallotVote>;

        }
    }

    decl_module! {
        pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin { }
    }
}

pub(crate) fn migrate_to_v1<T: Config>() {
    RuntimeLogger::init();

    let mut count = 0;
    log::info!("Updating types for the Metas storage");
    move_prefix(&Metas::final_prefix(), &v0::OldMetas::final_prefix());
    v0::OldMetas::drain().for_each(|(ca_id, ballot)| {
        count += 1;
        Metas::insert(CAId::from(ca_id), ballot);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the TimeRanges storage");
    move_prefix(
        &TimeRanges::final_prefix(),
        &v0::OldTimeRanges::final_prefix(),
    );
    v0::OldTimeRanges::drain().for_each(|(ca_id, range)| {
        count += 1;
        TimeRanges::insert(CAId::from(ca_id), range);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the MotionNumChoices storage");
    move_prefix(
        &MotionNumChoices::final_prefix(),
        &v0::OldMotionNumChoices::final_prefix(),
    );
    v0::OldMotionNumChoices::drain().for_each(|(ca_id, choices)| {
        count += 1;
        MotionNumChoices::insert(CAId::from(ca_id), choices);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the RCV storage");
    move_prefix(&RCV::final_prefix(), &v0::OldRCV::final_prefix());
    v0::OldRCV::drain().for_each(|(ca_id, rcv)| {
        count += 1;
        RCV::insert(CAId::from(ca_id), rcv);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the Results storage");
    move_prefix(&Results::final_prefix(), &v0::OldResults::final_prefix());
    v0::OldResults::drain().for_each(|(ca_id, balances)| {
        count += 1;
        Results::insert(CAId::from(ca_id), balances);
    });
    log::info!("{:?} items migrated", count);

    let mut count = 0;
    log::info!("Updating types for the Votes storage");
    move_prefix(&Votes::final_prefix(), &v0::OldVotes::final_prefix());
    v0::OldVotes::drain().for_each(|(ca_id, did, vote)| {
        count += 1;
        Votes::insert(CAId::from(ca_id), did, vote);
    });
    log::info!("{:?} items migrated", count);
}
