// This file is part of Substrate.

// Copyright (C) 2019-2020 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A collection of node-specific RPC methods.
//!
//! Since `substrate` core functionality makes no assumptions
//! about the modules used inside the runtime, so do
//! RPC methods defined in `sc-rpc` crate.
//! It means that `client/rpc` can't have any methods that
//! need some strong assumptions about the particular runtime.
//!
//! The RPCs available in this crate however can make some assumptions
//! about how the runtime is constructed and what FRAME pallets
//! are part of it. Therefore all node-runtime-specific RPCs can
//! be placed here or imported from corresponding FRAME RPC definitions.

#![warn(missing_docs)]

use polymesh_primitives::{
    AccountId, Balance, Block, BlockNumber, Hash, IdentityId, Index, Moment, Ticker,
};
use sc_client_api::AuxStore;
use sc_consensus_babe::{Config, Epoch};
use sc_consensus_epochs::SharedEpochChanges;
use sc_finality_grandpa::{
    FinalityProofProvider, GrandpaJustificationStream, SharedAuthoritySet, SharedVoterState,
};
use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sc_transaction_pool_api::TransactionPool;
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use sp_keystore::SyncCryptoStorePtr;
use std::sync::Arc;

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: SyncCryptoStorePtr,
}

/// Extra dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies.
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// A copy of the chain spec.
    pub chain_spec: Box<dyn sc_chain_spec::ChainSpec>,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
}

/// A IO handler that uses all Full RPC extensions.
pub type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Instantiate all Full RPC extensions.
pub fn create_full<C, P, UE, SC, B>(
    deps: FullDeps<C, P, SC, B>,
) -> Result<jsonrpc_core::IoHandler<sc_rpc_api::Metadata>, Box<dyn std::error::Error + Send + Sync>>
where
    C: ProvideRuntimeApi<Block>
        + HeaderBackend<Block>
        + AuxStore
        + HeaderMetadata<Block, Error = BlockChainError>
        + Sync
        + Send
        + 'static,
    C::Api: substrate_frame_rpc_system::AccountNonceApi<Block, AccountId, Index>,
    C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber, Hash>,
    C::Api: node_rpc::transaction_payment::TransactionPaymentRuntimeApi<Block, UE>,
    C::Api: pallet_staking_rpc::StakingRuntimeApi<Block>,
    C::Api: node_rpc::pips::PipsRuntimeApi<Block, AccountId>,
    C::Api: node_rpc::identity::IdentityRuntimeApi<Block, IdentityId, Ticker, AccountId, Moment>,
    C::Api: pallet_protocol_fee_rpc::ProtocolFeeRuntimeApi<Block>,
    C::Api: node_rpc::asset::AssetRuntimeApi<Block, AccountId>,
    C::Api: pallet_group_rpc::GroupRuntimeApi<Block>,
    C::Api: node_rpc::compliance_manager::ComplianceManagerRuntimeApi<Block, AccountId>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + 'static,
    UE: codec::Codec + Send + Sync + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
    use node_rpc::compliance_manager::{ComplianceManager, ComplianceManagerApi};
    use node_rpc::{
        asset::{Asset, AssetApi},
        identity::{Identity, IdentityApi},
        pips::{Pips, PipsApi},
        transaction_payment::{TransactionPayment, TransactionPaymentApi},
    };
    use pallet_contracts_rpc::{Contracts, ContractsApi};
    use pallet_group_rpc::{Group, GroupApi};
    use pallet_protocol_fee_rpc::{ProtocolFee, ProtocolFeeApi};
    use pallet_staking_rpc::{Staking, StakingApi};
    use sc_consensus_babe_rpc::BabeRpcHandler;
    use sc_finality_grandpa_rpc::GrandpaRpcHandler;
    use substrate_frame_rpc_system::{FullSystem, SystemApi};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        select_chain,
        chain_spec,
        deny_unsafe,
        babe,
        grandpa,
    } = deps;

    let BabeDeps {
        keystore,
        babe_config,
        shared_epoch_changes,
    } = babe;
    let GrandpaDeps {
        shared_voter_state,
        shared_authority_set,
        justification_stream,
        subscription_executor,
        finality_provider,
    } = grandpa;

    io.extend_with(SystemApi::to_delegate(FullSystem::new(
        client.clone(),
        pool,
        deny_unsafe,
    )));
    // Making synchronous calls in light client freezes the browser currently,
    // more context: https://github.com/PolymeshAssociation/substrate/pull/3480
    // These RPCs should use an asynchronous caller instead.
    io.extend_with(ContractsApi::to_delegate(Contracts::new(client.clone())));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
        BabeRpcHandler::new(
            client.clone(),
            shared_epoch_changes.clone(),
            keystore,
            babe_config,
            select_chain,
            deny_unsafe,
        ),
    ));
    io.extend_with(sc_finality_grandpa_rpc::GrandpaApi::to_delegate(
        GrandpaRpcHandler::new(
            shared_authority_set.clone(),
            shared_voter_state,
            justification_stream,
            subscription_executor,
            finality_provider,
        ),
    ));

    io.extend_with(sc_sync_state_rpc::SyncStateRpcApi::to_delegate(
        sc_sync_state_rpc::SyncStateRpcHandler::new(
            chain_spec,
            client.clone(),
            shared_authority_set,
            shared_epoch_changes,
        )?,
    ));
    io.extend_with(StakingApi::to_delegate(Staking::new(client.clone())));
    io.extend_with(PipsApi::to_delegate(Pips::new(client.clone())));
    io.extend_with(IdentityApi::to_delegate(Identity::new(client.clone())));
    io.extend_with(ProtocolFeeApi::to_delegate(ProtocolFee::new(
        client.clone(),
    )));
    io.extend_with(AssetApi::to_delegate(Asset::new(client.clone())));
    io.extend_with(GroupApi::to_delegate(Group::from(client.clone())));
    io.extend_with(ComplianceManagerApi::to_delegate(ComplianceManager::new(
        client,
    )));

    Ok(io)
}
