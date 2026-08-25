use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;
use sp_std::collections::btree_set::BTreeSet;

use pallet_asset::{Allowances, AssetBalance, BalanceOf, LockedBalance};
use polymesh_primitives::asset::{AssetHolder, AssetHolderKind, AssetType, NonFungibleType};
use polymesh_primitives::nft::{NFTId, NFTOwnerStatus};
use polymesh_primitives::settlement::{AffirmationRequirement, Leg, SettlementType};
use polymesh_primitives::{
    Balance, Fund, FundDescription, NFTs, Permissions, PortfolioId, PortfolioNumber,
    SubsetRestriction,
};

use crate::asset_pallet::setup::{create_and_issue_sample_asset, ISSUE_AMOUNT};
use crate::nft::{create_nft_collection, mint_nft};
use crate::storage::{add_secondary_key_with_perms, User};
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type Nft = pallet_nft::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type SettlementError = pallet_settlement::Error<TestStorage>;
type AssetError = pallet_asset::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;

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

fn non_fungible_fund(asset_id: polymesh_primitives::asset::AssetId, nft_id: NFTId) -> Fund {
    Fund {
        description: FundDescription::NonFungible(NFTs::new_unverified(asset_id, vec![nft_id])),
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
        assert_eq!(Asset::get_holders_balance(&to, &asset_id), 100);

        let mut events = frame_system::Pallet::<TestStorage>::events();

        assert_eq!(
            events.pop().unwrap().event,
            crate::storage::RuntimeEvent::Settlement(pallet_settlement::Event::FundsTransferred(
                alice.did,
                from,
                to,
                fungible_fund(asset_id, 100)
            ))
        );
    });
}

#[test]
fn cross_identity_transfer_creates_settlement() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        frame_system::Pallet::<TestStorage>::set_block_number(1);

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            None,
            AssetHolder::Account(bob.acc()),
            fungible_fund(asset_id, 100),
        ));

        // Balance moved from alice to bob.
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(bob.acc()), &asset_id),
            100
        );

        // Settlement instruction was created and executed.
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Settlement(
                    pallet_settlement::Event::InstructionExecuted(_, _)
                )
            )
        }));
    });
}

#[test]
fn spender_same_identity_uses_direct_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        // Bob has allowance to spend from alice's account.
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        // Transfer from alice's account to alice's default portfolio (same DID).
        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did));

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from.clone(),
            to.clone(),
            fungible_fund(asset_id, 100),
        ));

        // Allowance decremented.
        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            400
        );
        // Balance moved within alice's identity.
        assert_eq!(
            Asset::get_holders_balance(&from.unwrap(), &asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(Asset::get_holders_balance(&to, &asset_id), 100);
        assert_eq!(
            BalanceOf::<TestStorage>::get(&asset_id, &alice.did),
            ISSUE_AMOUNT
        );
    });
}

#[test]
fn spender_as_receiver_affirms_both_sides() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        // Bob spends alice's allowance to transfer to himself.
        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Account(bob.acc());

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 200),
        ));

        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            300
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 200
        );
        // Bob received the tokens (settlement executed immediately).
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(bob.acc()), &asset_id),
            200
        );
    });
}

#[test]
fn spender_unlimited_allowance_not_decremented() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(
            alice.origin(),
            asset_id,
            bob.acc(),
            Balance::MAX
        ));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Account(charlie.acc());

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 100),
        ));

        assert_eq!(
            Allowances::<TestStorage>::get((&alice.acc(), &bob.acc(), asset_id)),
            Balance::MAX
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(charlie.acc()), &asset_id),
            100
        );
    });
}

#[test]
fn spender_insufficient_allowance_returns_error() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 100));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Account(charlie.acc());

        assert_noop!(
            Settlement::transfer_funds(bob.origin(), from, to, fungible_fund(asset_id, 150)),
            AssetError::InsufficientAllowance
        );
    });
}

#[test]
fn spender_zero_allowance_removes_entry() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 200));

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Account(charlie.acc());

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            from,
            to,
            fungible_fund(asset_id, 200),
        ));

        assert!(!Allowances::<TestStorage>::contains_key((
            &alice.acc(),
            &bob.acc(),
            asset_id
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
        let to = AssetHolder::Account(bob.acc());
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
fn portfolio_custody_authorized_succeeds() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);

        // Issue tokens to alice's default portfolio.
        let asset_id = Asset::generate_asset_id(alice.acc(), false);
        assert_ok!(Asset::create_asset(
            alice.origin(),
            b"MyAsset".into(),
            true,
            Default::default(),
            Vec::new(),
            None,
        ));
        assert_ok!(Asset::issue(
            alice.origin(),
            asset_id,
            ISSUE_AMOUNT,
            AssetHolderKind::DefaultPortfolio,
        ));

        // Owner is default custodian — can transfer from own portfolio.
        let from = Some(AssetHolder::Portfolio(PortfolioId::default_portfolio(
            alice.did,
        )));
        let to = AssetHolder::Account(alice.acc());

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            from,
            to,
            fungible_fund(asset_id, 100),
        ));
    });
}

#[test]
fn portfolio_custody_unauthorized_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let _asset_id = create_and_issue_to_account(&alice);

        // Bob tries to transfer from alice's default portfolio without custody.
        let from = Some(AssetHolder::Portfolio(PortfolioId::default_portfolio(
            alice.did,
        )));
        let to = AssetHolder::Account(bob.acc());

        assert_noop!(
            Settlement::transfer_funds(bob.origin(), from, to, fungible_fund(_asset_id, 100)),
            PortfolioError::UnauthorizedCustodian
        );
    });
}

/// Helper to create an NFT collection and mint one NFT to an Account holder.
fn create_and_issue_nft_to_account(owner: &User) -> polymesh_primitives::asset::AssetId {
    let asset_id = Asset::generate_asset_id(owner.acc(), false);
    assert_ok!(Asset::create_asset(
        owner.origin(),
        b"MyNFTAsset".into(),
        false,
        AssetType::NonFungible(NonFungibleType::Derivative),
        Vec::new(),
        None,
    ));
    assert_ok!(Nft::create_nft_collection(
        owner.origin(),
        Some(asset_id),
        None,
        Vec::new().into(),
    ));
    assert_ok!(Nft::issue_nft(
        owner.origin(),
        asset_id,
        Vec::new(),
        AssetHolderKind::Account,
    ));
    asset_id
}

#[test]
fn nft_same_identity_transfer_succeeds() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_nft_to_account(&alice);

        let from = Some(AssetHolder::Account(alice.acc()));
        let to = AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did));

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            from,
            to,
            non_fungible_fund(asset_id, NFTId(1)),
        ));

        // NFT moved to portfolio.
        assert_eq!(
            pallet_nft::NFTHolder::<TestStorage>::get((&alice.acc(), &asset_id, &NFTId(1))),
            NFTOwnerStatus::NotOwned
        );
    });
}

#[test]
fn nft_cross_identity_creates_settlement() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_nft_to_account(&alice);

        frame_system::Pallet::<TestStorage>::set_block_number(1);

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            None,
            AssetHolder::Account(bob.acc()),
            non_fungible_fund(asset_id, NFTId(1)),
        ));

        // Settlement instruction was created and executed.
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Settlement(
                    pallet_settlement::Event::InstructionExecuted(_, _)
                )
            )
        }));
    });
}

#[test]
fn nft_spender_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = create_and_issue_nft_to_account(&alice);

        // Approve bob as spender (fungible allowance).
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        // Bob tries to transfer alice's NFT — rejected (allowances not supported for NFTs).
        assert_noop!(
            Settlement::transfer_funds(
                bob.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Account(bob.acc()),
                non_fungible_fund(asset_id, NFTId(1)),
            ),
            SettlementError::AllowancesNotSupportedForNFTs
        );
    });
}

#[test]
fn nft_reject_frozen_asset() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_nft_to_account(&alice);

        assert_ok!(Asset::freeze(alice.origin(), asset_id));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
                non_fungible_fund(asset_id, NFTId(1)),
            ),
            AssetError::InvalidTransferFrozenAsset
        );
    });
}
#[test]
fn reject_secondary_key_transfer_without_permission() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new_with(alice.did, Sr25519Keyring::Charlie);

        // Add charlie as secondary key for alice, but without portfolio permissions.
        let mut permissions = Permissions::default();
        permissions.portfolio = SubsetRestriction::empty();
        add_secondary_key_with_perms(alice.did, charlie.acc(), permissions);

        let asset_id = create_and_issue_sample_asset(&alice);

        // charlie tries to transfer from alice's default portfolio without portfolio permissions
        let from = AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did));
        let to = AssetHolder::Account(charlie.acc());

        assert_noop!(
            Settlement::transfer_funds(
                charlie.origin(),
                Some(from),
                to,
                fungible_fund(asset_id, 100)
            ),
            PortfolioError::SecondaryKeyNotAuthorizedForPortfolio
        );
    });
}

#[test]
fn cross_identity_transfer_when_caller_has_spending_approval() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<TestStorage>::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, charlie.acc(), 500));

        assert_ok!(Settlement::transfer_funds(
            charlie.origin(),
            Some(AssetHolder::Account(alice.acc())),
            AssetHolder::Account(bob.acc()),
            fungible_fund(asset_id, 100),
        ));

        // Balance moved from alice to bob.
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(bob.acc()), &asset_id),
            100
        );

        // Settlement instruction was created and executed.
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Settlement(
                    pallet_settlement::Event::InstructionExecuted(_, _)
                )
            )
        }));
    });
}

#[test]
fn cross_identity_transfer_when_caller_has_portfolio_permissions() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<TestStorage>::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_default_portfolio =
            AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did));
        let charlie = User::new_with(alice.did, Sr25519Keyring::Charlie);

        let asset_id = create_and_issue_sample_asset(&alice);

        add_secondary_key_with_perms(alice.did, charlie.acc(), Permissions::default());

        assert_ok!(Settlement::transfer_funds(
            charlie.origin(),
            Some(alice_default_portfolio.clone()),
            AssetHolder::Account(bob.acc()),
            fungible_fund(asset_id, 100),
        ));

        // Balance moved from alice to bob.
        assert_eq!(
            Asset::get_holders_balance(&alice_default_portfolio, &asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(bob.acc()), &asset_id),
            100
        );

        // Settlement instruction was created and executed.
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Settlement(
                    pallet_settlement::Event::InstructionExecuted(_, _)
                )
            )
        }));
    });
}

#[test]
fn cross_identity_transfer_when_caller_is_also_the_receiver() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<TestStorage>::set_block_number(1);

        let bob = User::new(Sr25519Keyring::Bob);
        let eve = User::new(Sr25519Keyring::Eve);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new_with(bob.did, Sr25519Keyring::Charlie);

        add_secondary_key_with_perms(bob.did, charlie.acc(), Permissions::default());

        assert_ok!(Settlement::set_mandatory_receiver_affirmation(
            eve.origin(),
            AffirmationRequirement::Required
        ));

        assert_ok!(Portfolio::create_portfolio(
            charlie.origin(),
            b"Charlie".into()
        ));
        let charlie_portfolio = PortfolioId::user_portfolio(bob.did, PortfolioNumber(1));
        crate::portfolio::set_custodian_ok(charlie.clone(), eve.clone(), charlie_portfolio.clone());
        assert!(
            pallet_portfolio::PortfolioCustodian::<TestStorage>::get(&charlie_portfolio)
                == Some(eve.did)
        );

        let asset_id = create_and_issue_to_account(&alice);
        assert_ok!(Asset::approve(alice.origin(), asset_id, bob.acc(), 500));

        assert_ok!(Settlement::transfer_funds(
            bob.origin(),
            Some(AssetHolder::Account(alice.acc())),
            AssetHolder::Portfolio(charlie_portfolio.clone()),
            fungible_fund(asset_id, 100),
        ));

        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT
        );
        assert_eq!(
            Asset::get_holders_balance(
                &AssetHolder::Portfolio(charlie_portfolio.clone()),
                &asset_id
            ),
            0
        );

        // Settlement instruction was created
        let events = frame_system::Pallet::<TestStorage>::events();
        assert!(events.iter().any(|record| {
            matches!(
                &record.event,
                crate::storage::EventTest::Settlement(
                    pallet_settlement::Event::InstructionCreated(..)
                )
            )
        }));
    });
}

#[test]
fn cross_identity_transfer_when_tokens_are_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, charlie.acc(), 500));

        // At this point alice has only 500 tokens not locked
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: AssetHolder::Account(alice.acc()),
                receiver: AssetHolder::Account(bob.acc()),
                asset_id,
                amount: ISSUE_AMOUNT - 500,
            }],
            BTreeSet::from([AssetHolder::Account(alice.acc())])
                .try_into()
                .unwrap(),
            None
        ));

        assert_eq!(
            LockedBalance::<TestStorage>::get(alice.acc(), &asset_id),
            ISSUE_AMOUNT - 500
        );

        // 500 tokens are frozen, so alice can't transfer any tokens
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(alice.acc()),
            500
        ));

        assert_noop!(
            Settlement::transfer_funds(
                charlie.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Account(bob.acc()),
                fungible_fund(asset_id, 1),
            ),
            AssetError::InsufficientBalance
        );

        // Now alice has 400 tokens available for transfer
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(alice.acc()),
            100
        ));

        assert_ok!(Settlement::transfer_funds(
            charlie.origin(),
            Some(AssetHolder::Account(alice.acc())),
            AssetHolder::Account(bob.acc()),
            fungible_fund(asset_id, 400),
        ));

        assert_eq!(
            LockedBalance::<TestStorage>::get(alice.acc(), &asset_id),
            ISSUE_AMOUNT - 500
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 400
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(bob.acc()), &asset_id),
            400
        );
    });
}

#[test]
fn same_identity_transfer_when_tokens_are_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_to_account(&alice);

        // At this point alice has only 500 tokens not locked
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleOnAffirmation,
            None,
            None,
            vec![Leg::Fungible {
                sender: AssetHolder::Account(alice.acc()),
                receiver: AssetHolder::Account(bob.acc()),
                asset_id,
                amount: ISSUE_AMOUNT - 500,
            }],
            BTreeSet::from([AssetHolder::Account(alice.acc())])
                .try_into()
                .unwrap(),
            None
        ));

        // 500 tokens are frozen, so alice can't transfer any tokens
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(alice.acc()),
            500
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
                fungible_fund(asset_id, 1),
            ),
            AssetError::InsufficientBalance
        );

        // Now alice has 400 tokens available for transfer
        assert_ok!(Asset::set_frozen_tokens(
            alice.origin(),
            asset_id,
            AssetHolder::Account(alice.acc()),
            100
        ));

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            Some(AssetHolder::Account(alice.acc())),
            AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
            fungible_fund(asset_id, 400),
        ));

        assert_eq!(
            LockedBalance::<TestStorage>::get(alice.acc(), &asset_id),
            ISSUE_AMOUNT - 500
        );
        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 400
        );
        assert_eq!(
            Asset::get_holders_balance(
                &AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
                &asset_id
            ),
            400
        );
        assert_eq!(
            AssetBalance::<TestStorage>::get(&alice.acc(), &asset_id),
            ISSUE_AMOUNT - 400
        );
        assert_eq!(
            BalanceOf::<TestStorage>::get(&asset_id, &alice.did),
            ISSUE_AMOUNT
        );
    });
}

#[test]
fn cross_identity_transfer_when_account_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::approve(alice.origin(), asset_id, charlie.acc(), 500));

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(alice.acc()),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                charlie.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Account(bob.acc()),
                fungible_fund(asset_id, 100),
            ),
            AssetError::InvalidTransferSenderIsFrozen
        );
    });
}

#[test]
fn same_identity_transfer_when_account_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = create_and_issue_to_account(&alice);

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(alice.acc()),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(AssetHolder::Account(alice.acc())),
                AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
                fungible_fund(asset_id, 100),
            ),
            AssetError::InvalidTransferSenderIsFrozen
        );

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(alice.acc()),
            asset_id,
            false,
        ));

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            Some(AssetHolder::Account(alice.acc())),
            AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
            fungible_fund(asset_id, 100),
        ),);

        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            ISSUE_AMOUNT - 100
        );
        assert_eq!(
            Asset::get_holders_balance(
                &AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
                &asset_id
            ),
            100
        );
    });
}

#[test]
fn same_identity_transfer_when_portfolio_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_portfolio = PortfolioId::default_portfolio(alice.did);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            alice_portfolio.clone().into(),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(alice_portfolio.clone().into()),
                AssetHolder::Account(alice.acc()),
                fungible_fund(asset_id, 100),
            ),
            AssetError::InvalidTransferSenderIsFrozen
        );

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Portfolio(PortfolioId::default_portfolio(alice.did)),
            asset_id,
            false,
        ));

        assert_ok!(Settlement::transfer_funds(
            alice.origin(),
            Some(alice_portfolio.clone().into()),
            AssetHolder::Account(alice.acc()),
            fungible_fund(asset_id, 100),
        ),);

        assert_eq!(
            Asset::get_holders_balance(&AssetHolder::Account(alice.acc()), &asset_id),
            100
        );
        assert_eq!(
            Asset::get_holders_balance(&alice_portfolio.into(), &asset_id),
            ISSUE_AMOUNT - 100
        );
    });
}

#[test]
fn cross_identity_transfer_when_portfolio_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_portfolio = PortfolioId::default_portfolio(alice.did);
        let asset_id = create_and_issue_sample_asset(&alice);

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            alice_portfolio.clone().into(),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(alice_portfolio.into()),
                AssetHolder::Account(bob.acc()),
                fungible_fund(asset_id, 100),
            ),
            AssetError::InvalidTransferSenderIsFrozen
        );
    });
}

#[test]
fn cross_identity_nft_transfer_when_portfolio_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(Sr25519Keyring::Bob);
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_portfolio = PortfolioId::default_portfolio(alice.did);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id,
            Vec::new(),
            AssetHolderKind::DefaultPortfolio,
        );

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            alice_portfolio.clone().into(),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(alice_portfolio.into()),
                AssetHolder::Account(bob.acc()),
                non_fungible_fund(asset_id, NFTId(1)),
            ),
            SettlementError::FailedAssetTransferringConditions
        );
    });
}

#[test]
fn same_identity_nft_transfer_when_portfolio_is_frozen() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let alice_portfolio = PortfolioId::default_portfolio(alice.did);

        let asset_id = create_nft_collection(
            alice.clone(),
            AssetType::NonFungible(NonFungibleType::Derivative),
            Vec::new().into(),
        );
        mint_nft(
            alice.clone(),
            asset_id,
            Vec::new(),
            AssetHolderKind::Account,
        );

        assert_ok!(Asset::set_holder_frozen(
            alice.origin(),
            AssetHolder::Account(alice.acc()),
            asset_id,
            true,
        ));

        assert_noop!(
            Settlement::transfer_funds(
                alice.origin(),
                Some(AssetHolder::Account(alice.acc())),
                alice_portfolio.into(),
                non_fungible_fund(asset_id, NFTId(1)),
            ),
            AssetError::InvalidTransferSenderIsFrozen
        );
    });
}
