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

use std::sync::Arc;

use polymesh_primitives::{
    AccountId, Balance, Block, BlockNumber, Hash, IdentityId, Index as Nonce, Moment, SecondaryKey,
    Signatory, Ticker,
};
use sc_client_api::light::{Fetcher, RemoteBlockchain};
use sc_consensus_babe::Epoch;
use sc_finality_grandpa::FinalityProofProvider;
use sc_rpc::{DenyUnsafe, SubscriptionTaskExecutor};
use sp_api::ProvideRuntimeApi;
use sp_block_builder::BlockBuilder;
use sp_blockchain::{Error as BlockChainError, HeaderBackend, HeaderMetadata};
use sp_consensus::SelectChain;
use sp_consensus_babe::BabeApi;
use txpool_api::TransactionPool;

/// A type representing all RPC extensions.
pub type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

/// Light client extra dependencies.
pub struct LightDeps<C, F, P> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// Remote access to the blockchain (async).
    pub remote_blockchain: Arc<dyn RemoteBlockchain<Block>>,
    /// Fetcher instance.
    pub fetcher: Arc<F>,
}

/// Extra dependencies for BABE.
pub struct BabeDeps {
    /// BABE protocol config.
    pub babe_config: sc_consensus_babe::Config,
    /// BABE pending epoch changes.
    pub shared_epoch_changes: sc_consensus_epochs::SharedEpochChanges<Block, Epoch>,
    /// The keystore that manages the keys of the node.
    pub keystore: sc_keystore::KeyStorePtr,
}

/// Dependencies for GRANDPA
pub struct GrandpaDeps<B> {
    /// Voting round info.
    pub shared_voter_state: sc_finality_grandpa::SharedVoterState,
    /// Authority set info.
    pub shared_authority_set: sc_finality_grandpa::SharedAuthoritySet<Hash, BlockNumber>,
    /// Receives notifications about justification events from Grandpa.
    pub justification_stream: sc_finality_grandpa::GrandpaJustificationStream<Block>,
    /// Executor to drive the subscription manager in the Grandpa RPC handler.
    pub subscription_executor: SubscriptionTaskExecutor,
    /// Finality proof provider.
    pub finality_provider: Arc<FinalityProofProvider<B, Block>>,
}

/// Full client dependencies
pub struct FullDeps<C, P, SC, B> {
    /// The client instance to use.
    pub client: Arc<C>,
    /// Transaction pool instance.
    pub pool: Arc<P>,
    /// The SelectChain Strategy
    pub select_chain: SC,
    /// Whether to deny unsafe calls
    pub deny_unsafe: DenyUnsafe,
    /// BABE specific dependencies.
    pub babe: BabeDeps,
    /// GRANDPA specific dependencies.
    pub grandpa: GrandpaDeps<B>,
}

/// Instantiate all RPC extensions.
pub fn create_full<C, P, UE, SC, B>(deps: FullDeps<C, P, SC, B>) -> RpcExtension
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block> + HeaderMetadata<Block, Error = BlockChainError>,
    C: Send + Sync + 'static,
    C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: pallet_contracts_rpc::ContractsRuntimeApi<Block, AccountId, Balance, BlockNumber>,
    // C::Api: pallet_transaction_payment_rpc::TransactionPaymentRuntimeApi<Block, Balance, UE>,
    C::Api: node_rpc::transaction_payment::TransactionPaymentRuntimeApi<Block, Balance, UE>,
    C::Api: pallet_staking_rpc::StakingRuntimeApi<Block>,
    C::Api: node_rpc::pips::PipsRuntimeApi<Block, AccountId, Balance>,
    C::Api: node_rpc::identity::IdentityRuntimeApi<
        Block,
        IdentityId,
        Ticker,
        AccountId,
        SecondaryKey<AccountId>,
        Signatory<AccountId>,
        Moment,
    >,
    C::Api: pallet_protocol_fee_rpc::ProtocolFeeRuntimeApi<Block>,
    C::Api: node_rpc::asset::AssetRuntimeApi<Block, AccountId, Balance>,
    C::Api: pallet_group_rpc::GroupRuntimeApi<Block>,
    C::Api: node_rpc::compliance_manager::ComplianceManagerRuntimeApi<Block, AccountId, Balance>,
    C::Api: BabeApi<Block>,
    C::Api: BlockBuilder<Block>,
    P: TransactionPool + Sync + Send + 'static,
    UE: codec::Codec + Send + Sync + 'static,
    SC: SelectChain<Block> + 'static,
    B: sc_client_api::Backend<Block> + Send + Sync + 'static,
    B::State: sc_client_api::backend::StateBackend<sp_runtime::traits::HashFor<Block>>,
{
    use frame_rpc_system::{FullSystem, SystemApi};
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
    use sc_finality_grandpa_rpc::{GrandpaApi, GrandpaRpcHandler};

    let mut io = jsonrpc_core::IoHandler::default();
    let FullDeps {
        client,
        pool,
        select_chain,
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
    io.extend_with(ContractsApi::to_delegate(Contracts::new(client.clone())));
    io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
        client.clone(),
    )));
    io.extend_with(sc_consensus_babe_rpc::BabeApi::to_delegate(
        BabeRpcHandler::new(
            client.clone(),
            shared_epoch_changes,
            keystore,
            babe_config,
            select_chain,
            deny_unsafe,
        ),
    ));
    io.extend_with(GrandpaApi::to_delegate(GrandpaRpcHandler::new(
        shared_authority_set,
        shared_voter_state,
        justification_stream,
        subscription_executor,
        finality_provider,
    )));
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

    io
}

/// Instantiate all RPC extensions for light node.
pub fn create_light<C, P, F, UE>(deps: LightDeps<C, F, P>) -> RpcExtension
where
    C: ProvideRuntimeApi<Block>,
    C: HeaderBackend<Block>,
    C: Send + Sync + 'static,
    C::Api: frame_rpc_system::AccountNonceApi<Block, AccountId, Nonce>,
    C::Api: node_rpc::transaction_payment::TransactionPaymentRuntimeApi<Block, Balance, UE>,
    P: TransactionPool + Sync + Send + 'static,
    F: Fetcher<Block> + 'static,
    UE: codec::Codec + Send + Sync + 'static,
{
    use frame_rpc_system::{LightSystem, SystemApi};

    let LightDeps {
        client,
        pool,
        remote_blockchain,
        fetcher,
    } = deps;
    let mut io = jsonrpc_core::IoHandler::default();
    io.extend_with(SystemApi::<Hash, AccountId, Nonce>::to_delegate(
        LightSystem::new(client, remote_blockchain, fetcher, pool),
    ));
    io
}
