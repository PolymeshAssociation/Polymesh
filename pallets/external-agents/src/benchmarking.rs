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

use crate::*;
use frame_benchmarking::benchmarks;
use polymesh_common_utilities::benchs::{create_and_issue_sample_asset, user, AccountIdOf, User};
use polymesh_common_utilities::traits::asset::Config as Asset;
use polymesh_common_utilities::TestUtilsFn;
use polymesh_primitives::{AuthorizationData, ExtrinsicPermissions, PalletName, PalletPermissions};
use sp_std::prelude::*;

pub(crate) const SEED: u32 = 0;
const MAX_PALLETS: u32 = 19;

fn setup<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, AssetId) {
    let owner = user("owner", SEED);
    let asset_id = create_and_issue_sample_asset::<T>(&owner, true, None, b"SampleAsset", false);
    (owner, asset_id)
}

fn perms(n: u32) -> ExtrinsicPermissions {
    ExtrinsicPermissions::these(
        (0..=n as u64).map(|w| (PalletName::generate(w), PalletPermissions::whole())),
    )
}

fn add_auth<T: Asset + TestUtilsFn<AccountIdOf<T>>>(
    owner: &User<T>,
    asset_id: AssetId,
) -> (User<T>, u64) {
    let other = user("other", SEED);
    let auth_id = pallet_identity::Module::<T>::add_auth(
        owner.did(),
        other.did().into(),
        AuthorizationData::BecomeAgent(asset_id, AgentGroup::Full),
        None,
    )
    .unwrap();
    (other, auth_id)
}

fn setup_removal<T: Asset + TestUtilsFn<AccountIdOf<T>>>() -> (User<T>, User<T>, AssetId) {
    let (owner, asset_id) = setup::<T>();
    let (other, auth_id) = add_auth::<T>(&owner, asset_id);
    Module::<T>::accept_become_agent(other.origin().into(), auth_id).unwrap();
    (owner, other, asset_id)
}

fn custom_group<T: Config>(owner: User<T>, asset_id: AssetId) {
    Module::<T>::create_group(owner.origin().into(), asset_id, <_>::default()).unwrap();
}

benchmarks! {
    where_clause { where T: Asset, T: TestUtilsFn<AccountIdOf<T>> }

    create_group {
        let p in 0..MAX_PALLETS;

        let perms = perms(p);
        let (owner, asset_id) = setup::<T>();
        assert_eq!(AGId(0), AGIdSequence::get(asset_id));
    }: _(owner.origin, asset_id, perms)
    verify {
        assert_eq!(AGId(1), AGIdSequence::get(asset_id));
    }

    set_group_permissions {
        let p in 0..MAX_PALLETS;

        let (owner, asset_id) = setup::<T>();
        custom_group(owner.clone(), asset_id);

        let perms = perms(p);
        let perms2 = perms.clone();
    }: _(owner.origin(), asset_id, AGId(1), perms)
    verify {
        assert_eq!(Some(perms2), GroupPermissions::get(asset_id, AGId(1)));
    }

    remove_agent {
        let (owner, other, asset_id) = setup_removal::<T>();
    }: _(owner.origin(), asset_id, other.did())
    verify {
        assert_eq!(None, GroupOfAgent::get(asset_id, other.did()));
    }

    abdicate {
        let (_, other, asset_id) = setup_removal::<T>();
    }: _(other.origin(), asset_id)
    verify {
        assert_eq!(None, GroupOfAgent::get(asset_id, other.did()));
    }

    change_group_custom {
        let (owner, other, asset_id) = setup_removal::<T>();
        custom_group(owner.clone(), asset_id);
        let group = AgentGroup::Custom(AGId(1));
    }: change_group(owner.origin(), asset_id, other.did(), group)
    verify {
        assert_eq!(Some(group), GroupOfAgent::get(asset_id, other.did()));
    }

    change_group_builtin {
        let (owner, other, asset_id) = setup_removal::<T>();
    }: change_group(owner.origin(), asset_id, other.did(), AgentGroup::ExceptMeta)
    verify {
        assert_eq!(Some(AgentGroup::ExceptMeta), GroupOfAgent::get(asset_id, other.did()));
    }

    accept_become_agent {
        let (owner, asset_id) = setup::<T>();
        let (other, auth_id) = add_auth::<T>(&owner, asset_id);
    }: _(other.origin(), auth_id)
    verify {
        assert!(GroupOfAgent::get(asset_id, other.did()).is_some());
    }

    create_group_and_add_auth {
        let p in 0..MAX_PALLETS;

        let perms = perms(p);
        let (owner, asset_id) = setup::<T>();
        assert_eq!(AGId(0), AGIdSequence::get(asset_id));
    }: _(owner.origin, asset_id, perms, owner.did(), None)
    verify {
        assert_eq!(AGId(1), AGIdSequence::get(asset_id));
    }

    create_and_change_custom_group {
        let p in 0..MAX_PALLETS;
        let perms = perms(p);
        let (owner, other, asset_id) = setup_removal::<T>();
        assert_eq!(AGId(0), AGIdSequence::get(asset_id));
    }: _(owner.origin, asset_id, perms, other.did())
    verify {
        assert_eq!(AGId(1), AGIdSequence::get(asset_id));
    }

}
