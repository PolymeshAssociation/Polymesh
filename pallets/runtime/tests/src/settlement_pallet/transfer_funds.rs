use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_asset::Allowances;
use polymesh_primitives::asset::AssetHolder;
use polymesh_primitives::nft::NFTId;
use polymesh_primitives::{
    Balance, Fund, FundDescription, HoldingsUpdateReason, NFTs, PortfolioId,
};

use crate::asset_pallet::setup::ISSUE_AMOUNT;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type SettlementError = pallet_settlement::Error<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;

/// Helper to issue tokens to an Account holder.
fn create_and_issue_to_account(owner: &User) -> polymesh_primitives::asset::AssetId {
    use polymesh_primitives::AssetHolderKind;

    let asset_id = Asset::generate_asset_id(owner.acc(), false);
    assert_ok!(Asset::create_asset(
        owner.origin(),
        b"MyAsset".into(),
        true,
        Default::default(),
        Vec::new(),
        None,
    ));
    assert_ok!(Asset::issue(
        owner.origin(),
        asset_id,
        ISSUE_AMOUNT,
        AssetHolderKind::Account,
    ));
    asset_id
}

fn fungible_fund(asset_id: polymesh_primitives::asset::AssetId, amount: Balance) -> Fund {
    Fund {
        description: FundDescription::Fungible { asset_id, amount },
        memo: None,
    }
}

#[test]
fn same_identity_transfer_succeeds() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_to_account(&alice);
        let from = AssetHolder::Account(alice.acc());
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did));

        frame_system::Pallet::<TestStorage>::set_block_number(1);

        // from = None defaults to caller's account.
        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            None,
            to.clone(),
            fungible_fund(asset_id, 100),
        ));

        assert_eq!(
            Asset::get_holders_balance(&from, &asset_id),
            ISSUE_AMOUNT - 100
        );

        // AssetBalanceUpdated emitted with instruction_id: None (direct transfer).
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Asset(pallet_asset::Event::AssetBalanceUpdated(
                    _did, a_id, amount, Some(src), Some(dst),
                    HoldingsUpdateReason::Transferred { instruction_id: None, .. }
                )) if *a_id == asset_id && *amount == 100 && *src == from && *dst == to
            )
        }));
    });
}

#[test]
fn spender_finite_allowance_decrements() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 200),
        ));

        // Allowance decremented.
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), &asset_id)),
            300
        );
        // Balance moved.
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 200
        );
    });
}

#[test]
fn spender_unlimited_allowance_not_decremented() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(
            alice.origin(),
            asset_id,
            bob.acc(),
            Balance::MAX
        ));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 100),
        ));

        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), &asset_id)),
            Balance::MAX
        );
    });
}

#[test]
fn insufficient_allowance_returns_error() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 100));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));

        assert_noop!(
            Settlement::transfer_funds(bob.origin(), from, to, fungible_fund(asset_id, 150)),
            AssetError::InsufficientAllowance
        );
    });
}

#[test]
fn zero_remaining_allowance_removes_entry() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 200));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 200),
        ));

        assert!(!Allowances::<TestStorage>::contains_key((
            &alice.acc(),
            &bob.acc(),
            &asset_id
        )));
    });
}

#[test]
fn self_transfer_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_to_account(&alice);
        let same_holder = AssetHolder::Account(alice.acc());

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(same_holder.clone()),
                same_holder,
                fungible_fund(asset_id, 100),
            ),
            SettlementError::SenderSameAsReceiver
        );
    });
}

#[test]
fn spender_nft_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));
        let nft_fund = Fund {
            description: FundDescription::NonFungible(NFTs::new_unverified(
                asset_id,
                vec![NFTId(1)],
            )),
            memo: None,
        };

        assert_noop!(
            Settlement::transfer_funds(bob.origin(), from, to, nft_fund),
            SettlementError::AllowancesNotSupportedForNFTs
        );
    });
}

#[test]
fn atomicity_failed_transfer_restores_allowance() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));
        assert_ok!(Asset::freeze(alice.origin(), asset_id));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(bob.did));

        assert_noop!(
            Settlement::transfer_funds(bob.origin(), from, to, fungible_fund(asset_id, 100)),
            AssetError::InvalidTransferFrozenAsset
        );

        // Allowance unchanged — extrinsic rollback reverts the decrement.
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), &asset_id)),
            500
        );
    });
}
