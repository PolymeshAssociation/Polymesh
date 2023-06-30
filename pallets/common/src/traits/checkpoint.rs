use codec::{Decode, Encode};
use frame_support::decl_event;
use frame_support::weights::Weight;
use polymesh_primitives::calendar::{CalendarPeriod, CheckpointSchedule};
use polymesh_primitives::{
    asset::CheckpointId, impl_checked_inc, Balance, IdentityId, Moment, Ticker,
};
use scale_info::TypeInfo;
use sp_std::collections::btree_map::BTreeMap;
use sp_std::collections::btree_set::BTreeSet;
use sp_std::prelude::Vec;

/// ID of a `StoredSchedule`.
#[derive(Encode, Decode, TypeInfo)]
#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct ScheduleId(pub u64);
impl_checked_inc!(ScheduleId);

/// One or more scheduled checkpoints in the future.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, Default, PartialEq, Eq)]
pub struct ScheduleCheckpoints {
    /// The timestamps of the scheduled checkpoints.
    pub pending: BTreeSet<Moment>,
}

impl ScheduleCheckpoints {
    pub fn new(at: Moment) -> Self {
        Self {
            pending: [at].into(),
        }
    }

    pub fn from_period(start: Moment, period: CalendarPeriod, remaining: u32) -> Self {
        Self::from_old(CheckpointSchedule { start, period }, remaining)
    }

    pub fn from_old(old: CheckpointSchedule, remaining: u32) -> Self {
        let remaining = remaining.min(10);
        let mut now = old.start;
        Self {
            pending: (0..remaining)
                .into_iter()
                .filter_map(|n| {
                    if n == 0 {
                        return Some(now);
                    }
                    let at = old.next_checkpoint(now);
                    if let Some(at) = at {
                        now = at;
                    }
                    at
                })
                .collect(),
        }
    }

    pub fn new_checkpoints(pending: BTreeSet<Moment>) -> Self {
        Self { pending }
    }

    pub fn is_empty(&self) -> bool {
        self.pending.is_empty()
    }

    pub fn len(&self) -> usize {
        self.pending.len()
    }

    pub fn remove_expired(&mut self, now: Moment) -> Vec<Moment> {
        let mut removed = Vec::new();
        self.pending.retain(|&next| {
            if now >= next {
                removed.push(next);
                // Expired.
                false
            } else {
                // Still pending.
                true
            }
        });
        removed
    }

    pub fn ensure_no_expired(&self, now: Moment) -> bool {
        self.pending.iter().all(|&p| p >= now)
    }

    pub fn next(&self) -> Option<Moment> {
        self.pending.iter().next().copied()
    }
}

/// Create a schedule due exactly at the provided `start: Moment` time.
impl From<Moment> for ScheduleCheckpoints {
    fn from(start: Moment) -> Self {
        Self::new(start)
    }
}

/// Track the next checkpoint for each schedule.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub struct NextCheckpoints {
    /// A cache of the closest checkpoint from all schedules.
    /// This is to quickly check if a checkpoint needs to be created.
    pub next_at: Moment,
    /// Total pending checkpoints.
    pub total_pending: u64,
    /// The `next` checkpoint from each active schedule.
    pub schedules: BTreeMap<ScheduleId, Moment>,
}

impl Default for NextCheckpoints {
    fn default() -> Self {
        Self {
            next_at: Moment::MAX,
            total_pending: 0,
            schedules: Default::default(),
        }
    }
}

impl NextCheckpoints {
    pub fn is_empty(&self) -> bool {
        self.schedules.is_empty()
    }

    pub fn expired(&self, now: Moment) -> bool {
        now >= self.next_at
    }

    pub fn expired_schedules(&mut self, now: Moment) -> Vec<ScheduleId> {
        let mut expired = Vec::new();
        let mut min_next = Moment::MAX;
        // Remove expired schedules.
        self.schedules.retain(|&id, &mut next| {
            if now >= next {
                expired.push(id);
                // Expired.
                false
            } else {
                // Still pending.
                if next < min_next {
                    min_next = next;
                }
                true
            }
        });
        // Update `next_at`.
        self.next_at = min_next;
        expired
    }

    pub fn inc_total_pending(&mut self, num: u64) {
        self.total_pending = self.total_pending.saturating_add(num);
    }

    pub fn dec_total_pending(&mut self, num: u64) {
        self.total_pending = self.total_pending.saturating_sub(num);
    }

    pub fn recal_next(&mut self) {
        let mut min_next = Moment::MAX;
        for &next in self.schedules.values() {
            if next < min_next {
                min_next = next;
            }
        }
        self.next_at = min_next;
    }

    pub fn add_schedule_next(&mut self, id: ScheduleId, cp: Moment) {
        self.schedules.insert(id, cp);
        if cp < self.next_at {
            self.next_at = cp;
        }
    }

    pub fn remove_schedule(&mut self, id: ScheduleId) {
        if Some(self.next_at) == self.schedules.remove(&id) {
            // Need to recalculate the next cp.
            self.recal_next();
        }
    }
}

pub trait WeightInfo {
    fn create_checkpoint() -> Weight;
    fn set_schedules_max_complexity() -> Weight;
    fn create_schedule() -> Weight;
    fn remove_schedule() -> Weight;
}

decl_event! {
    pub enum Event {
        /// A checkpoint was created.
        ///
        /// (caller DID, ticker, checkpoint ID, total supply, checkpoint timestamp)
        CheckpointCreated(Option<IdentityId>, Ticker, CheckpointId, Balance, Moment),

        /// The maximum complexity for an arbitrary ticker's schedule set was changed.
        ///
        /// (GC DID, the new maximum)
        MaximumSchedulesComplexityChanged(IdentityId, u64),

        /// A checkpoint schedule was created.
        ///
        /// (caller DID, ticker, schedule id, schedule)
        ScheduleCreated(IdentityId, Ticker, ScheduleId, ScheduleCheckpoints),

        /// A checkpoint schedule was removed.
        ///
        /// (caller DID, ticker, schedule id, schedule)
        ScheduleRemoved(IdentityId, Ticker, ScheduleId, ScheduleCheckpoints),
    }
}
