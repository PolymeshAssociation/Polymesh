use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_runtime::{DispatchResult, Perbill};

use polymesh_primitives::{AuthorizationData, IdentityId, Permissions};

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

            assert_eq!(
                pallet_identity::AccountKeyRefCount::<TestStorage>::get(alice.acc()),
                1
            );

            assert_ok!(pallet_staking::Pallet::<TestStorage>::chill(alice.origin(),));

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

fn running_count(did: IdentityId) -> u32 {
    pallet_validators::PermissionedIdentity::<TestStorage>::get(did)
        .unwrap()
        .running_count
}

#[test]
fn readding_identity_counts_stashes_left_by_remove_permissioned_validator() {
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

        // De-permissioning leaves both stashes validating.
        assert_ok!(
            pallet_validators::Pallet::<TestStorage>::remove_permissioned_validator(
                Origin::root(),
                alice.did,
            )
        );
        assert_eq!(pallet_staking::Validators::<TestStorage>::count(), 2);

        // The live stashes still occupy the identity's slots after the re-add.
        permission(alice.did, 2);
        assert_eq!(running_count(alice.did), 2);
        assert_eq!(
            validate_from_secondary_key(alice.did, Sr25519Keyring::Dave),
            Err(pallet_staking::Error::<TestStorage>::TooManyValidators.into())
        );
    });
}

#[test]
fn readding_frozen_identity_counts_secondary_key_stashes() {
    ExtBuilder::default().build().execute_with(|| {
        pallet_staking::ValidatorCount::<TestStorage>::put(100);
        let alice: User = User::new(Sr25519Keyring::Alice);

        permission(alice.did, 2);
        assert_ok!(validate_from_secondary_key(alice.did, Sr25519Keyring::Bob));
        assert_ok!(
            pallet_validators::Pallet::<TestStorage>::remove_permissioned_validator(
                Origin::root(),
                alice.did,
            )
        );
        assert_ok!(pallet_identity::Pallet::<TestStorage>::freeze_secondary_keys(alice.origin()));

        permission(alice.did, 2);
        assert_eq!(running_count(alice.did), 1);
    });
}

#[test]
fn readding_identity_counts_stashes_left_by_chill_from_governance() {
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

        // Only one of the two stashes is chilled.
        assert_ok!(
            pallet_validators::Pallet::<TestStorage>::chill_from_governance(
                Origin::root(),
                alice.did,
                vec![bob],
            )
        );
        assert_eq!(pallet_staking::Validators::<TestStorage>::count(), 1);

        permission(alice.did, 1);
        assert_eq!(running_count(alice.did), 1);
        assert_eq!(
            validate_from_secondary_key(alice.did, Sr25519Keyring::Dave),
            Err(pallet_staking::Error::<TestStorage>::TooManyValidators.into())
        );

        // Chilling the remaining stash releases its slot.
        assert_ok!(pallet_staking::Pallet::<TestStorage>::chill(
            Origin::signed(charlie)
        ));
        assert_eq!(running_count(alice.did), 0);
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
