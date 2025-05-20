use frame_support::{assert_err_ignore_postinfo, assert_noop, assert_ok, assert_storage_noop};
use sp_keyring::AccountKeyring;

use pallet_asset::BalanceOf;
use pallet_nft::{NFTOwner, NumberOfNFTs};
use pallet_portfolio::{PortfolioAssetBalances, PortfolioLockedNFT};
use pallet_portfolio::{PortfolioLockedAssets, PortfolioNFT};
use pallet_settlement::Error;
use polymesh_primitives::settlement::{InstructionId, SettlementType};
use polymesh_primitives::{NFTId, NFTs, PortfolioId, PortfolioKind, PortfolioNumber};
use polymesh_runtime_common::Weight;

use super::setup::add_and_affirm_simple_instruction;
use crate::storage::User;
use crate::{ExtBuilder, TestStorage};

type Asset = pallet_asset::Pallet<TestStorage>;
type Nft = pallet_nft::Pallet<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type System = frame_system::Pallet<TestStorage>;
type Timestamp = pallet_timestamp::Pallet<TestStorage>;

type AssetError = pallet_asset::Error<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;

#[test]
fn invalid_caller() {
    ExtBuilder::default().build().execute_with(|| {
        let eve = User::new(AccountKeyring::Eve);
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                eve.origin(),
                InstructionId(0),
                None,
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::CallerIsNotAMediator
        ));

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                eve.origin(),
                InstructionId(0),
                Some(PortfolioId::user_portfolio(bob.did, PortfolioNumber(1))),
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::CallerIsNotAMediator
        ));
    });
}

#[test]
fn exceeded_maximum_locking_period() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        Timestamp::set_timestamp(Timestamp::get() + 3);

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::ExceededMaximumLockingPeriod
        ));
    });
}

#[test]
fn controller_transfer_nft_is_locked() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (_, asset_id) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        assert_noop!(
            Nft::controller_transfer(
                dave.origin(),
                NFTs::new_unverified(asset_id, vec![NFTId(1)]),
                PortfolioId::default_portfolio(alice.did),
                PortfolioKind::Default,
            ),
            NFTError::InvalidNFTTransferNFTIsLocked
        );
    });
}

#[test]
fn controller_transfer_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (asset_id, _) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        assert_noop!(
            Asset::controller_transfer(
                dave.origin(),
                asset_id,
                1_000,
                PortfolioId::default_portfolio(alice.did),
            ),
            PortfolioError::InsufficientPortfolioBalance
        );
    });
}

#[test]
fn unexpected_settle_on_affirmation() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnAffirmation,
        );

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::UnexpectedSettlementType
        ));
    });
}

#[test]
fn unexpected_settle_on_block() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnBlock(System::block_number() + 1),
        );

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::UnexpectedSettlementType
        ));
    });
}

#[test]
fn execute_before_lock() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                1,
                0,
                None
            ),
            Error::<TestStorage>::UnexpectedSettlementType
        ));
    });
}

#[test]
fn successfully_execute_after_locking() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (asset_id, nft_asset_id) =
            add_and_affirm_simple_instruction(alice, bob, dave, SettlementType::SettleAfterLock);

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        assert_ok!(Settlement::execute_manual_instruction(
            dave.origin(),
            InstructionId(0),
            None,
            1,
            1,
            0,
            None
        ));

        let bob_default_portfolio = PortfolioId::default_portfolio(bob.did);
        let alice_default_portfolio = PortfolioId::default_portfolio(alice.did);

        // The balances should have been updated
        assert_eq!(BalanceOf::<TestStorage>::get(&asset_id, &alice.did), 0);
        assert_eq!(BalanceOf::<TestStorage>::get(&asset_id, &bob.did), 1_000);
        assert_eq!(
            PortfolioAssetBalances::<TestStorage>::get(&alice_default_portfolio, &asset_id),
            0
        );
        assert_eq!(
            PortfolioAssetBalances::<TestStorage>::get(&bob_default_portfolio, &asset_id),
            1_000
        );

        assert_eq!(NumberOfNFTs::<TestStorage>::get(&nft_asset_id, bob.did), 1);
        assert_eq!(
            NumberOfNFTs::<TestStorage>::get(&nft_asset_id, alice.did),
            0
        );
        assert_eq!(
            PortfolioNFT::<TestStorage>::get(bob_default_portfolio, (&nft_asset_id, NFTId(1))),
            true
        );
        assert_eq!(
            PortfolioNFT::<TestStorage>::get(alice_default_portfolio, (&nft_asset_id, NFTId(1))),
            false
        );
        assert_eq!(
            NFTOwner::<TestStorage>::get(nft_asset_id, NFTId(1)),
            Some(bob_default_portfolio)
        );

        // Alls locks must have been removed
        assert_eq!(
            PortfolioLockedAssets::<TestStorage>::get(alice_default_portfolio, asset_id),
            0
        );
        assert_eq!(
            PortfolioLockedNFT::<TestStorage>::get(
                alice_default_portfolio,
                (nft_asset_id, NFTId(1))
            ),
            false
        );
    });
}
