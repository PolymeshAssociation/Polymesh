use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{DispatchResult, Perbill};

use polymesh_primitives::{AccountId, AuthorizationData, IdentityId, Permissions};

use crate::ext_builder::ExtBuilder;
use crate::storage::{add_secondary_key, TestStorage, User};

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;

#[test]
fn updating_controller() {
    let charlie = vec![Sr25519Keyring::Charlie.to_account_id()];
    ExtBuilder::default()
        .did_registrars(charlie)
        .build()
        .execute_with(|| {
            let alice: User = User::new(Sr25519Keyring::Alice);
            let eve: User = User::new_with(alice.did, Sr25519Keyring::Eve);

            add_secondary_key(alice.did, eve.acc());

            let auth_id = pallet_identity::Pallet::<TestStorage>::add_auth(
                alice.did,
                eve.signatory_acc(),
                AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
                None,
            )
            .unwrap();

            assert_ok!(
                pallet_validators::Pallet::<TestStorage>::add_permissioned_validator(
                    Origin::root(),
                    alice.did,
                    None
                )
            );

            assert_ok!(pallet_staking::Pallet::<TestStorage>::bond(
                alice.origin(),
                10_000_000,
                pallet_staking::RewardDestination::Account(alice.acc())
            ));

            assert_ok!(pallet_staking::Pallet::<TestStorage>::validate(
                Origin::signed(alice.acc()),
                pallet_staking::ValidatorPrefs::default()
            ));

            assert!(pallet_identity::Pallet::<TestStorage>::is_did_active(
                alice.did
            ));

            assert_ok!(
                pallet_validators::Pallet::<TestStorage>::remove_permissioned_validator(
                    Origin::root(),
                    alice.did,
                )
            );

            // Removing the permission chills the stash, releasing its ref count.
            assert!(!pallet_staking::Validators::<TestStorage>::contains_key(
                alice.acc()
            ));
            assert_eq!(
                pallet_identity::AccountKeyRefCount::<TestStorage>::get(alice.acc()),
                0
            );

            assert_ok!(
                pallet_identity::Pallet::<TestStorage>::rotate_primary_key_to_secondary(
                    eve.origin(),
                    auth_id
                )
            );
        });
}

/// Registers `key` as a secondary key of `did`, then bonds it and starts validating from it.
fn validate_from_secondary_key(did: IdentityId, key: Sr25519Keyring) -> DispatchResult {
    let user = User::new_with(did, key).balance(100_000_000);
    add_secondary_key(did, user.acc());
    assert_ok!(pallet_staking::Pallet::<TestStorage>::bond(
        user.origin(),
        10_000_000,
        pallet_staking::RewardDestination::Account(user.acc())
    ));
    pallet_staking::Pallet::<TestStorage>::validate(
        user.origin(),
        pallet_staking::ValidatorPrefs::default(),
    )
}

/// Permissions `did` to run `intended_count` validators.
fn permission(did: IdentityId, intended_count: u32) {
    assert_ok!(
        pallet_validators::Pallet::<TestStorage>::add_permissioned_validator(
            Origin::root(),
            did,
            Some(intended_count)
        )
    );
}

fn remove_permission(did: IdentityId) {
    assert_ok!(
        pallet_validators::Pallet::<TestStorage>::remove_permissioned_validator(
            Origin::root(),
            did,
        )
    );
}

fn running_count(did: IdentityId) -> u32 {
    pallet_validators::PermissionedIdentity::<TestStorage>::get(did)
        .unwrap()
        .running_count
}

fn key_ref_count(key: Sr25519Keyring) -> u64 {
    pallet_identity::AccountKeyRefCount::<TestStorage>::get(key.to_account_id())
}

#[test]
fn remove_permissioned_validator_chills_all_stashes() {
    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(validate_from_secondary_key(
            alice.did,
            Sr25519Keyring::Charlie
        ));
        assert_eq!(running_count(alice.did), 2);
        assert_eq!(key_ref_count(Sr25519Keyring::Bob), 1);

        remove_permission(alice.did);
        assert_eq!(pallet_staking::Validators::<TestStorage>::count(), 0);
        assert_eq!(key_ref_count(Sr25519Keyring::Bob), 0);
        assert_eq!(key_ref_count(Sr25519Keyring::Charlie), 0);

        // The re-added identity starts with all of its slots free.
        permission(alice.did, 2);
        assert_eq!(running_count(alice.did), 0);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Dave));
        assert_eq!(running_count(alice.did), 1);
    });
}

#[test]
fn remove_permissioned_validator_chills_frozen_secondary_key_stashes() {
    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(pallet_identity::Pallet::<TestStorage>::freeze_secondary_keys(alice.origin()));

        remove_permission(alice.did);
        assert_eq!(pallet_staking::Validators::<TestStorage>::count(), 0);
        assert_eq!(key_ref_count(Sr25519Keyring::Bob), 0);
    });
}

fn tracked_stashes(did: IdentityId) -> Vec<AccountId> {
    let mut stashes: Vec<AccountId> =
        pallet_validators::IdentityValidatorStashes::<TestStorage>::iter_key_prefix(did).collect();
    stashes.sort();
    stashes
}

fn sorted(mut keys: Vec<AccountId>) -> Vec<AccountId> {
    keys.sort();
    keys
}

#[test]
fn validator_stashes_are_tracked_by_staking_hooks() {
    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob = Sr25519Keyring::Bob.to_account_id();
        let charlie = Sr25519Keyring::Charlie.to_account_id();

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(validate_from_secondary_key(
            alice.did,
            Sr25519Keyring::Charlie
        ));
        assert_eq!(
            tracked_stashes(alice.did),
            sorted(vec![bob.clone(), charlie.clone()])
        );

        // Re-validating an existing validator doesn't change anything.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::validate(
            Origin::signed(bob.clone()),
            pallet_staking::ValidatorPrefs::default(),
        ));
        assert_eq!(running_count(alice.did), 2);
        assert_eq!(tracked_stashes(alice.did).len(), 2);

        // Chill releases the stash.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::chill(
            Origin::signed(bob)
        ));
        assert_eq!(tracked_stashes(alice.did), vec![charlie.clone()]);

        // Switching to nominating also releases the stash.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::nominate(
            Origin::signed(charlie.clone()),
            vec![charlie.clone().into()],
        ));
        assert!(tracked_stashes(alice.did).is_empty());
        assert_eq!(running_count(alice.did), 0);
    });
}

#[test]
fn remove_permissioned_validator_chills_stashes_left_by_chill_from_governance() {
    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob = Sr25519Keyring::Bob.to_account_id();
        let charlie = Sr25519Keyring::Charlie.to_account_id();

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(validate_from_secondary_key(
            alice.did,
            Sr25519Keyring::Charlie
        ));

        // Only one of the two stashes is chilled; the other stays tracked.
        assert_ok!(
            pallet_validators::Pallet::<TestStorage>::chill_from_governance(
                Origin::root(),
                alice.did,
                vec![bob],
            )
        );
        assert_eq!(tracked_stashes(alice.did), vec![charlie.clone()]);

        permission(alice.did, 2);
        remove_permission(alice.did);
        assert!(!pallet_staking::Validators::<TestStorage>::contains_key(
            &charlie
        ));
        assert!(tracked_stashes(alice.did).is_empty());
        assert_eq!(key_ref_count(Sr25519Keyring::Charlie), 0);
    });
}

#[test]
fn migrate_v2_tracks_existing_validator_stashes() {
    use frame_support::traits::{GetStorageVersion, StorageVersion};

    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);
        let bob = Sr25519Keyring::Bob.to_account_id();
        let charlie = Sr25519Keyring::Charlie.to_account_id();

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(validate_from_secondary_key(
            alice.did,
            Sr25519Keyring::Charlie
        ));

        // Simulate the pre-upgrade state.
        let _ = pallet_validators::IdentityValidatorStashes::<TestStorage>::clear(u32::MAX, None);
        StorageVersion::new(1).put::<pallet_validators::Pallet<TestStorage>>();

        pallet_validators::migrations::migrate_v2::<TestStorage>();
        assert_eq!(
            pallet_validators::Pallet::<TestStorage>::on_chain_storage_version(),
            2
        );
        assert_eq!(
            tracked_stashes(alice.did),
            sorted(vec![bob.clone(), charlie])
        );

        // Running it again is a no-op.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::chill(
            Origin::signed(bob)
        ));
        pallet_validators::migrations::migrate_v2::<TestStorage>();
        assert_eq!(tracked_stashes(alice.did).len(), 1);
    });
}

#[test]
fn existing_validator_cannot_bypass_commission_cap() {
    ExtBuilder::default().build().execute_with(|| {
        let cap = Perbill::from_percent(20);
        pallet_validators::ValidatorCommissionCap::<TestStorage>::put(cap);

        let alice: User = User::new(Sr25519Keyring::Alice);
        assert_ok!(
            pallet_validators::Pallet::<TestStorage>::add_permissioned_validator(
                Origin::root(),
                alice.did,
                None
            )
        );
        assert_ok!(pallet_staking::Pallet::<TestStorage>::bond(
            alice.origin(),
            10_000_000,
            pallet_staking::RewardDestination::Account(alice.acc())
        ));

        let prefs_at_cap = pallet_staking::ValidatorPrefs {
            commission: cap,
            blocked: false,
        };
        let prefs_over_cap = pallet_staking::ValidatorPrefs {
            commission: Perbill::from_percent(100),
            blocked: false,
        };

        // Register at the cap — succeeds.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::validate(
            Origin::signed(alice.acc()),
            prefs_at_cap,
        ));

        // Already-registered validator tries to exceed cap — must be rejected.
        assert_noop!(
            pallet_staking::Pallet::<TestStorage>::validate(
                Origin::signed(alice.acc()),
                prefs_over_cap,
            ),
            pallet_validators::Error::<TestStorage>::CommissionTooHigh
        );
    });
}
