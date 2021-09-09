use codec::{Decode, Encode};
use frame_support::decl_event;
use frame_support::weights::Weight;
use polymesh_primitives::calendar::{CheckpointId, CheckpointSchedule};
use polymesh_primitives::{Balance, EventDid, IdentityId, Moment, Ticker};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};

/// ID of a `StoredSchedule`.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScheduleId(pub u64);

/// One or more scheduled checkpoints in the future.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Encode, Decode, Copy, Clone, Debug, PartialEq, Eq)]
pub struct StoredSchedule {
    /// A series of checkpoints in the future defined by the schedule.
    pub schedule: CheckpointSchedule,
    /// The ID of the schedule itself.
    /// Not to be confused for a checkpoint's ID.
    pub id: ScheduleId,
    /// When the next checkpoint is due to be created.
    /// Used as a cache for more efficient sorting.
    pub at: Moment,
    /// Number of CPs that the schedule may create
    /// before it is evicted from the schedule set.
    ///
    /// The value `0` is special cased to mean infinity.
    pub remaining: u32,
}

pub trait WeightInfo {
    fn create_checkpoint() -> Weight;
    fn set_schedules_max_complexity() -> Weight;
    fn create_schedule(existing_schedules: u32) -> Weight;
    fn remove_schedule(existing_schedules: u32) -> Weight;
}

decl_event! {
    pub enum Event {
        /// A checkpoint was created.
        ///
        /// (caller DID, ticker, checkpoint ID, total supply, checkpoint timestamp)
        CheckpointCreated(Option<EventDid>, Ticker, CheckpointId, Balance, Moment),

        /// The maximum complexity for an arbitrary ticker's schedule set was changed.
        ///
        /// (GC DID, the new maximum)
        MaximumSchedulesComplexityChanged(IdentityId, u64),

        /// A checkpoint schedule was created.
        ///
        /// (caller DID, ticker, schedule)
        ScheduleCreated(EventDid, Ticker, StoredSchedule),

        /// A checkpoint schedule was removed.
        ///
        /// (caller DID, ticker, schedule)
        ScheduleRemoved(IdentityId, Ticker, StoredSchedule),
    }
}
