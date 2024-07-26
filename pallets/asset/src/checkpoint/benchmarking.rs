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

use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;

use polymesh_common_utilities::{
    benchs::{AccountIdOf, User, UserBuilder},
    TestUtilsFn,
};

use super::*;
use crate::benchmarking::create_sample_asset;

const CP_BASE: u64 = 2000;

fn init_with_existing<T: Config>(asset_owner: &User<T>, existing: u64) -> AssetID {
    let asset_id = create_sample_asset::<T>(&asset_owner, true);

    for n in 0..existing {
        let schedule = ScheduleCheckpoints::new(CP_BASE + n);
        Module::<T>::create_schedule(asset_owner.origin.clone().into(), asset_id, schedule)
            .unwrap();
    }

    asset_id
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    set_schedules_max_complexity {}: _(RawOrigin::Root, 7)
    verify {
        assert_eq!(Module::<T>::schedules_max_complexity(), 7)
    }

    create_checkpoint {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let asset_id = create_sample_asset::<T>(&alice, true);
    }: _(alice.origin, asset_id)
    verify {
        assert_eq!(Module::<T>::checkpoint_id_sequence(asset_id), CheckpointId(1))
    }

    create_schedule {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");

        let max = Module::<T>::schedules_max_complexity();
        let schedule = ScheduleCheckpoints::new_checkpoints(
            (0..max).into_iter().map(|n| CP_BASE + n).collect()
        );

        // Must fit in the max complexity.
        Module::<T>::set_schedules_max_complexity(
            RawOrigin::Root.into(),
            10 * max
        ).unwrap();

        let asset_id = init_with_existing::<T>(&alice, max);
    }: _(alice.origin, asset_id, schedule)
    verify {
        assert_eq!(Module::<T>::schedule_id_sequence(asset_id), ScheduleId(max + 1))
    }

    remove_schedule {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let max = Module::<T>::schedules_max_complexity();

        let id = ScheduleId(max);
        let asset_id = init_with_existing::<T>(&alice, max);
    }: _(alice.origin, asset_id, id)
    verify {
        assert_eq!(Module::<T>::scheduled_checkpoints(asset_id, id), None);
    }
}
