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

use super::*;
use crate::benchmarking::owned_ticker;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use polymesh_common_utilities::{benchs::AccountIdOf, TestUtilsFn};

const CP_BASE: u64 = 2000;

fn init<T: Config + TestUtilsFn<AccountIdOf<T>>>() -> (RawOrigin<T::AccountId>, Ticker) {
    <pallet_timestamp::Now<T>>::set(1000u32.into());
    let (owner, ticker) = owned_ticker::<T>();
    (owner.origin(), ticker)
}

fn init_with_existing<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    existing: u64,
) -> (RawOrigin<T::AccountId>, Ticker) {
    let (owner, ticker) = init::<T>();

    for n in 0..existing {
        let schedule = ScheduleCheckpoints::new(CP_BASE + n);
        Module::<T>::create_schedule(owner.clone().into(), ticker, schedule).unwrap();
    }

    (owner, ticker)
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

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
        let max = Module::<T>::schedules_max_complexity();
        let schedule = ScheduleCheckpoints::new_checkpoints(
            (0..max).into_iter().map(|n| CP_BASE + n).collect()
        );

        // Must fit in the max complexity.
        Module::<T>::set_schedules_max_complexity(
            RawOrigin::Root.into(),
            10 * max
        ).unwrap();

        let (owner, ticker) = init_with_existing::<T>(max);
    }: _(owner, ticker, schedule)
    verify {
        assert_eq!(Module::<T>::schedule_id_sequence(ticker), ScheduleId(max + 1))
    }

    remove_schedule {
        let max = Module::<T>::schedules_max_complexity();

        let id = ScheduleId(max);
        let (owner, ticker) = init_with_existing::<T>(max);
    }: _(owner, ticker, id)
    verify {
        assert_eq!(Module::<T>::scheduled_checkpoints(ticker, id), None);
    }
}
