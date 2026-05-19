use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_runtime::Perbill;

use polymesh_primitives::{AuthorizationData, Permissions};

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
