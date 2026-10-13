//! Per-NFT freezes (`Nft::set_nfts_frozen`).
//!
//! A frozen NFT must not leave its holder by any path other than a controller transfer. Each
//! test below covers one path that can move or burn an NFT.

use chrono::prelude::Utc;
use frame_support::{assert_noop, assert_ok};
use sp_keyring::Sr25519Keyring;

use pallet_nft::{Event, FrozenNFT, Owner};
use polymesh_primitives::asset::{AssetHolder, AssetId, AssetType, NonFungibleType};
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::settlement::{InstructionId, Leg, SettlementType};
use polymesh_primitives::{
    with_transaction, AssetHolderKind, IdentityId, NFTId, NFTs, PortfolioId, PortfolioKind,
    PortfolioName, PortfolioNumber, WeightMeter,
};

use super::asset_test::set_timestamp;
use super::nft::{create_nft_collection, mint_nft};
use crate::ext_builder::ExtBuilder;
use crate::storage::{default_asset_holder_set, EventTest, TestStorage, User};

type ComplianceManager = pallet_compliance_manager::Pallet<TestStorage>;
type EAError = pallet_external_agents::Error<TestStorage>;
type NFT = pallet_nft::Pallet<TestStorage>;
type NFTError = pallet_nft::Error<TestStorage>;
type Portfolio = pallet_portfolio::Pallet<TestStorage>;
type PortfolioError = pallet_portfolio::Error<TestStorage>;
type Settlement = pallet_settlement::Pallet<TestStorage>;
type SettlementError = pallet_settlement::Error<TestStorage>;
type System = frame_system::Pallet<TestStorage>;

/// Creates a collection owned by `owner`, mints `n` NFTs to `holder_kind` and pauses
/// compliance so only freeze rules can block a transfer.
fn setup(owner: &User, n: u64, holder_kind: AssetHolderKind) -> AssetId {
    set_timestamp(Utc::now().timestamp() as _);
    System::set_block_number(1);
    let asset_id = create_nft_collection(
        owner.clone(),
        AssetType::NonFungible(NonFungibleType::Derivative),
        Vec::new().into(),
    );
    for _ in 0..n {
        mint_nft(owner.clone(), asset_id, Vec::new(), holder_kind.clone());
    }
    assert_ok!(ComplianceManager::pause_asset_compliance(
        owner.origin(),
        asset_id
    ));
    asset_id
}

fn default_portfolio(user: &User) -> AssetHolder {
    PortfolioId::default_portfolio(user.did).into()
}

fn nfts(asset_id: AssetId, ids: &[u64]) -> NFTs {
    NFTs::new_unverified(asset_id, ids.iter().map(|id| NFTId(*id)).collect())
}

fn freeze(agent: &User, holder: &AssetHolder, nfts: NFTs) {
    assert_ok!(NFT::set_nfts_frozen(
        agent.origin(),
        holder.clone(),
        nfts,
        true
    ));
}

fn settle(
    sender: &AssetHolder,
    receiver: &AssetHolder,
    nfts: NFTs,
) -> Result<(), sp_runtime::DispatchError> {
    let mut weight_meter = WeightMeter::max_limit_no_minimum();
    with_transaction(|| {
        NFT::base_nft_transfer(
            sender.clone(),
            receiver.clone(),
            nfts,
            InstructionId(0),
            None,
            IdentityId::default(),
            &mut weight_meter,
        )
    })
}

#[test]
fn freeze_and_unfreeze() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = setup(&alice, 2, AssetHolderKind::DefaultPortfolio);
        let holder = default_portfolio(&alice);

        assert_ok!(NFT::set_nfts_frozen(
            alice.origin(),
            holder.clone(),
            nfts(asset_id, &[1]),
            true
        ));
        assert!(NFT::is_nft_frozen(&asset_id, &NFTId(1), &holder));
        assert!(!NFT::is_nft_frozen(&asset_id, &NFTId(2), &holder));
        assert_eq!(
            System::events().last().unwrap().event,
            EventTest::Nft(Event::NFTsFrozenStatusUpdated {
                caller_did: alice.did,
                holder: holder.clone(),
                nfts: nfts(asset_id, &[1]),
                frozen: true,
            })
        );

        assert_ok!(NFT::set_nfts_frozen(
            alice.origin(),
            holder.clone(),
            nfts(asset_id, &[1]),
            false
        ));
        assert!(!NFT::is_nft_frozen(&asset_id, &NFTId(1), &holder));
        // `false` is never stored.
        assert!(!FrozenNFT::<TestStorage>::contains_key((
            &asset_id,
            &holder,
            &NFTId(1)
        )));
    });
}

#[test]
fn only_an_agent_can_freeze() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);

        // Not even the holder can freeze or unfreeze its own NFT.
        assert_noop!(
            NFT::set_nfts_frozen(
                bob.origin(),
                default_portfolio(&alice),
                nfts(asset_id, &[1]),
                true
            ),
            EAError::UnauthorizedAgent
        );
        freeze(&alice, &default_portfolio(&bob), nfts(asset_id, &[1]));
        assert_noop!(
            NFT::set_nfts_frozen(
                bob.origin(),
                default_portfolio(&bob),
                nfts(asset_id, &[1]),
                false
            ),
            EAError::UnauthorizedAgent
        );
    });
}

#[test]
fn freeze_rejects_invalid_input() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let holder = default_portfolio(&alice);

        assert_noop!(
            NFT::set_nfts_frozen(alice.origin(), holder.clone(), nfts(asset_id, &[]), true),
            NFTError::ZeroCount
        );
        assert_noop!(
            NFT::set_nfts_frozen(
                alice.origin(),
                holder.clone(),
                nfts(asset_id, &[1, 1]),
                true
            ),
            NFTError::DuplicatedNFTId
        );
        let too_many: Vec<u64> = (1..=11).collect();
        assert_noop!(
            NFT::set_nfts_frozen(
                alice.origin(),
                holder.clone(),
                nfts(asset_id, &too_many),
                true
            ),
            NFTError::MaxNumberOfNFTsPerLegExceeded
        );
        // NFT 2 was never minted.
        assert_noop!(
            NFT::set_nfts_frozen(
                alice.origin(),
                holder.clone(),
                nfts(asset_id, &[1, 2]),
                true
            ),
            NFTError::NFTNotFound
        );
        // The portfolio does not exist.
        let missing_portfolio: AssetHolder =
            PortfolioId::user_portfolio(alice.did, PortfolioNumber(9)).into();
        assert_noop!(
            NFT::set_nfts_frozen(
                alice.origin(),
                missing_portfolio,
                nfts(asset_id, &[1]),
                true
            ),
            PortfolioError::PortfolioDoesNotExist
        );
    });
}

/// Settlement transfers (instruction execution) of a frozen NFT fail; other NFTs still move.
#[test]
fn frozen_nft_cannot_be_settled() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 2, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        assert_noop!(
            settle(&alice_holder, &bob_holder, nfts(asset_id, &[1])),
            NFTError::InvalidNFTTransferNFTIsFrozen
        );
        // A frozen NFT in a batch blocks the whole batch.
        assert_noop!(
            settle(&alice_holder, &bob_holder, nfts(asset_id, &[2, 1])),
            NFTError::InvalidNFTTransferNFTIsFrozen
        );
        // The freeze is per NFT.
        assert_ok!(settle(&alice_holder, &bob_holder, nfts(asset_id, &[2])));
    });
}

/// A frozen NFT can't be locked in an instruction, so it can't be affirmed.
#[test]
fn frozen_nft_cannot_be_affirmed() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);

        freeze(&alice, &default_portfolio(&alice), nfts(asset_id, &[1]));

        let legs = vec![Leg::NonFungible {
            sender: default_portfolio(&alice),
            receiver: default_portfolio(&bob),
            nfts: nfts(asset_id, &[1]),
        }];
        assert_noop!(
            Settlement::add_and_affirm_instruction(
                alice.origin(),
                None,
                SettlementType::SettleOnAffirmation,
                None,
                None,
                legs,
                default_asset_holder_set(alice.did),
                None,
            ),
            NFTError::NFTIsFrozen
        );
    });
}

/// Freezing an NFT after it was affirmed (and locked) stops the instruction from executing.
#[test]
fn freeze_after_affirmation_blocks_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        let legs = vec![Leg::NonFungible {
            sender: alice_holder.clone(),
            receiver: default_portfolio(&bob),
            nfts: nfts(asset_id, &[1]),
        }];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleManual(0),
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
        ));

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        let result = Settlement::execute_manual_instruction(
            alice.origin(),
            InstructionId(0),
            Some(alice_holder.clone()),
            0,
            1,
            0,
            None,
        );
        assert_eq!(
            result.unwrap_err().error,
            SettlementError::FailedAssetTransferringConditions.into()
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}

/// Settle-after-lock: a freeze set after affirmation stops the mediator locking the
/// instruction, and a freeze set after locking stops the locked execution path.
#[test]
fn frozen_nft_blocks_locked_settlement() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);

        // `lock_instruction` dry-runs the transfer, so a freeze makes it fail.
        let legs = vec![Leg::NonFungible {
            sender: alice_holder.clone(),
            receiver: bob_holder.clone(),
            nfts: nfts(asset_id, &[1]),
        }];
        assert_ok!(Settlement::add_and_affirm_with_mediators(
            alice.origin(),
            None,
            SettlementType::SettleAfterLock,
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
            [bob.did]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .try_into()
                .unwrap(),
        ));
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            bob.origin(),
            InstructionId(0),
            None
        ));
        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));
        assert!(Settlement::lock_instruction(
            bob.origin(),
            InstructionId(0),
            frame_support::weights::Weight::MAX
        )
        .is_err());

        // The locked execution path (`simplified_nft_transfer`) rejects a frozen NFT too.
        assert_noop!(
            NFT::simplified_nft_transfer(
                alice_holder.clone(),
                bob_holder,
                nfts(asset_id, &[1]),
                Some(InstructionId(0)),
                None,
                alice.did,
            ),
            NFTError::InvalidNFTTransferNFTIsFrozen
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}

/// `transfer_nft` (account to account) routes through settlement and is blocked.
#[test]
fn frozen_nft_cannot_be_sent_with_transfer_nft() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::Account);
        let alice_holder = AssetHolder::Account(alice.acc());

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        assert!(NFT::transfer_nft(alice.origin(), nfts(asset_id, &[1]), bob.acc(), None).is_err());
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}

/// Same-identity `transfer_funds` (account to own portfolio) is blocked.
#[test]
fn frozen_nft_cannot_move_to_own_portfolio() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = setup(&alice, 1, AssetHolderKind::Account);
        let alice_holder = AssetHolder::Account(alice.acc());

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        let fund = Fund {
            description: FundDescription::NonFungible(nfts(asset_id, &[1])),
            memo: None,
        };
        assert_noop!(
            Settlement::transfer_funds(alice.origin(), None, default_portfolio(&alice), fund)
                .map_err(|e| e.error),
            NFTError::InvalidNFTTransferNFTIsFrozen
        );
    });
}

/// Moving a frozen NFT between the holder's own portfolios is blocked.
#[test]
fn frozen_nft_cannot_move_between_portfolios() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        assert_ok!(Portfolio::create_portfolio(
            alice.origin(),
            PortfolioName(b"Other".to_vec())
        ));
        let from = PortfolioId::default_portfolio(alice.did);
        let to = PortfolioId {
            did: alice.did,
            kind: PortfolioKind::User(PortfolioNumber(1)),
        };

        freeze(&alice, &from.clone().into(), nfts(asset_id, &[1]));

        let fund = Fund {
            description: FundDescription::NonFungible(nfts(asset_id, &[1])),
            memo: None,
        };
        assert_noop!(
            Portfolio::move_portfolio_funds(alice.origin(), from, to, vec![fund]),
            PortfolioError::InvalidTransferNFTIsFrozen
        );
    });
}

#[test]
fn frozen_nft_cannot_be_redeemed() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);

        freeze(&alice, &default_portfolio(&alice), nfts(asset_id, &[1]));

        assert_noop!(
            NFT::redeem_nft(
                alice.origin(),
                asset_id,
                NFTId(1),
                AssetHolderKind::DefaultPortfolio,
                None
            ),
            NFTError::NFTIsFrozen
        );
    });
}

/// The transfer report flags a freeze whether or not settlement locks are skipped.
#[test]
fn transfer_report_flags_frozen_nft() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        for skip_locked_check in [false, true] {
            let mut weight_meter = WeightMeter::max_limit_no_minimum();
            let errors = NFT::nft_transfer_report(
                &alice_holder,
                &default_portfolio(&bob),
                &nfts(asset_id, &[1]),
                skip_locked_check,
                &mut weight_meter,
            );
            assert!(
                errors.contains(&NFTError::InvalidNFTTransferNFTIsFrozen.into()),
                "skip_locked_check = {skip_locked_check}: {errors:?}"
            );
        }
    });
}

/// The freeze is per holder: a freeze set for Bob doesn't stop Alice sending him the NFT,
/// but applies once he holds it.
#[test]
fn freeze_is_per_holder() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let charlie = User::new(Sr25519Keyring::Charlie);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);

        freeze(&alice, &bob_holder, nfts(asset_id, &[1]));

        assert_ok!(settle(&alice_holder, &bob_holder, nfts(asset_id, &[1])));
        assert_noop!(
            settle(
                &bob_holder,
                &default_portfolio(&charlie),
                nfts(asset_id, &[1])
            ),
            NFTError::InvalidNFTTransferNFTIsFrozen
        );
    });
}

#[test]
fn unfreeze_allows_transfer() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));
        assert_ok!(NFT::set_nfts_frozen(
            alice.origin(),
            alice_holder.clone(),
            nfts(asset_id, &[1]),
            false
        ));
        assert_ok!(settle(
            &alice_holder,
            &default_portfolio(&bob),
            nfts(asset_id, &[1])
        ));
    });
}

/// Controller transfers take frozen NFTs, unfreezing them first, and report which ones.
#[test]
fn controller_transfer_unfreezes() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 2, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);
        assert_ok!(settle(&alice_holder, &bob_holder, nfts(asset_id, &[1, 2])));

        freeze(&alice, &bob_holder, nfts(asset_id, &[1]));

        assert_ok!(NFT::controller_transfer(
            alice.origin(),
            nfts(asset_id, &[1, 2]),
            bob_holder.clone(),
            AssetHolderKind::DefaultPortfolio,
        ));
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder.clone())
        );
        assert!(!NFT::is_nft_frozen(&asset_id, &NFTId(1), &bob_holder));
        // Only the NFT that was frozen is reported as unfrozen.
        assert!(System::events().iter().any(|record| record.event
            == EventTest::Nft(Event::NFTsFrozenStatusUpdated {
                caller_did: alice.did,
                holder: bob_holder.clone(),
                nfts: nfts(asset_id, &[1]),
                frozen: false,
            })));
    });
}

#[test]
fn controller_transfer_to_unfreezes() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);
        assert_ok!(settle(&alice_holder, &bob_holder, nfts(asset_id, &[1])));

        freeze(&alice, &bob_holder, nfts(asset_id, &[1]));

        assert_ok!(NFT::controller_transfer_to(
            alice.origin(),
            nfts(asset_id, &[1]),
            bob_holder.clone(),
            alice_holder.clone(),
        ));
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
        assert!(!NFT::is_nft_frozen(&asset_id, &NFTId(1), &bob_holder));
    });
}

/// A controller transfer that fails must not leave the NFTs unfrozen.
#[test]
fn failed_controller_transfer_keeps_freeze() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 2, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);
        let bob_holder = default_portfolio(&bob);
        // Bob holds NFT 1 only.
        assert_ok!(settle(&alice_holder, &bob_holder, nfts(asset_id, &[1])));

        freeze(&alice, &bob_holder, nfts(asset_id, &[1]));

        // NFT 2 isn't Bob's, so the transfer fails after NFT 1 was unfrozen.
        assert!(with_transaction(|| NFT::controller_transfer_to(
            alice.origin(),
            nfts(asset_id, &[1, 2]),
            bob_holder.clone(),
            alice_holder.clone(),
        ))
        .is_err());
        assert!(NFT::is_nft_frozen(&asset_id, &NFTId(1), &bob_holder));
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(bob_holder)
        );
    });
}

/// The settlement error is surfaced for frozen NFTs leaving through a cross-identity
/// `transfer_funds`, which affirms (locks) on the sender's behalf.
#[test]
fn frozen_nft_cannot_be_sent_with_transfer_funds() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        let fund = Fund {
            description: FundDescription::NonFungible(nfts(asset_id, &[1])),
            memo: None,
        };
        let result = Settlement::transfer_funds(
            alice.origin(),
            Some(alice_holder.clone()),
            default_portfolio(&bob),
            fund,
        );
        assert!(result.is_err());
        assert_ne!(
            result.unwrap_err().error,
            SettlementError::SenderSameAsReceiver.into()
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}

/// A freeze set for a holder that never received the NFT can still be cleared after the NFT is
/// redeemed.
#[test]
fn stale_freeze_can_be_cleared() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let bob_holder = default_portfolio(&bob);

        freeze(&alice, &bob_holder, nfts(asset_id, &[1]));
        assert_ok!(NFT::redeem_nft(
            alice.origin(),
            asset_id,
            NFTId(1),
            AssetHolderKind::DefaultPortfolio,
            None
        ));

        // The NFT no longer exists, so it can't be frozen again...
        assert_noop!(
            NFT::set_nfts_frozen(
                alice.origin(),
                bob_holder.clone(),
                nfts(asset_id, &[1]),
                true
            ),
            NFTError::NFTNotFound
        );
        // ...but the stale freeze can be removed.
        assert_ok!(NFT::set_nfts_frozen(
            alice.origin(),
            bob_holder.clone(),
            nfts(asset_id, &[1]),
            false
        ));
        assert!(!FrozenNFT::<TestStorage>::contains_key((
            &asset_id,
            &bob_holder,
            &NFTId(1)
        )));
    });
}

/// Settle-after-lock end to end: a freeze set after the mediator locked the instruction stops
/// the locked execution, the same as a fungible frozen balance does.
#[test]
fn freeze_after_lock_blocks_locked_execution() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let dave = User::new(Sr25519Keyring::Dave);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        let legs = vec![Leg::NonFungible {
            sender: alice_holder.clone(),
            receiver: default_portfolio(&bob),
            nfts: nfts(asset_id, &[1]),
        }];
        assert_ok!(Settlement::add_and_affirm_with_mediators(
            alice.origin(),
            None,
            SettlementType::SettleAfterLock,
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
            [dave.did]
                .into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .try_into()
                .unwrap(),
        ));
        // Receivers are affirmed automatically; only the mediator remains.
        assert_ok!(Settlement::affirm_instruction_as_mediator(
            dave.origin(),
            InstructionId(0),
            None
        ));
        assert_ok!(Settlement::lock_instruction(
            dave.origin(),
            InstructionId(0),
            frame_support::weights::Weight::MAX
        ));

        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        let result = Settlement::execute_manual_instruction(
            dave.origin(),
            InstructionId(0),
            None,
            0,
            1,
            0,
            None,
        );
        assert_eq!(
            result.unwrap_err().error,
            NFTError::InvalidNFTTransferNFTIsFrozen.into()
        );
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}

/// A freeze must not trap a settlement lock: rejecting the instruction still releases it, and
/// the NFT stays frozen.
#[test]
fn reject_releases_lock_on_frozen_nft() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::DefaultPortfolio);
        let alice_holder = default_portfolio(&alice);

        let legs = vec![Leg::NonFungible {
            sender: alice_holder.clone(),
            receiver: default_portfolio(&bob),
            nfts: nfts(asset_id, &[1]),
        }];
        assert_ok!(Settlement::add_and_affirm_instruction(
            alice.origin(),
            None,
            SettlementType::SettleManual(0),
            None,
            None,
            legs,
            default_asset_holder_set(alice.did),
            None,
        ));
        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        assert_ok!(Settlement::reject_instruction(
            alice.origin(),
            InstructionId(0),
            alice_holder.clone(),
        ));
        assert!(
            !pallet_portfolio::PortfolioLockedNFT::<TestStorage>::contains_key(
                PortfolioId::default_portfolio(alice.did),
                (asset_id, NFTId(1))
            )
        );
        assert!(NFT::is_nft_frozen(&asset_id, &NFTId(1), &alice_holder));
    });
}

/// An approved spender can't move a frozen NFT either.
#[test]
fn approved_spender_cannot_move_frozen_nft() {
    ExtBuilder::default().build().execute_with(|| {
        let alice = User::new(Sr25519Keyring::Alice);
        let bob = User::new(Sr25519Keyring::Bob);
        let asset_id = setup(&alice, 1, AssetHolderKind::Account);
        let alice_holder = AssetHolder::Account(alice.acc());

        assert_ok!(NFT::approve(
            alice.origin(),
            asset_id,
            NFTId(1),
            Some(bob.acc())
        ));
        freeze(&alice, &alice_holder, nfts(asset_id, &[1]));

        let fund = Fund {
            description: FundDescription::NonFungible(nfts(asset_id, &[1])),
            memo: None,
        };
        let result = Settlement::transfer_funds(
            bob.origin(),
            Some(alice_holder.clone()),
            AssetHolder::Account(bob.acc()),
            fund,
        );
        assert_eq!(result.unwrap_err().error, NFTError::NFTIsFrozen.into());
        assert_eq!(
            Owner::<TestStorage>::get(asset_id, NFTId(1)),
            Some(alice_holder)
        );
    });
}
