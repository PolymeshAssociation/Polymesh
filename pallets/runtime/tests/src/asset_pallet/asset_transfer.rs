use frame_support::assert_ok;
use sp_keyring::Sr25519Keyring;

use pallet_asset::AssetBalance;
use pallet_settlement::InstructionAffirmsPending;
use polymesh_primitives::asset::AssetType;
use polymesh_primitives::settlement::InstructionId;
use polymesh_primitives::AssetHolderKind;

use super::setup::ISSUE_AMOUNT;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type AssetPallet = pallet_asset::Pallet<TestStorage>;

#[test]
fn asset_transfer_with_receiver_pre_approved() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = AssetPallet::generate_asset_id(alice.acc(), false);
        assert_ok!(AssetPallet::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            AssetType::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(AssetPallet::issue(
            alice.origin(),
            asset_id,
            ISSUE_AMOUNT,
            AssetHolderKind::Account
        ));

        assert_ok!(AssetPallet::pre_approve_asset(bob.origin(), asset_id));
        assert_ok!(AssetPallet::transfer_asset(
            alice.origin(),
            asset_id,
            bob.acc(),
            100,
            None
        ));

        assert_eq!(
            InstructionAffirmsPending::<TestStorage>::get(InstructionId(0)),
            0
        );
        assert_eq!(
            AssetBalance::<TestStorage>::get(&alice.acc(), asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(AssetBalance::<TestStorage>::get(&bob.acc(), asset_id), 100);
    })
}
