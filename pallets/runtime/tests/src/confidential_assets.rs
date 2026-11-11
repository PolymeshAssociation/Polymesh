use codec::Encode;
use frame_support::{
    assert_noop, assert_ok,
    traits::{Currency, Get, StorageVersion},
};
use frame_system::RawOrigin;
use pallet_confidential_assets::{
    weights::SubstrateWeight, AccountDid, AssetDetails, AssetFrozen, AssetLeaves, Details, Error,
    FeeAccountDid, FreezeOrigin, Keys, Pallet, RelayerSubmitExtensionMarker, WeightInfo,
};
use polymesh_dart::{
    curve_tree::{FeeAccountTreeConfig, ProverCurveTree},
    AccountKeys, AccountPublicKey, AccountRegistrationProof, AssetState,
    BatchedFeeAccountRegistrationProof, BatchedFeeAccountTopupProof,
    EncryptionKeyRegistrationProof, EncryptionPublicKey, FeeAccountRegistrationProof, LeafIndex,
    PolymeshLimits, FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FEE_ASSET_ID,
};
use polymesh_primitives::{AccountId, IdentityId};
use rand::{rngs::StdRng, thread_rng, SeedableRng};
use std::collections::{BTreeMap, BTreeSet};

use crate::{ExtBuilder, TestStorage as Runtime};

type Timestamp = pallet_timestamp::Pallet<Runtime>;

fn with_asset(test: impl FnOnce(IdentityId, AssetState)) {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let owner = IdentityId::from(1u128);
        let auditor = AccountKeys::from_seed("FreezeAuditor").unwrap().enc.public;
        let state = AssetState::new::<()>(0, &[], &[auditor]).unwrap();
        Details::<Runtime>::insert(
            0,
            AssetDetails::<Runtime> {
                owner_did: owner,
                total_supply: 0,
                data: Default::default(),
            },
        );
        Keys::<Runtime>::insert(0, &state.keys);
        test(owner, state);
    });
}

#[test]
fn fee_account_root_expires_after_four_hours_for_all_uses() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        pallet_babe::CurrentSlot::<Runtime>::set(sp_consensus_babe::Slot::from(
            1 / pallet_babe::Pallet::<Runtime>::slot_duration(),
        ));
        Timestamp::set_timestamp(1);

        let mut rng = StdRng::from_entropy();
        let account = AccountKeys::from_seed("RelayerFeeRootAge").unwrap();
        let (_, fee_state) = FeeAccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &account.acct,
            FEE_ASSET_ID,
            100,
            &[0; 32],
        )
        .unwrap();
        let mut fee_tree =
            ProverCurveTree::<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>::new(1)
                .unwrap();
        fee_tree.set_block_number(1).unwrap();
        fee_tree
            .insert(
                fee_state
                    .current_commitment()
                    .unwrap()
                    .as_leaf_value()
                    .unwrap(),
            )
            .unwrap();
        let root_block = Pallet::<Runtime>::store_fee_account_curve_tree_root(
            pallet_confidential_assets::TimestampedTreeRoot::new(fee_tree.root().unwrap()),
        );

        let timestamp = 14_400_001;
        pallet_babe::CurrentSlot::<Runtime>::set(sp_consensus_babe::Slot::from(
            timestamp / pallet_babe::Pallet::<Runtime>::slot_duration(),
        ));
        Timestamp::set_timestamp(timestamp);

        assert!(Pallet::<Runtime>::get_fee_account_curve_tree_root(root_block.into()).is_ok());

        let timestamp = 14_400_002;
        pallet_babe::CurrentSlot::<Runtime>::set(sp_consensus_babe::Slot::from(
            timestamp / pallet_babe::Pallet::<Runtime>::slot_duration(),
        ));
        Timestamp::set_timestamp(timestamp);
        assert!(matches!(
            Pallet::<Runtime>::get_fee_account_curve_tree_root(root_block.into()),
            Err(Error::CurveTreeRootNotFound)
        ));
    });
}

#[test]
fn confidential_assets_registration_proofs_enforce_runtime_key_limit() {
    ExtBuilder::default().build().execute_with(|| {
        let account = AccountId::new([71; 32]);
        let did =
            pallet_identity::Pallet::<Runtime>::testing_register_did(account.clone()).unwrap();
        let did_bytes: [u8; 32] = did.into();
        let accounts = (0..21)
            .map(|index| AccountKeys::from_seed(&format!("RegistrationLimit{index}")).unwrap())
            .collect::<Vec<_>>();
        let mut rng = thread_rng();

        let account_proof =
            AccountRegistrationProof::<PolymeshLimits>::new(&mut rng, &accounts, &did_bytes)
                .unwrap();
        assert_eq!(account_proof.len(), 21);
        assert_noop!(
            Pallet::<Runtime>::register_accounts(
                RawOrigin::Signed(account.clone()).into(),
                account_proof,
            ),
            Error::<Runtime>::TooManyProofs
        );

        let encryption_keys = accounts
            .iter()
            .map(|account| account.enc.clone())
            .collect::<Vec<_>>();
        let encryption_proof = EncryptionKeyRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &encryption_keys,
            &did_bytes,
        )
        .unwrap();
        assert_eq!(encryption_proof.len(), 21);
        assert_noop!(
            Pallet::<Runtime>::register_encryption_keys(
                RawOrigin::Signed(account).into(),
                encryption_proof,
            ),
            Error::<Runtime>::TooManyProofs
        );
    });
}

#[test]
fn confidential_assets_account_key_ownership_is_shared_between_asset_and_fee_accounts() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let owner = AccountId::new([74; 32]);
        let other_owner = AccountId::new([75; 32]);
        let owner_did =
            pallet_identity::Pallet::<Runtime>::testing_register_did(owner.clone()).unwrap();
        let other_did =
            pallet_identity::Pallet::<Runtime>::testing_register_did(other_owner).unwrap();
        let owner_did_bytes: [u8; 32] = owner_did.into();
        let other_did_bytes: [u8; 32] = other_did.into();
        let mut rng = StdRng::from_entropy();
        let _ =
            pallet_balances::Pallet::<Runtime>::make_free_balance_be(&owner, 10_000_000_000u128);

        let shared_keys = AccountKeys::from_seed("SharedAccountOwnership").unwrap();
        let (fee_registration, _) = FeeAccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &shared_keys.acct,
            FEE_ASSET_ID,
            100,
            &owner_did_bytes,
        )
        .unwrap();
        let fee_registration = BatchedFeeAccountRegistrationProof {
            proofs: vec![fee_registration].try_into().unwrap(),
        };
        assert_ok!(Pallet::<Runtime>::register_fee_accounts(
            RawOrigin::Signed(owner.clone()).into(),
            fee_registration,
        ));

        let account_registration = AccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &[shared_keys.clone()],
            &owner_did_bytes,
        )
        .unwrap();
        assert_ok!(Pallet::<Runtime>::register_accounts(
            RawOrigin::Signed(owner.clone()).into(),
            account_registration,
        ));

        let fee_owned_by_other = AccountKeys::from_seed("FeeOwnedByOther").unwrap();
        AccountDid::<Runtime>::insert(&fee_owned_by_other.acct.public, other_did);
        let (fee_registration, _) = FeeAccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &fee_owned_by_other.acct,
            FEE_ASSET_ID,
            100,
            &owner_did_bytes,
        )
        .unwrap();
        let fee_registration = BatchedFeeAccountRegistrationProof {
            proofs: vec![fee_registration].try_into().unwrap(),
        };
        assert_noop!(
            Pallet::<Runtime>::register_fee_accounts(
                RawOrigin::Signed(owner.clone()).into(),
                fee_registration,
            ),
            Error::<Runtime>::NotAccountOwner
        );

        let account_owned_by_other = AccountKeys::from_seed("AccountOwnedByOther").unwrap();
        FeeAccountDid::<Runtime>::insert(&account_owned_by_other.acct.public, other_did);
        let account_registration = AccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &[account_owned_by_other],
            &owner_did_bytes,
        )
        .unwrap();
        assert_noop!(
            Pallet::<Runtime>::register_accounts(
                RawOrigin::Signed(owner.clone()).into(),
                account_registration,
            ),
            Error::<Runtime>::NotAccountOwner
        );

        let topup_keys = AccountKeys::from_seed("TopupAccountOwnedByOther").unwrap();
        let (_, fee_state) = FeeAccountRegistrationProof::<PolymeshLimits>::new(
            &mut rng,
            &topup_keys.acct,
            FEE_ASSET_ID,
            100,
            &other_did_bytes,
        )
        .unwrap();
        let mut fee_tree =
            ProverCurveTree::<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>::new(1)
                .unwrap();
        fee_tree.set_block_number(1).unwrap();
        fee_tree
            .insert(
                fee_state
                    .current_commitment()
                    .unwrap()
                    .as_leaf_value()
                    .unwrap(),
            )
            .unwrap();
        fee_tree.store_root().unwrap();
        let mut topups = [(&topup_keys.acct, 1, fee_state)];
        let topup = BatchedFeeAccountTopupProof::<PolymeshLimits>::new(
            &mut rng,
            &mut topups,
            &[],
            &fee_tree,
        )
        .unwrap();

        FeeAccountDid::<Runtime>::insert(&topup_keys.acct.public, other_did);
        assert_noop!(
            Pallet::<Runtime>::topup_fee_accounts(RawOrigin::Signed(owner).into(), topup,),
            Error::<Runtime>::NotAccountOwner
        );
    });
}

#[test]
fn confidential_assets_freeze_authority_and_leaf() {
    with_asset(|owner, mut state| {
        let unfrozen_leaf = state.commitment().unwrap();
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Issuer)
        );
        state.frozen = true;
        let frozen_leaf = state.commitment().unwrap();
        assert_ne!(frozen_leaf, unfrozen_leaf);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(frozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            false
        ));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(unfrozen_leaf)
        );

        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Root)
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, false),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(owner), state.asset_id, true),
            Error::<Runtime>::AssetFrozenByRoot
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(frozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            false
        ));
        assert_eq!(AssetFrozen::<Runtime>::get(state.asset_id), None);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(state.asset_id)),
            Some(unfrozen_leaf)
        );
        assert_ok!(Pallet::<Runtime>::set_asset_frozen(
            RawOrigin::Root.into(),
            state.asset_id,
            true
        ));
        assert_eq!(
            AssetFrozen::<Runtime>::get(state.asset_id),
            Some(FreezeOrigin::Root)
        );
    });
}

#[test]
fn confidential_assets_leaf_update_events() {
    with_asset(|owner, mut state| {
        for frozen in [true, false] {
            frame_system::Pallet::<Runtime>::reset_events();
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
                Some(owner),
                state.asset_id,
                frozen
            ));
            state.frozen = frozen;
            let expected: <Runtime as frame_system::Config>::RuntimeEvent =
                pallet_confidential_assets::Event::<Runtime>::AssetStateLeafUpdated {
                    leaf_index: LeafIndex::from(state.asset_id),
                    asset_leaf: state.commitment().unwrap(),
                }
                .into();
            let events = frame_system::Pallet::<Runtime>::events();
            assert_eq!(
                events
                    .iter()
                    .filter(|record| record.event == expected)
                    .count(),
                1
            );
            assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
                Some(owner),
                state.asset_id,
                frozen
            ));
            assert_eq!(frame_system::Pallet::<Runtime>::events(), events);
        }
    });
}

#[test]
fn confidential_assets_migrate_v0_keys_and_asset_leaf() {
    #[derive(Encode)]
    struct LegacyKeys {
        enc_keys: BTreeSet<EncryptionPublicKey>,
        mediators: BTreeMap<AccountPublicKey, u8>,
    }

    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let asset_id = 0;
        let owner = IdentityId::from(1u128);
        let mediator_a = AccountKeys::from_seed("LegacyMediatorA").unwrap();
        let mediator_b = AccountKeys::from_seed("LegacyMediatorB").unwrap();
        let enc_keys = BTreeSet::from([mediator_a.enc.public, mediator_b.enc.public]);
        let mediator_index = |key: EncryptionPublicKey| {
            enc_keys
                .iter()
                .position(|candidate| *candidate == key)
                .unwrap() as u8
        };
        let mediator_a_index = mediator_index(mediator_a.enc.public);
        let mediator_b_index = mediator_index(mediator_b.enc.public);
        let mediators = BTreeMap::from([
            (mediator_a.acct.public, mediator_a_index),
            (mediator_b.acct.public, mediator_b_index),
        ]);
        let old_keys = LegacyKeys {
            enc_keys,
            mediators,
        };
        let raw_keys = old_keys.encode();

        Details::<Runtime>::insert(
            asset_id,
            AssetDetails::<Runtime> {
                owner_did: owner,
                total_supply: 0,
                data: Default::default(),
            },
        );
        sp_io::storage::set(&Keys::<Runtime>::hashed_key_for(asset_id), &raw_keys);

        let mediator_pairs = [
            (mediator_a.acct.public, mediator_a.enc.public),
            (mediator_b.acct.public, mediator_b.enc.public),
        ];
        let expected = AssetState::new::<PolymeshLimits>(asset_id, &mediator_pairs, &[]).unwrap();
        let stale_key = AccountKeys::from_seed("LegacyStaleAuditor")
            .unwrap()
            .enc
            .public;
        let stale_leaf = AssetState::new::<PolymeshLimits>(asset_id, &[], &[stale_key])
            .unwrap()
            .commitment()
            .unwrap();
        AssetLeaves::<Runtime>::insert(LeafIndex::from(asset_id), stale_leaf);

        StorageVersion::new(0).put::<pallet_confidential_assets::Pallet<Runtime>>();
        let _migration_weight = Pallet::<Runtime>::migrate_to_v1();

        let migrated_keys = Keys::<Runtime>::get(asset_id).unwrap();
        assert_eq!(migrated_keys.enc_keys, expected.keys.enc_keys);
        assert_eq!(migrated_keys.mediators, expected.keys.mediators);
        assert_eq!(
            AssetLeaves::<Runtime>::get(LeafIndex::from(asset_id)),
            Some(expected.commitment().unwrap())
        );
        assert!(Pallet::<Runtime>::get_asset_curve_tree_root(1).is_ok());
        assert_eq!(
            StorageVersion::get::<pallet_confidential_assets::Pallet<Runtime>>(),
            StorageVersion::new(1)
        );
    });
}

#[test]
fn confidential_assets_migration_skips_invalid_legacy_keys_without_panic() {
    ExtBuilder::default().build().execute_with(|| {
        let asset_id = 0;
        Details::<Runtime>::insert(
            asset_id,
            AssetDetails::<Runtime> {
                owner_did: IdentityId::from(1u128),
                total_supply: 0,
                data: Default::default(),
            },
        );
        let invalid_keys = vec![0xff];
        sp_io::storage::set(&Keys::<Runtime>::hashed_key_for(asset_id), &invalid_keys);
        StorageVersion::new(0).put::<pallet_confidential_assets::Pallet<Runtime>>();

        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            Pallet::<Runtime>::migrate_to_v1()
        }));

        assert!(result.is_ok());
        assert_eq!(
            StorageVersion::get::<pallet_confidential_assets::Pallet<Runtime>>(),
            StorageVersion::new(1)
        );
        assert_eq!(
            sp_io::storage::get(&Keys::<Runtime>::hashed_key_for(asset_id))
                .map(|bytes| bytes.to_vec()),
            Some(invalid_keys)
        );
    });
}

#[test]
fn confidential_assets_freeze_rejects_non_owner_and_missing_asset() {
    with_asset(|_, state| {
        let other_did = IdentityId::from(2u128);
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(other_did), state.asset_id, true),
            Error::<Runtime>::NotAssetOwner
        );
        assert_noop!(
            Pallet::<Runtime>::base_set_asset_frozen(Some(other_did), state.asset_id, false),
            Error::<Runtime>::NotAssetOwner
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), 1, true),
            Error::<Runtime>::AssetMissing
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::Root.into(), 1, false),
            Error::<Runtime>::AssetMissing
        );
        assert_noop!(
            Pallet::<Runtime>::set_asset_frozen(RawOrigin::None.into(), state.asset_id, true),
            sp_runtime::DispatchError::BadOrigin
        );
    });
}

#[test]
fn confidential_assets_frozen_revealed_lookup_rejected() {
    with_asset(|owner, state| {
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(Pallet::<Runtime>::get_asset_keys_lookup(asset_ids).is_ok());
        assert_ok!(Pallet::<Runtime>::base_set_asset_frozen(
            Some(owner),
            state.asset_id,
            true
        ));
        let asset_ids = [state.asset_id].into_iter().collect();
        assert!(matches!(
            Pallet::<Runtime>::get_asset_keys_lookup(asset_ids),
            Err(Error::<Runtime>::AssetIsFrozen)
        ));
    });
}

fn submission_settlement(
    context: &[u8],
    legs: usize,
    revealed: bool,
) -> polymesh_dart::SettlementProof<PolymeshLimits> {
    use polymesh_dart::{
        curve_tree::{AssetTreeConfig, ProverCurveTree},
        LegBuilder, LegConfig, SettlementBuilder, ASSET_TREE_L, ASSET_TREE_M,
    };
    let sender = AccountKeys::from_seed("SubmissionSender").unwrap();
    let receiver = AccountKeys::from_seed("SubmissionReceiver").unwrap();
    let asset = AssetState::new::<()>(0, &[], &[]).unwrap();
    Keys::<Runtime>::insert(0, &asset.keys);
    let mut tree = ProverCurveTree::<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>::new(1).unwrap();
    tree.set_block_number(1).unwrap();
    tree.insert(asset.commitment().unwrap()).unwrap();
    tree.store_root().unwrap();
    Pallet::<Runtime>::store_asset_curve_tree_root(
        pallet_confidential_assets::TimestampedTreeRoot::new(tree.root().unwrap()),
    );
    let mut builder = SettlementBuilder::<PolymeshLimits>::new(b"SubmissionContext");
    for _ in 0..legs {
        builder.add_leg(LegBuilder {
            sender: sender.public_keys(),
            receiver: receiver.public_keys(),
            asset: asset.clone(),
            amount: 1,
            config: LegConfig {
                reveal_asset_id: revealed,
                ..Default::default()
            },
            public_enc_keys: vec![],
        });
    }
    builder
        .encrypt_and_prove_with_context(context, &mut thread_rng(), &tree)
        .unwrap()
}

#[test]
fn confidential_assets_create_settlement_enforces_runtime_leg_limit() {
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let account = AccountId::new([73; 32]);
        let proof = submission_settlement(&[], 11, false);

        assert_eq!(proof.legs.len(), 11);
        assert_noop!(
            Pallet::<Runtime>::create_settlement(RawOrigin::Signed(account).into(), proof),
            Error::<Runtime>::SettlementTooManyLegs
        );
    });
}

#[test]
fn confidential_assets_ordinary_submission_context() {
    use polymesh_dart::{BatchedProof, BatchedProofs, FeeAccountState};
    ExtBuilder::default().build().execute_with(|| {
        frame_system::Pallet::<Runtime>::set_block_number(1);
        let account = AccountId::new([72; 32]);
        let ordinary = submission_settlement(&[], 1, false);
        assert_ok!(Pallet::<Runtime>::create_settlement(
            RawOrigin::Signed(account.clone()).into(),
            ordinary,
        ));
        let ordinary = submission_settlement(&[], 2, false);
        let batch = BatchedProofs {
            proofs: vec![BatchedProof::CreateSettlement(ordinary)]
                .try_into()
                .unwrap(),
        };
        assert_ok!(Pallet::<Runtime>::submit_batched_proofs(
            RawOrigin::Signed(account.clone()).into(),
            batch,
        ));
        let keys = AccountKeys::from_seed("SubmissionFee").unwrap();
        let fee_state = FeeAccountState::new(&mut thread_rng(), &keys.acct.public, 0, 100).unwrap();
        let context = polymesh_dart::relayer_batch_proof_context(&fee_state.nullifier().unwrap());
        let bound = submission_settlement(&context, 1, false);
        assert_noop!(
            Pallet::<Runtime>::create_settlement(
                RawOrigin::Signed(account.clone()).into(),
                bound.clone(),
            ),
            Error::<Runtime>::InvalidProof
        );
        let batch = BatchedProofs {
            proofs: vec![BatchedProof::CreateSettlement(bound)]
                .try_into()
                .unwrap(),
        };
        assert_noop!(
            Pallet::<Runtime>::submit_batched_proofs(RawOrigin::Signed(account).into(), batch),
            Error::<Runtime>::InvalidProof
        );
    });
}

#[test]
fn confidential_assets_relayer_context_and_payment_persistence() {
    use codec::Encode;
    use frame_support::traits::Currency;
    use polymesh_dart::{
        curve_tree::{FeeAccountTreeConfig, ProverCurveTree},
        BatchedProof, BatchedProofs, FeeAccountRegistrationProof, FeePaymentWithBatchedProofs,
        FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FEE_ASSET_ID,
    };

    for (broadcast, invalid_batch, legs, revealed) in [
        (true, false, 1, false),
        (false, false, 2, false),
        (false, false, 1, true),
        (true, false, 2, true),
        (false, true, 1, false),
    ] {
        ExtBuilder::default().build().execute_with(|| {
            frame_system::Pallet::<Runtime>::set_block_number(1);
            let mut rng = thread_rng();
            let keys = AccountKeys::from_seed("RelayerContextFee").unwrap();
            let (_, mut fee_state) = FeeAccountRegistrationProof::<PolymeshLimits>::new(
                &mut rng,
                &keys.acct,
                FEE_ASSET_ID,
                10_000_000,
                &[0; 32],
            )
            .unwrap();
            let context = polymesh_dart::relayer_batch_proof_context(
                &fee_state.current_state.nullifier().unwrap(),
            );
            let settlement = submission_settlement(&context, legs, revealed);
            let settlement_ref = settlement.settlement_ref();
            let mut proofs = vec![BatchedProof::CreateSettlement(settlement.clone())];
            if invalid_batch {
                proofs.push(BatchedProof::CreateSettlement(settlement));
            }
            let batch = BatchedProofs::<PolymeshLimits> {
                proofs: proofs.try_into().unwrap(),
            };
            let mut tree = ProverCurveTree::<
                FEE_ACCOUNT_TREE_L,
                FEE_ACCOUNT_TREE_M,
                FeeAccountTreeConfig,
            >::new(1)
            .unwrap();
            tree.set_block_number(1).unwrap();
            tree.insert(
                fee_state
                    .current_commitment()
                    .unwrap()
                    .as_leaf_value()
                    .unwrap(),
            )
            .unwrap();
            tree.store_root().unwrap();
            Pallet::<Runtime>::store_fee_account_curve_tree_root(
                pallet_confidential_assets::TimestampedTreeRoot::new(tree.root().unwrap()),
            );
            let relayer = AccountId::new([73; 32]);
            let target = (!broadcast).then(|| relayer.encode());
            let payment = FeePaymentWithBatchedProofs::new(
                &mut rng,
                &keys.acct,
                batch,
                &mut fee_state,
                target.as_deref(),
                1_000_000,
                &tree,
            )
            .unwrap();
            assert_eq!(
                context,
                polymesh_dart::relayer_batch_proof_context(&payment.fee_payment.nullifier)
            );
            let batch_hash = payment.fee_payment_ctx(target.as_deref());
            let nullifier = payment.fee_payment.nullifier;
            let _ = pallet_balances::Pallet::<Runtime>::make_free_balance_be(
                &Pallet::<Runtime>::fee_account_id(),
                10_000_000,
            );
            let before = pallet_balances::Pallet::<Runtime>::free_balance(&relayer);
            assert_noop!(
                Pallet::<Runtime>::relayer_submit_batched_proofs(
                    RawOrigin::Signed(relayer.clone()).into(),
                    payment.clone(),
                ),
                Error::<Runtime>::RelayerCallMustBeTopLevel
            );
            assert_eq!(
                pallet_balances::Pallet::<Runtime>::free_balance(&relayer),
                before
            );
            RelayerSubmitExtensionMarker::<Runtime>::put(());
            let post = Pallet::<Runtime>::relayer_submit_batched_proofs(
                RawOrigin::Signed(relayer.clone()).into(),
                payment.clone(),
            )
            .unwrap();
            // A failed batch is charged the full declared weight.
            let expected_weight = (!invalid_batch).then(|| {
                SubstrateWeight::relayer_batched_proofs(
                    &payment.batched_proofs,
                    <Runtime as pallet_confidential_assets::Config>::MaxSettlementLegs::get(),
                )
            });
            assert_eq!(post.actual_weight, expected_weight);
            assert_eq!(
                pallet_balances::Pallet::<Runtime>::free_balance(&relayer),
                before + 1_000_000
            );
            let result = if invalid_batch {
                Err(Error::<Runtime>::SettlementAlreadyExists.into())
            } else {
                Ok(())
            };
            frame_system::Pallet::<Runtime>::assert_last_event(
                pallet_confidential_assets::Event::<Runtime>::RelayerBatchedProofs {
                    relayer: relayer.clone(),
                    batch_hash,
                    amount: 1_000_000,
                    batch_result: result,
                }
                .into(),
            );
            assert!(matches!(
                Pallet::<Runtime>::ensure_settlement_finalized(settlement_ref),
                Err(error) if error == if invalid_batch {
                    Error::<Runtime>::SettlementNotFound.into()
                } else {
                    Error::<Runtime>::SettlementNotExecuted.into()
                }
            ));
            assert!(matches!(
                Pallet::<Runtime>::ensure_fee_account_state_nullifier_unique(&nullifier),
                Err(Error::<Runtime>::NullifierAlreadyUsed)
            ));
        });
    }
}
