use frame_support::{assert_noop, assert_ok, pallet_prelude::*, storage_alias};
use frame_system::RawOrigin;
use pallet_confidential_assets::{
    AssetDetails, AssetFrozen, AssetLeaf, Error, FreezeOrigin, Pallet,
};
use polymesh_dart::{
    AccountKeys, AccountRegistrationProof, AssetId, AssetKeys, AssetState,
    EncryptionKeyRegistrationProof, LeafIndex, PolymeshLimits,
};
use polymesh_primitives::{AccountId, IdentityId};
use rand::thread_rng;

use crate::{ExtBuilder, TestStorage as Runtime};

#[storage_alias]
type Details<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, AssetId, AssetDetails<T>, OptionQuery>;

#[storage_alias]
type Keys<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, AssetId, AssetKeys, OptionQuery>;

#[storage_alias]
type AssetLeaves<T: pallet_confidential_assets::Config> =
    StorageMap<Pallet<T>, Twox64Concat, LeafIndex, AssetLeaf, OptionQuery>;

fn with_asset(test: impl FnOnce(IdentityId, AssetState)) {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let owner = IdentityId::from(1u128);
        let auditor = AccountKeys::from_seed("FreezeAuditor").unwrap().enc.public;
        let state = AssetState::new::<()>(0, &[], &[auditor]).unwrap();
        Details::<Runtime>::insert(
            0,
            AssetDetails::<Runtime> {
                owner_did: owner,
                total_supply: 0,
                data: Default::default(),
            },
        );
        Keys::<Runtime>::insert(0, &state.keys);
        test(owner, state);
    });
}

#[test]
fn confidential_assets_registration_proofs_enforce_runtime_key_limit() {
    ExtBuilder::default().build().execute_with(|| {
        let account = AccountId::new([71; 32]);
        let did =
            pallet_identity::Pallet::<Runtime>::testing_register_did(account.clone()).unwrap();
        let did_bytes: [u8; 32] = did.into();
        let accounts = (0..21)
            .map(|index| AccountKeys::from_seed(&format!("RegistrationLimit{index}")).unwrap())
            .collect::<Vec<_>>();
        let mut rng = thread_rng();

        let account_proof =
            AccountRegistrationProof::<PolymeshLimits>::new(&mut rng, &accounts, &did_bytes)
                .unwrap();
        assert_eq!(account_proof.len(), 21);
        assert_noop!(
            Pallet::<Runtime>::register_accounts(
                RawOrigin::Signed(account.clone()).into(),
                account_proof,
            ),
            Error::<Runtime>::TooManyProofs
        );

        let encryption_keys = accounts
            .iter()
            .map(|account| account.enc.clone())
            .collect::<Vec<_>>();
        let encryption_proof = EncryptionKeyRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &encryption_keys,
            &did_bytes,
        )
        .unwrap();
        assert_eq!(encryption_proof.len(), 21);
        assert_noop!(
            Pallet::<Runtime>::register_encryption_keys(
                RawOrigin::Signed(account).into(),
                encryption_proof,
            ),
            Error::<Runtime>::TooManyProofs
        );
    });
}

#[test]
fn confidential_assets_freeze_authority_and_leaf() {
    with_asset(|owner, mut state| {
        let unfrozen_leaf = state.commitment().unwrap();
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Issuer)
        );
        state.frozen = true;
        let frozen_leaf = state.commitment().unwrap();
        assert_ne!(frozen_leaf, unfrozen_leaf);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(frozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            false
        ));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(unfrozen_leaf)
        );

        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Root)
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, false),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(frozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            false
        ));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(unfrozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Root)
        );
    });
}

#[test]
fn confidential_assets_leaf_update_events() {
    with_asset(|owner, mut state| {
        for frozen in [true, false] {
            frame_system::Pallet::<Runtime>::reset_events();
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
                Some(owner),
                state.asset_id,
                frozen
            ));
            state.frozen = frozen;
            let expected: <Runtime as frame_system::Config>::RuntimeEvent =
                pallet_confidential_assets::Event::<Runtime>::AssetStateLeafUpdated {
                    leaf_index: LeafIndex::from(state.asset_id),
                    asset_leaf: state.commitment().unwrap(),
                }
                .into();
            let events = frame_system::Pallet::<Runtime>::events();
            assert_eq!(
                events
                    .iter()
                    .filter(|record| record.event == expected)
                    .count(),
                1
            );
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
                Some(owner),
                state.asset_id,
                frozen
            ));
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
fn confidential_assets_frozen_revealed_lookup_rejected() {
    with_asset(|owner, state| {
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(Pallet::<Runtime>::get_asset_keys_lookup(asset_ids).is_ok());
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(matches!(
            Pallet::<Runtime>::get_asset_keys_lookup(asset_ids),
            Err(Error::<Runtime>::AssetIsFrozen)
        ));
    });
}
