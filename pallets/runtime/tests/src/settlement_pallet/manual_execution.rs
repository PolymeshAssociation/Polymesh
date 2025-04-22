use frame_support::{assert_err_ignore_postinfo, assert_noop, assert_storage_noop};
use sp_keyring::AccountKeyring;

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

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                eve.origin(),
                InstructionId(0),
                None,
                1,
                0,
                0,
                None
            ),
            Error::<TestStorage>::CallerIsNotAParty
        ));

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                eve.origin(),
                InstructionId(0),
                Some(PortfolioId::user_portfolio(bob.did, PortfolioNumber(1))),
                1,
                0,
                0,
                None
            ),
            PortfolioError::UnauthorizedCustodian
        ));
    });
}

#[test]
fn exceeded_maximum_locking_period() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let _ = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        Timestamp::set_timestamp(Timestamp::get() + 3);

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                0,
                0,
                None
            ),
            Error::<TestStorage>::ExceededMaximumLockingPeriod
        ));
    });
}

#[test]
fn controller_transfer_nft_not_owned() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (_, asset_id) = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

        Settlement::lock_instruction(dave.origin(), InstructionId(0), Weight::MAX).unwrap();

        Nft::controller_transfer(
            dave.origin(),
            NFTs::new_unverified(asset_id, vec![NFTId(1)]),
            PortfolioId::default_portfolio(alice.did),
            PortfolioKind::Default,
        )
        .unwrap();

        assert_storage_noop!(assert_err_ignore_postinfo!(
            Settlement::execute_manual_instruction(
                dave.origin(),
                InstructionId(0),
                None,
                1,
                0,
                0,
                None
            ),
            NFTError::InvalidNFTTransferNFTNotOwned
        ));
    });
}

#[test]
fn controller_transfer_insufficient_balance() {
    ExtBuilder::default().build().execute_with(|| {
        let bob = User::new(AccountKeyring::Bob);
        let dave = User::new(AccountKeyring::Dave);
        let alice = User::new(AccountKeyring::Alice);

        let (asset_id, _) = add_and_affirm_simple_instruction(
            alice,
            bob,
            dave,
            SettlementType::SettleOnComplianceCheck,
        );

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
