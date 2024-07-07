use frame_support::{assert_ok, StorageMap};
use sp_keyring::AccountKeyring;

use polymesh_primitives::{AuthorizationData, Permissions};

use crate::ext_builder::ExtBuilder;
use crate::storage::{add_secondary_key, TestStorage, User};

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;

#[test]
fn updating_controller() {
    let charlie = vec![AccountKeyring::Charlie.to_account_id()];
    ExtBuilder::default()
        .cdd_providers(charlie)
        .build()
        .execute_with(|| {
            let alice: User = User::new(AccountKeyring::Alice);
            let eve: User = User::new_with(alice.did, AccountKeyring::Eve);

            add_secondary_key(alice.did, eve.acc());

            let auth_id = pallet_identity::Module::<TestStorage>::add_auth(
                alice.did,
                eve.signatory_acc(),
                AuthorizationData::RotatePrimaryKeyToSecondary(Permissions::default()),
                None,
            )
            .unwrap();

            assert_ok!(
                pallet_staking::Pallet::<TestStorage>::add_permissioned_validator(
                    Origin::root(),
                    alice.did,
                    None
                )
            );

            assert_ok!(pallet_staking::Pallet::<TestStorage>::bond(
                alice.origin(),
                sp_runtime::MultiAddress::Id(alice.acc()),
                10_000_000,
                pallet_staking::RewardDestination::Controller
            ));

            assert_ok!(
                pallet_staking::Pallet::<TestStorage>::set_min_bond_threshold(
                    Origin::root(),
                    50_000
                )
            );

            assert_ok!(pallet_staking::Pallet::<TestStorage>::validate(
                Origin::signed(alice.acc()),
                pallet_staking::ValidatorPrefs::default()
            ));

            assert_ok!(pallet_identity::Module::<TestStorage>::revoke_claim(
                Origin::signed(AccountKeyring::Charlie.to_account_id()),
                alice.did,
                polymesh_primitives::Claim::CustomerDueDiligence(Default::default())
            ));

            assert_ok!(
                pallet_staking::Pallet::<TestStorage>::remove_permissioned_validator(
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
                pallet_identity::Module::<TestStorage>::rotate_primary_key_to_secondary(
                    eve.origin(),
                    auth_id,
                    None
                )
            );

            assert_ok!(pallet_staking::Pallet::<TestStorage>::set_controller(
                alice.origin(),
                sp_runtime::MultiAddress::Id(eve.acc()),
            ));
        });
}
