use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_asset::{AssetBalance, FrozenBalance};
use polymesh_primitives::{AssetHolder, AssetHolderKind};

use super::setup::create_and_issue_sample_asset;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;

#[test]
fn forced_transfer_consumes_frozen_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            1_000,
            AssetHolderKind::Account
        ));
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            alice.acc(),
            800
        ));

        // Only 200 tokens are unfrozen - a regular transfer of 500 must fail.
        assert_noop!(
            Asset::ensure_sufficient_balance(&AssetHolder::Account(alice.acc()), &asset_id, 500),
            AssetError::InvalidTransferFrozenBalance
        );

        // A forced transfer dips into the frozen balance and reduces it.
        assert_ok!(Asset::forced_transfer(
            alice.origin(),
            asset_id,
            500,
            alice.acc(),
            bob.acc(),
        ));

        assert_eq!(AssetBalance::<TestStorage>::get(&alice.acc(), &asset_id), 500);
        assert_eq!(AssetBalance::<TestStorage>::get(&bob.acc(), &asset_id), 500);
        assert_eq!(FrozenBalance::<TestStorage>::get(&alice.acc(), &asset_id), 500);
    });
}

#[test]
fn forced_transfer_within_unfrozen_balance_keeps_frozen_amount() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            1_000,
            AssetHolderKind::Account
        ));
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            alice.acc(),
            800
        ));

        // 200 tokens are unfrozen - a forced transfer of 200 must not touch the frozen amount.
        assert_ok!(Asset::forced_transfer(
            alice.origin(),
            asset_id,
            200,
            alice.acc(),
            bob.acc(),
        ));

        assert_eq!(AssetBalance::<TestStorage>::get(&alice.acc(), &asset_id), 800);
        assert_eq!(AssetBalance::<TestStorage>::get(&bob.acc(), &asset_id), 200);
        assert_eq!(FrozenBalance::<TestStorage>::get(&alice.acc(), &asset_id), 800);
    });
}

#[test]
fn forced_transfer_requires_agent_permissions() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            1_000,
            AssetHolderKind::Account
        ));

        assert_noop!(
            Asset::forced_transfer(bob.origin(), asset_id, 100, alice.acc(), bob.acc()),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn forced_transfer_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);

        let asset_id = create_and_issue_sample_asset(&alice);
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            1_000,
            AssetHolderKind::Account
        ));

        assert_noop!(
            Asset::forced_transfer(alice.origin(), asset_id, 1_001, alice.acc(), bob.acc()),
            AssetError::InsufficientBalance
        );
    });
}
