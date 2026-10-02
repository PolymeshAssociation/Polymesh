use frame_support::{assert_noop, assert_ok, pallet_prelude::*, storage_alias};
use frame_system::RawOrigin;
use pallet_confidential_assets::{AssetDetails, AssetFrozen, AssetLeaf, Error, FreezeOrigin, Pallet};
use polymesh_dart::{AccountKeys, AssetId, AssetKeys, AssetState, LeafIndex};
use polymesh_primitives::IdentityId;
use polymesh_runtime_develop::Runtime;
use sp_runtime::BuildStorage;

#[storage_alias]
type Details<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, AssetId, AssetDetails<T>, OptionQuery>;

#[storage_alias]
type Keys<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, AssetId, AssetKeys, OptionQuery>;

#[storage_alias]
type AssetLeaves<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, LeafIndex, AssetLeaf, OptionQuery>;

#[storage_alias]
type CurrentWorkerSessionId<T: pallet_confidential_assets::Config> =
    StorageValue<Pallet<T>, u32, OptionQuery>;

fn with_asset(test: impl FnOnce(IdentityId, AssetState)) {
    let storage = frame_system::GenesisConfig::<Runtime>::default().build_storage().unwrap();
    sp_io::TestExternalities::new(storage).execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let owner = IdentityId::from(1u128);
        let auditor = AccountKeys::from_seed("FreezeAuditor").unwrap().enc.public;
        let state = AssetState::new::<()>(0, &[], &[auditor]).unwrap();
        Details::<Runtime>::insert(0, AssetDetails::<Runtime> {
            owner_did: owner,
            total_supply: 0,
            data: Default::default(),
        });
        Keys::<Runtime>::insert(0, &state.keys);
        CurrentWorkerSessionId::<Runtime>::put(0);
        test(owner, state);
    });
}

#[test]
fn confidential_assets_freeze_authority_and_leaf() {
    with_asset(|owner, mut state| {
        let unfrozen_leaf = state.commitment().unwrap();
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), Some(FreezeOrigin::Issuer));
        state.frozen = true;
        let frozen_leaf = state.commitment().unwrap();
        assert_ne!(frozen_leaf, unfrozen_leaf);
        assert_eq!(AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)), Some(frozen_leaf));
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, false));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)), Some(unfrozen_leaf));

        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true));
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, true));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), Some(FreezeOrigin::Root));
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, false),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, true));
        assert_eq!(AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)), Some(frozen_leaf));
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, false));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)), Some(unfrozen_leaf));
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, true));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), Some(FreezeOrigin::Root));
    });
}

#[test]
fn confidential_assets_leaf_update_events() {
    with_asset(|owner, mut state| {
        for frozen in [true, false] {
            frame_system::Pallet::<Runtime>::reset_events();
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, frozen));
            state.frozen = frozen;
            let expected: <Runtime as frame_system::Config>::RuntimeEvent =
                pallet_confidential_assets::Event::<Runtime>::AssetStateLeafUpdated {
                    leaf_index: LeafIndex::from(state.asset_id),
                    asset_leaf: state.commitment().unwrap(),
                }.into();
            let events = frame_system::Pallet::<Runtime>::events();
            assert_eq!(events.iter().filter(|record| record.event == expected).count(), 1);
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, frozen));
            assert_eq!(frame_system::Pallet::<Runtime>::events(), events);
        }
    });
}

#[test]
fn confidential_assets_freeze_rejects_non_owner_and_missing_asset() {
    with_asset(|_, state| {
        let other_did = IdentityId::from(2u128);
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(other_did), state.asset_id, true),
            Error::<Runtime>::NotAssetOwner
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(other_did), state.asset_id, false),
            Error::<Runtime>::NotAssetOwner
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), 1, true),
            Error::<Runtime>::AssetMissing
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), 1, false),
            Error::<Runtime>::AssetMissing
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::None.into(), state.asset_id, true),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn confidential_assets_freeze_rolls_back_failed_leaf_update() {
    with_asset(|_, state| {
        CurrentWorkerSessionId::<Runtime>::kill();
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, true),
            Error::<Runtime>::NoCurrentWorkerSession
        );
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        CurrentWorkerSessionId::<Runtime>::put(0);
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, true));
        let frozen_leaf = AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id));
        CurrentWorkerSessionId::<Runtime>::kill();
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), state.asset_id, false),
            Error::<Runtime>::NoCurrentWorkerSession
        );
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), Some(FreezeOrigin::Root));
        assert_eq!(AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)), frozen_leaf);
    });
}

#[test]
fn confidential_assets_frozen_revealed_lookup_rejected() {
    with_asset(|owner, state| {
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(Pallet::<Runtime>::get_asset_keys_lookup(asset_ids).is_ok());
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true));
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(matches!(
            Pallet::<Runtime>::get_asset_keys_lookup(asset_ids),
            Err(Error::<Runtime>::AssetIsFrozen)
        ));
    });
}