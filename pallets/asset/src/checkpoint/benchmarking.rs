// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use super::*;
use crate::benchmarking::owned_ticker;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use polymesh_primitives::calendar::CalendarUnit;

const MAX_STORED_SCHEDULES: u32 = 1000;

const SCHEDULE: ScheduleSpec = ScheduleSpec {
    // Not entirely clear this is more expensive, but probably is.
    // In either case, this triggers `start == now`, which is definitely more expensive.
    start: None,
    // Forces us into the most amount of branches.
    remaining: 2,
    period: CalendarPeriod {
        // We want the schedule to be recurring as it triggers more branches.
        amount: 5,
        // Months have the most complicated "next at" calculation, so we use months.
        unit: CalendarUnit::Month,
    },
};

fn init<T: Trait>() -> (RawOrigin<T::AccountId>, Ticker) {
    <pallet_timestamp::Now<T>>::set(1000.into());
    let (owner, ticker) = owned_ticker::<T>();
    (owner.origin(), ticker)
}

fn init_with_existing<T: Trait>(existing: u32) -> (RawOrigin<T::AccountId>, Ticker) {
    let (owner, ticker) = init::<T>();

    // Ensure max compexity admits `existing + 1` schedules.
    let max = SCHEDULE.period.complexity() * (existing as u64 + 1);
    Module::<T>::set_schedules_max_complexity(RawOrigin::Root.into(), max).unwrap();

    // First create some schedules. To make sorting more expensive, we'll make em all equal.
    for _ in 0..existing {
        Module::<T>::create_schedule(owner.clone().into(), ticker, SCHEDULE).unwrap();
    }

    (owner, ticker)
}

benchmarks! {
    _ {}

    set_schedules_max_complexity {}: _(RawOrigin::Root, 7)
    verify {
        assert_eq!(Module::<T>::schedules_max_complexity(), 7)
    }

    create_checkpoint {
        let (owner, ticker) = init::<T>();
    }: _(owner, ticker)
    verify {
        assert_eq!(Module::<T>::checkpoint_id_sequence(ticker), CheckpointId(1))
    }

    create_schedule {
        // Stored schedules before creating this one.
        let s in 0 .. MAX_STORED_SCHEDULES;

        let (owner, ticker) = init_with_existing::<T>(s);
    }: _(owner, ticker, SCHEDULE)
    verify {
        assert_eq!(Module::<T>::schedule_id_sequence(ticker), ScheduleId(s as u64 + 1))
    }

    remove_schedule {
        // Stored schedules before creating this one.
        let s in 1 .. MAX_STORED_SCHEDULES;

        let id = ScheduleId(s as u64);
        let (owner, ticker) = init_with_existing::<T>(s);
    }: _(owner, ticker, id)
    verify {
        assert!(Module::<T>::schedules(ticker).iter().all(|s| s.id != id));
    }
}
