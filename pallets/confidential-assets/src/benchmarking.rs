// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh

use frame_benchmarking::benchmarks;
use sp_consensus_babe::Slot;
use sp_std::vec;
use sp_std::vec::Vec;

use rand_chacha::ChaCha20Rng as Rng;
use rand_core::SeedableRng;

use polymesh_dart::{
    curve_tree::{CurveTreeLookup, MultiLeafPathAndRoot},
    FeeAccountState, LegBuilder, SettlementBuilder, ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_L,
};
use polymesh_worker_protocol_dart_v1::{GenerateDartProofRequest, GenerateDartProofResponse};

use polymesh_primitives::erc20::{Name, Symbol};

use crate::testing::*;
use crate::*;

fn set_timestamp<T: Config + pallet_babe::Config>(n: T::Moment)
where
    Slot: From<T::Moment>,
{
    let duration = pallet_babe::Pallet::<T>::slot_duration();
    pallet_babe::CurrentSlot::<T>::set(Slot::from(n / duration));
    pallet_timestamp::Pallet::<T>::set_timestamp(n.into());
}

fn init_curve_trees<T: Config>() {
    // Init the session.
    init_block::<T>();

    let mut rng = Rng::from_seed([42u8; 32]);
    // Create an Confidential user.
    let user = DartUser::<T>::new("FeeAccountUser");
    let keys = user.keys();
    let pub_keys = user.public_keys();
    let asset_id = 0;

    // Initialize one asset leaf in the asset curve tree.
    let auditor_keys = BoundedBTreeSet::new();
    let mut mediator_keys = BoundedBTreeMap::new();
    mediator_keys
        .try_insert(pub_keys.acct, pub_keys.enc)
        .expect("Failed to push 'mediator' keys");
    Pallet::<T>::update_asset_leaf(user.did(), asset_id, &mediator_keys, &auditor_keys, true)
        .expect("Failed to update asset leaf");

    // Generate new fee account state commitment.
    {
        // Generate an initial fee account state for the fee asset.
        let account_state = FeeAccountState::new(&mut rng, &keys.acct.public, FEE_ASSET_ID, 42)
            .expect("Failed to create fee account state");
        let commitment = account_state
            .commitment()
            .expect("Failed to get fee account state commitment");
        let nullifier = account_state
            .nullifier()
            .expect("Failed to get fee account state nullifier");
        Pallet::<T>::insert_fee_account_leaf(commitment, Some(nullifier))
            .expect("Failed to insert fee account leaf");
    }

    // Generate an initial account state for each asset.
    {
        let (account_state, _rho_rand) = keys
            .account_state(asset_id, 0, &[42])
            .expect("Failed to create account state");
        let account_commitment = account_state
            .commitment()
            .expect("Failed to get account state commitment");
        let nullifier = account_state
            .nullifier()
            .expect("Failed to get account state nullifier");
        Pallet::<T>::insert_account_leaf(account_commitment, Some(nullifier))
            .expect("Failed to insert account leaf");
    }

    // Initialize the current curve tree roots.
    Pallet::<T>::update_account_curve_tree_root();
    Pallet::<T>::update_fee_account_curve_tree_root();
}

benchmarks! {
    where_clause { where T: Config + pallet_babe::Config, Slot: From<T::Moment> }

    generate_and_save_dart_params {
    }: {
        // Make sure to unload the previously loaded parameters, to force generating new parameters for the benchmarks.
        polymesh_dart::init::unload_params();
        Pallet::<T>::generate_and_save_dart_params();
    }

    session_overhead {
    }: {
        Pallet::<T>::start_session();
        Pallet::<T>::end_session();
    }

    curve_tree_min_update {
        init_curve_trees::<T>();

        // Move current timestamp forward by the minimum curve tree root update interval to make sure the curve tree root will be updated.
        let min_update_interval = T::MinCurveTreeRootUpdateInterval::get();
        let now = pallet_timestamp::Pallet::<T>::get();
        set_timestamp::<T>(now + min_update_interval);
    }: {
        Pallet::<T>::curve_tree_min_update();
    }

    root_pruning {
        let r in 0 .. MAX_ROOT_PRUNING_BLOCKS as u32;

        init_curve_trees::<T>();
        let max_age = T::MaxAssetCurveTreeRootAge::get();
        let mut timestamp = pallet_timestamp::Pallet::<T>::get();
        let mut block_number = frame_system::Pallet::<T>::block_number();
        for _ in 0..=r {
            // Move time and block number forward to make the current curve tree root old enough for pruning.
            timestamp = timestamp.saturating_add(max_age);
            set_timestamp::<T>(timestamp);
            block_number = block_number.saturating_add(1u32.into());
            frame_system::Pallet::<T>::set_block_number(block_number);

            // Record new curve tree root to create more roots for pruning.
            Pallet::<T>::curve_tree_min_update();
        }

        // Move block number forward past the recent blocks to keep, to make sure the old roots will be pruned.
        timestamp = timestamp.saturating_add(max_age);
        block_number = block_number.saturating_add(1u32.into());

        let stop_at = block_number;
    }: {
        Pallet::<T>::asset_curve_tree_pruning(timestamp, stop_at);
    }

    update_account_curve_tree_root {
        // Number of leaves to insert.
        let l in 0 .. (ACCOUNT_TREE_L as u32 + 1);

        // Create an asset issuer and create an asset.
        let asset_issuer = DartUser::<T>::new("AssetIssuer");

        let keys = asset_issuer.keys();

        // Generate new account state commitments.
        let mut account_commitments = Vec::with_capacity(l as usize);
        for asset_id in 0..l {
            // Generate an initial account state for each asset.
            let (account_state, _rho_rand) = keys.account_state(asset_id, 0, &[42]).expect("Failed to create account state");
            let account_state_commitment = account_state.commitment().expect("Failed to get account state commitment");
            let nullifier = account_state.nullifier().expect("Failed to get account state nullifier");
            account_commitments.push((account_state_commitment, nullifier));
        }

        // Insert the account state commitments into the account curve tree.
        for (account_commitment, nullifier) in account_commitments {
            Pallet::<T>::insert_account_leaf(account_commitment, Some(nullifier))
                .expect("Failed to insert account leaf");
        }
    }: {
        Pallet::<T>::update_account_curve_tree_root();
    }

    update_fee_account_curve_tree_root {
        // Number of leaves to insert.
        let l in 0 .. (FEE_ACCOUNT_TREE_L as u32 + 1);

        let mut rng = Rng::from_seed([42u8; 32]);
        // Create an Confidential user.
        let user = DartUser::<T>::new("FeeAccountUser");

        let keys = user.keys();

        // Generate new account state commitments.
        let mut fee_account_commitments = Vec::with_capacity(l as usize);
        for asset_id in 0..l {
            // Generate an initial fee account state for the fee asset.
            let account_state = FeeAccountState::new(&mut rng, &keys.acct.public, FEE_ASSET_ID, 42)
                .expect("Failed to create fee account state");
            let account_state_commitment = account_state.commitment().expect("Failed to get fee account state commitment");
            let nullifier = account_state.nullifier().expect("Failed to get fee account state nullifier");
            fee_account_commitments.push((account_state_commitment, nullifier));
        }

        // Insert the account state commitments into the account curve tree.
        for (account_commitment, nullifier) in fee_account_commitments {
            Pallet::<T>::insert_fee_account_leaf(account_commitment, Some(nullifier))
                .expect("Failed to insert fee account leaf");
        }
    }: {
        Pallet::<T>::update_fee_account_curve_tree_root();
    }

    register_accounts {
        // Number of proofs to batch.
        let k in 0 .. <T as Config>::MaxKeysPerRegProof::get();

        init_block::<T>();
        // Create an asset issuer and create an asset.
        let user = DartUser::<T>::new("Batched User");

        let mut accounts = Vec::with_capacity(k as usize);
        for idx in 0..k {
            // Create a new account with the same user but different keys.
            let account = user.new_account("Batching account", idx);
            accounts.push(account.keys());
        }
        // Generate the account registration proof.
        let req = GenerateDartProofRequest::AccountRegistration {
            accounts,
            did: user.did().into(),
        };

        let proof = if let Ok(GenerateDartProofResponse::AccountRegistration { proof }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate account registration proof");
        };
    }: _(user.raw_origin(), proof)

    register_encryption_keys {
        // Number of proofs to batch.
        let k in 0 .. <T as Config>::MaxKeysPerRegProof::get();

        init_block::<T>();
        // Create an asset issuer and create an asset.
        let user = DartUser::<T>::new("Batched Key User");

        let mut encryption_keys = Vec::with_capacity(k as usize);
        for idx in 0..k {
            // Create a new account with the same user but different keys.
            let account = user.new_account("Batching key account", idx);
            encryption_keys.push(account.keys().enc.clone());
        }
        // Generate the encryption key registration proof.
        let req = GenerateDartProofRequest::EncryptionKeyRegistration {
            keys: encryption_keys,
            did: user.did().into(),
        };

        let proof = if let Ok(GenerateDartProofResponse::EncryptionKeyRegistration { proof }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate encryption key registration proof");
        };
    }: _(user.raw_origin(), proof)

    register_fee_accounts {
        // Number of proofs to batch.
        let p in 0 .. <T as Config>::MaxFeeAccountRegProofs::get();

        init_block::<T>();
        // Create an Confidential user for the fee account.
        let user = DartUser::<T>::new("Batched FeeAccountUser");

        let mut accounts = Vec::with_capacity(p as usize);
        for idx in 0..p {
            // Create a new account with the same user but different keys.
            let account = user.new_account("Batching fee account", idx);

            accounts.push((account.keys().acct.clone(), FEE_ASSET_ID, 42));
        }

        // Generate the fee accounts registration proof.
        let req = GenerateDartProofRequest::BatchedFeeAccountRegistration {
            accounts,
            did: user.did().into(),
        };

        let proof = if let Ok(GenerateDartProofResponse::BatchedFeeAccountRegistration { proof, .. }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate fee account registration proofs");
        };
    }: _(user.raw_origin(), proof)

    create_asset {
        init_block::<T>();

        // Create an asset issuer and create an asset.
        let asset_issuer = DartUser::<T>::new("AssetIssuer");

        // Register the asset issuer's account.
        asset_issuer.register_account();

        // Create the maximum number of mediators.
        let auditor_keys = BoundedBTreeSet::new();
        let mut mediator_keys = BoundedBTreeMap::new();
        for i in 0..<T as Config>::MaxAssetMediators::get() {
            let mediator = DartUser::<T>::auditor_user("Mediator", 0, i);
            mediator.register_account();
            let med_keys = mediator.public_keys();
            mediator_keys
                .try_insert(med_keys.acct, med_keys.enc)
                .expect("Failed to push mediator keys");
        }

        // Asset Data.
        let data = b"Test Asset"
            .to_vec()
            .try_into()
            .expect("Failed to convert asset data to AssetData");
        // Asset Name.
        let name = Name::test_asset(0);
        // Asset Symbol.
        let symbol = Symbol::test_asset(0);
        // Asset Decimals.
        let decimals = 2u8;
    }: _(asset_issuer.raw_origin(), name, symbol, decimals, mediator_keys, auditor_keys, data)

    register_account_assets {
        // Number of proofs to batch.
        let p in 0 .. <T as Config>::MaxAccountAssetRegProofs::get();

        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create an asset issuer and create an asset.
        let user = DartUser::<T>::asset_user("Batching user", 0, 0);

        let mut accounts = Vec::with_capacity(p as usize);
        let mut account_assets = Vec::with_capacity(p as usize);
        for idx in 0..p {
            // Create asset issuer and an asset.
            let asset = DartTestAsset::<T>::new(&mut off_chain, "Test Batch reg", idx, 0, 1, None);

            // Create a new account with the same user but different keys.
            let account = user.new_account("Batching account", idx);

            accounts.push(account.keys());
            account_assets.push((account.keys(), asset.id, 0));
        }

        // Register all the accounts first.
        user.register_accounts(accounts);

        // Generate the account registration proof.
        let req = GenerateDartProofRequest::BatchedAccountAssetRegistration {
            account_assets,
            did: user.did().into(),
        };

        let proof = if let Ok(GenerateDartProofResponse::BatchedAccountAssetRegistration { proof, .. }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate account asset registration proofs");
        };
    }: _(user.raw_origin(), proof)

    mint_asset {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, None);

        // Asset issuer registers the asset.
        asset.issuer.register_account_asset(asset.id);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Mint asset proof.
        let amount = 1000;
        let (proof, _) = asset.issuer.mint_asset_proof(&off_chain, asset.id, amount);

    }: _(asset.issuer.raw_origin(), proof)

    topup_fee_accounts {
        // Number of proofs to batch.
        let p in 0 .. <T as Config>::MaxFeeAccountTopupProofs::get();

        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create an Confidential user for the fee account.
        let user = DartUser::<T>::new("Batched FeeAccountUser topup");

        let mut accounts = Vec::with_capacity(p as usize);
        let mut account_registrations = Vec::with_capacity(p as usize);
        // Make sure at least one fee account is created when p=0.
        let min_p = if p == 0 { 1 } else { p };
        for idx in 0..min_p {
            // Create a new account with the same user but different keys.
            let account = user.new_account("Batching fee account topup", idx);

            accounts.push(account.keys().acct.clone());
            account_registrations.push((account.keys().acct.clone(), FEE_ASSET_ID, 42));
        }

        // First register the fee accounts.
        let states = user.register_fee_accounts(account_registrations);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Top-up each fee account.
        let mut topups = Vec::with_capacity(p as usize);
        let mut paths = Vec::new();
        for (acct_key, state) in accounts.into_iter().zip(states.into_iter()) {
            let current_state_commitment = state
                .current_commitment()
                .expect("current commitment")
                .as_leaf_value()
                .expect("as leaf value");
            topups.push((acct_key, 100u64, state));

            paths.push(off_chain.fee_account_tree
                .get_path_and_root(current_state_commitment)
                .expect("Failed to get path to leaf"));
        }

        // Generate the fee accounts topup proof.
        let proof = if p > 0 {
            let req = GenerateDartProofRequest::BatchedFeeAccountTopup {
                topups,
                paths: MultiLeafPathAndRoot::from_paths(paths).expect("Failed to create MultiLeafPathAndRoot"),
                did: user.did().into(),
            };

            if let Ok(GenerateDartProofResponse::BatchedFeeAccountTopup { proof, .. }) = req.generate(Pallet::<T>::session_id()?) {
                proof
            } else {
                panic!("Failed to generate fee account topup proofs");
            }
        } else {
            BatchedFeeAccountTopupProof {
                root_block: off_chain.fee_account_tree.get_block_number().expect("Failed to get block number"),
                proofs: Default::default(),
            }
        };
    }: _(user.raw_origin(), proof)

    verify_fee_payment {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create an Confidential user for the fee account.
        let user = DartUser::<T>::new("Batched FeeAccountUser payment");

        // Create a new account with the same user but different keys.
        let account = user.keys().acct.clone();

        // First register the fee account.
        let account_state = user.register_fee_account(100);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        let current_state_commitment = account_state
            .current_commitment()
            .expect("current commitment")
            .as_leaf_value()
            .expect("as leaf value");

        let path = off_chain.fee_account_tree
                .get_path_and_root(current_state_commitment)
                .expect("Failed to get path to leaf");

        // Generate the fee payment proof.
        let amount = 42u64;
        let batch_tx_fee = Pallet::<T>::amount_to_balance(amount).expect("Failed to convert amount to balance");
        let batch_hash = ProofHash([42u8; 32]);
        let req = GenerateDartProofRequest::FeeAccountPayment {
            ctx: batch_hash,
            account,
            amount,
            path,
            account_state,
        };

        let proof = if let Ok(GenerateDartProofResponse::FeeAccountPayment { proof, .. }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate fee account payment proof");
        };
        let relayer = user.account();
    }: {
        Pallet::<T>::verify_fee_payment(
            relayer,
            batch_tx_fee,
            batch_hash,
            proof,
        ).expect("Failed to verify fee payment proof");
    }

    create_settlement {
        // Number of legs in transaction.
        let l in 0 .. <T as Config>::MaxSettlementLegs::get();

        // Set skip verify flag to true to speed up benchmark setup.
        set_skip_verify::<T>(true);

        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Venue user.
        let venue = DartUser::<T>::new("Venue");

        // Create a settlement to transfer some assets from the issuer to the investor.
        let mut settlement = SettlementBuilder::<PolymeshLimits>::new(b"Test");

        for asset_idx in 0..l {
            // Create asset issuer and an asset.
            let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", asset_idx, 2, 0, Some(1000));
            let asset_state = asset.asset_state();

            // Create investor.
            let user = asset.create_investor();

            settlement.add_leg(LegBuilder {
                sender: asset.issuer.public_keys(),
                receiver: user.public_keys(),
                asset: asset_state,
                amount: 500,
                config: Default::default(),
                public_enc_keys: vec![],
            })
        }

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        let mut paths = Vec::new();
        for leg in &settlement.legs {
            paths.push(off_chain
                .asset_tree
                .get_path_and_root_by_index(leg.asset.asset_id as LeafIndex)
                .expect("Failed to get path to leaf"));
        }

        // Make sure there is at least one leave path.
        if paths.is_empty() {
            let asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 2, 0, Some(1000));
            let path = off_chain
                .asset_tree
                .get_path_and_root_by_index(asset.id as LeafIndex)
                .expect("Failed to get path to leaf");
            paths.push(path);
        }

        // Generate the settlement proof.
        let req = GenerateDartProofRequest::CreateSettlement {
            paths: MultiLeafPathAndRoot::from_paths(paths).expect("Failed to create MultiLeafPathAndRoot"),
            settlement,
        };
        let proof = if let Ok(GenerateDartProofResponse::CreateSettlement { proof }) = req.generate(Pallet::<T>::session_id()?) {
            proof
        } else {
            panic!("Failed to generate create settlement proof");
        };

        // Reset skip verify flag.
        set_skip_verify::<T>(false);

    }: _(venue.raw_origin(), proof)

    sender_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Generate the sender's affirmation proof.
        let (proof, _) = leg.sender.sender_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.sender.raw_origin(), proof)

    receiver_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Generate the receiver's affirmation proof.
        let (proof, _) = leg.receiver.receiver_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.receiver.raw_origin(), proof)

    instant_sender_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // The receiver affirms the leg.
        leg.receiver_affirmation(&mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Generate the sender's instant affirmation proof.
        let (proof, _) = leg.sender.instant_sender_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.sender.raw_origin(), proof)

    instant_receiver_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // The sender affirms the leg.
        leg.sender_affirmation(&mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Generate the receiver's affirmation proof.
        let (proof, _) = leg.receiver.instant_receiver_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.receiver.raw_origin(), proof)

    mediator_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 1, 0, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = &settlement.legs[0];
        let mediator = leg.mediators[0].clone();

        // Generate the mediator's affirmation proof.
        let proof = mediator.mediator_affirmation_proof(&off_chain, leg.leg_ref, true, Some((leg.asset_id, leg.amount)));
    }: _(mediator.raw_origin(), proof)

    receiver_claim {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Affirm all the legs in the settlement.
        settlement.affirm_legs(&mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Generate the receiver's claim proof.
        let (proof, _) = leg.receiver.receiver_claim_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.receiver.raw_origin(), proof)

    sender_update_counter {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 0, 1, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Affirm all the legs in the settlement.
        settlement.affirm_legs(&mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Generate the sender's update counter proof.
        let (proof, _) = leg.sender.sender_counter_update_proof(&off_chain, leg.leg_ref, leg.asset_id);
    }: _(leg.sender.raw_origin(), proof)

    sender_revert_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 1, 0, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Sender affirms the leg.
        leg.sender_affirmation(&mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Mediator rejects the leg.  The settlement will go into a rejected state.
        leg.mediator_affirmation(&off_chain, false);

        // Generate the sender's revert affirmation proof.
        let (proof, _) = leg.sender.sender_revert_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id, leg.amount);
    }: _(leg.sender.raw_origin(), proof)

    receiver_revert_affirmation {
        // Offchain prover state.
        let mut off_chain = OffchainProverState::<T>::new();

        // Create asset issuer and an asset.
        let mut asset = DartTestAsset::<T>::new(&mut off_chain, "Test Asset", 0, 1, 0, Some(1000));

        // Create settlement and one investor.
        let (settlement, _) =
            DartSettlementState::create_investors_and_fund(&mut asset, 1, 500, &mut off_chain);

        // Get the first leg of the settlement.
        let leg = settlement.legs[0].clone();

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Receiver affirms the leg.
        leg.receiver_affirmation(&mut off_chain);

        // Update the curve tree roots.
        off_chain.apply_new_leaves();

        // Mediator rejects the leg.  The settlement will go into a rejected state.
        leg.mediator_affirmation(&off_chain, false);

        // Generate the receiver's revert affirmation proof.
        let (proof, _) = leg.receiver.receiver_revert_affirmation_proof(&off_chain, leg.leg_ref, leg.asset_id);
    }: _(leg.receiver.raw_origin(), proof)
}
