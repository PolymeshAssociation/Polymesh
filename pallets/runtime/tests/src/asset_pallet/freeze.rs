use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_runtime::AccountId32;

use pallet_asset::{FrozenAccounts, FrozenBalance};
use polymesh_primitives::AssetHolder;

use super::setup::create_and_issue_sample_asset;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type IdentityError = pallet_identity::Error<TestStorage>;

#[test]
fn set_holder_frozen_rejects_account_without_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let unlinked_account = AccountId32::from([42; 32]);

        assert_noop!(
            Asset::set_holder_frozen(
                alice.origin(),
                AssetHolder::Account(unlinked_account.clone()),
                asset_id,
                true,
            ),
            IdentityError::IdentityNotFoundForAccountPortfolio
        );
        assert!(!FrozenAccounts::<TestStorage>::get(
            &unlinked_account,
            &asset_id
        ));

        // Unfreezing is always allowed so stale entries can be cleared.
        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(unlinked_account),
            asset_id,
            false,
        ));
    });
}

#[test]
fn set_frozen_tokens_rejects_account_without_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_sample_asset(&alice);
        let unlinked_account = AccountId32::from([42; 32]);

        assert_noop!(
            Asset::set_frozen_tokens(
                alice.origin(),
                asset_id,
                AssetHolder::Account(unlinked_account.clone()),
                1_000,
            ),
            IdentityError::IdentityNotFoundForAccountPortfolio
        );
        assert_eq!(
            FrozenBalance::<TestStorage>::get(&unlinked_account, &asset_id),
            0
        );

        // Clearing the frozen amount is always allowed so stale entries can be removed.
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(unlinked_account),
            0,
        ));
    });
}

#[test]
fn freeze_account_with_identity() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(bob.acc()),
            asset_id,
            true,
        ));
        assert!(FrozenAccounts::<TestStorage>::get(&bob.acc(), &asset_id));

        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(bob.acc()),
            1_000,
        ));
        assert_eq!(
            FrozenBalance::<TestStorage>::get(&bob.acc(), &asset_id),
            1_000
        );
    });
}
