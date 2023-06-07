//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// pub use crate::chain_spec::{
//     testnet::ChainSpec as TestnetChainSpec,
// };
pub use codec::Codec;
use futures::stream::StreamExt;
use polymesh_node_rpc as node_rpc;
pub use polymesh_primitives::{
    crypto::native_schnorrkel, host_functions::native_rng::native_rng, AccountId, Balance, Block,
    BlockNumber, Hash, IdentityId, Index as Nonce, Moment, Ticker,
};
pub use polymesh_runtime_develop;
pub use polymesh_runtime_mainnet;
pub use polymesh_runtime_testnet;
use prometheus_endpoint::Registry;
pub use sc_client_api::backend::Backend;
use sc_client_api::BlockBackend;
pub use sc_consensus::LongestChain;
use sc_consensus_slots::SlotProportion;
use sc_executor::NativeElseWasmExecutor;
pub use sc_executor::{NativeExecutionDispatch, RuntimeVersionOf};
use sc_network::NetworkService;
use sc_network_common::{protocol::event::Event, service::NetworkEventStream};
use sc_service::{
    config::Configuration, error::Error as ServiceError, RpcHandlers, TaskManager, WarpSyncParams,
};
pub use sc_service::{
    config::{PrometheusConfig, Role},
    ChainSpec, Error, PruningMode, RuntimeGenesis, TFullBackend, TFullCallExecutor, TFullClient,
    TransactionPoolOptions,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
pub use sp_api::{ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi, StateBackend};
pub use sp_consensus::SelectChain;
pub use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

/// Known networks based on name.
pub enum Network {
    Mainnet,
    Testnet,
    Other,
}

pub trait IsNetwork {
    fn network(&self) -> Network;
}

impl IsNetwork for dyn ChainSpec {
    fn network(&self) -> Network {
        let name = self.name();
        if name.starts_with("Polymesh Mainnet") {
            Network::Mainnet
        } else if name.starts_with("Polymesh Testnet") {
            Network::Testnet
        } else {
            Network::Other
        }
    }
}

macro_rules! native_executor_instance {
    ($exec:ident, $module:ident, $ehf:ty) => {
        pub struct $exec;
        impl NativeExecutionDispatch for $exec {
            type ExtendHostFunctions = $ehf;

            fn dispatch(method: &str, data: &[u8]) -> Option<Vec<u8>> {
                $module::api::dispatch(method, data)
            }

            fn native_version() -> sc_executor::NativeVersion {
                $module::native_version()
            }
        }
    };
}

type EHF = (
    frame_benchmarking::benchmarking::HostFunctions,
    native_rng::HostFunctions,
);

native_executor_instance!(
    GeneralExecutor,
    polymesh_runtime_develop,
    (EHF, native_schnorrkel::HostFunctions)
);
native_executor_instance!(TestnetExecutor, polymesh_runtime_testnet, EHF);
native_executor_instance!(MainnetExecutor, polymesh_runtime_mainnet, EHF);

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_consensus_babe::BabeApi<Block>
    + grandpa::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
    + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + pallet_staking_rpc_runtime_api::StakingApi<Block>
    + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
    + node_rpc_runtime_api::identity::IdentityApi<Block, IdentityId, Ticker, AccountId, Moment>
    + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
    + node_rpc_runtime_api::asset::AssetApi<Block, AccountId>
    + pallet_group_rpc_runtime_api::GroupApi<Block>
    + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId>
    + node_rpc_runtime_api::nft::NFTApi<Block>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api> RuntimeApiCollection for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_consensus_babe::BabeApi<Block>
        + grandpa::GrandpaApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
        + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + pallet_staking_rpc_runtime_api::StakingApi<Block>
        + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
        + node_rpc_runtime_api::identity::IdentityApi<Block, IdentityId, Ticker, AccountId, Moment>
        + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
        + node_rpc_runtime_api::asset::AssetApi<Block, AccountId>
        + pallet_group_rpc_runtime_api::GroupApi<Block>
        + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId>
        + node_rpc_runtime_api::nft::NFTApi<Block>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

// Using prometheus, use a registry with a prefix of `polymesh`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
    if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
        *registry = Registry::new_custom(Some("polymesh".into()), None)?;
    }

    Ok(())
}

type BabeLink = sc_consensus_babe::BabeLink<Block>;

type FullLinkHalf<R, D> = grandpa::LinkHalf<Block, FullClient<R, D>, FullSelectChain>;
pub type FullClient<R, D> = sc_service::TFullClient<Block, R, NativeElseWasmExecutor<D>>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<R, D> =
    grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<R, D>, FullSelectChain>;
type FullBabeImportQueue<R, D> = sc_consensus::DefaultImportQueue<Block, FullClient<R, D>>;
type FullStateBackend = sc_client_api::StateBackendFor<FullBackend, Block>;
type FullPool<R, D> = sc_transaction_pool::FullPool<Block, FullClient<R, D>>;
pub type FullServiceComponents<R, D, F> = sc_service::PartialComponents<
    FullClient<R, D>,
    FullBackend,
    FullSelectChain,
    FullBabeImportQueue<R, D>,
    FullPool<R, D>,
    (
        F,
        (FullBabeBlockImport<R, D>, FullLinkHalf<R, D>, BabeLink),
        grandpa::SharedVoterState,
        Option<Telemetry>,
    ),
>;
type FullBabeBlockImport<R, D> =
    sc_consensus_babe::BabeBlockImport<Block, FullClient<R, D>, FullGrandpaBlockImport<R, D>>;

pub fn new_partial<R, D>(
    config: &mut Configuration,
) -> Result<
    FullServiceComponents<
        R,
        D,
        impl Fn(
            sc_rpc::DenyUnsafe,
            sc_rpc::SubscriptionTaskExecutor,
        ) -> Result<jsonrpsee::RpcModule<()>, Error>,
    >,
    Error,
>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
{
    set_prometheus_registry(config)?;

    let telemetry = config
        .telemetry_endpoints
        .clone()
        .filter(|x| !x.is_empty())
        .map(|endpoints| -> Result<_, sc_telemetry::Error> {
            let worker = TelemetryWorker::new(16)?;
            let telemetry = worker.handle().new_telemetry(endpoints);
            Ok((worker, telemetry))
        })
        .transpose()?;

    let executor = NativeElseWasmExecutor::<D>::new(
        config.wasm_method,
        config.default_heap_pages,
        config.max_runtime_instances,
        config.runtime_cache_size,
    );

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, R, NativeElseWasmExecutor<D>>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_essential_handle(),
        client.clone(),
    );

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::configuration(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let slot_duration = babe_link.config().slot_duration();
    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        block_import.clone(),
        Some(Box::new(justification_import)),
        client.clone(),
        select_chain.clone(),
        move |_, ()| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

            let slot =
                sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                    *timestamp,
                    slot_duration,
                );

            Ok((slot, timestamp))
        },
        &task_manager.spawn_essential_handle(),
        config.prometheus_registry(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;

    let import_setup = (block_import, grandpa_link, babe_link);

    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link, babe_link) = &import_setup;

        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = grandpa::SharedVoterState::empty();
        let rpc_setup = shared_voter_state.clone();

        let finality_proof_provider = grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let babe_config = babe_link.config().clone();
        let shared_epoch_changes = babe_link.epoch_changes().clone();

        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let keystore = keystore_container.sync_keystore();
        let chain_spec = config.chain_spec.cloned_box();

        let rpc_backend = backend.clone();
        let rpc_extensions_builder = move |deny_unsafe, subscription_executor| {
            let deps = node_rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                chain_spec: chain_spec.cloned_box(),
                deny_unsafe,
                babe: node_rpc::BabeDeps {
                    babe_config: babe_config.clone(),
                    shared_epoch_changes: shared_epoch_changes.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: node_rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                    justification_stream: justification_stream.clone(),
                    subscription_executor,
                    finality_provider: finality_proof_provider.clone(),
                },
            };

            node_rpc::create_full(deps, rpc_backend.clone()).map_err(Into::into)
        };

        (rpc_extensions_builder, rpc_setup)
    };

    Ok(sc_service::PartialComponents {
        client,
        backend,
        task_manager,
        keystore_container,
        select_chain,
        import_queue,
        transaction_pool,
        other: (rpc_extensions_builder, import_setup, rpc_setup, telemetry),
    })
}

pub struct NewFullBase<R, D>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
{
    /// The task manager of the node.
    pub task_manager: TaskManager,
    /// The client instance of the node.
    pub client: Arc<FullClient<R, D>>,
    /// The networking service of the node.
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    /// The transaction pool of the node.
    pub transaction_pool: Arc<FullPool<R, D>>,
    /// The rpc handlers of the node.
    pub rpc_handlers: RpcHandlers,
}

/// Creates a full service from the configuration.
pub fn new_full_base<R, D, F>(
    mut config: Configuration,
    with_startup_data: F,
) -> Result<NewFullBase<R, D>, ServiceError>
where
    F: FnOnce(&FullBabeBlockImport<R, D>, &BabeLink),
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        other: (rpc_builder, import_setup, rpc_setup, mut telemetry),
    } = new_partial(&mut config)?;

    let shared_voter_state = rpc_setup;
    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
    let grandpa_protocol_name = grandpa::protocol_standard_name(
        &client
            .block_hash(0)
            .ok()
            .flatten()
            .expect("Genesis block exists; qed"),
        &config.chain_spec,
    );

    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config(
            grandpa_protocol_name.clone(),
        ));
    let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        import_setup.1.shared_authority_set().clone(),
        Vec::default(),
    ));

    #[cfg(feature = "cli")]
    config.network.request_response_protocols.push(
        sc_consensus_grandpa_warp_sync::request_response_config_for_chain(
            &config,
            task_manager.spawn_handle(),
            backend.clone(),
        ),
    );

    let (network, system_rpc_tx, tx_handler_controller, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let role = config.role.clone();
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks =
        Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let prometheus_registry = config.prometheus_registry().cloned();

    let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend,
        client: client.clone(),
        keystore: keystore_container.sync_keystore(),
        network: network.clone(),
        rpc_builder: Box::new(rpc_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        system_rpc_tx,
        tx_handler_controller,
        telemetry: telemetry.as_mut(),
    })?;

    let (block_import, grandpa_link, babe_link) = import_setup;

    (with_startup_data)(&block_import, &babe_link);

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let client_clone = client.clone();
        let slot_duration = babe_link.config().slot_duration();
        let babe_config = sc_consensus_babe::BabeParams {
            keystore: keystore_container.sync_keystore(),
            client: client.clone(),
            select_chain,
            env: proposer,
            block_import,
            sync_oracle: network.clone(),
            justification_sync_link: network.clone(),
            create_inherent_data_providers: move |parent, ()| {
                let client_clone = client_clone.clone();
                async move {
                    let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                    let slot =
                        sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                            *timestamp,
                            slot_duration,
                        );

                    let storage_proof =
                        sp_transaction_storage_proof::registration::new_data_provider(
                            &*client_clone,
                            &parent,
                        )?;

                    Ok((slot, timestamp, storage_proof))
                }
            },
            force_authoring,
            backoff_authoring_blocks,
            babe_link,
            block_proposal_slot_portion: SlotProportion::new(0.5),
            max_block_proposal_slot_portion: None,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        };

        let babe = sc_consensus_babe::start_babe(babe_config)?;
        task_manager.spawn_essential_handle().spawn_blocking(
            "babe-proposer",
            Some("block-authoring"),
            babe,
        );
    }

    // Spawn authority discovery module.
    if role.is_authority() {
        let authority_discovery_role =
            sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
        let dht_event_stream =
            network
                .event_stream("authority-discovery")
                .filter_map(|e| async move {
                    match e {
                        Event::Dht(e) => Some(e),
                        _ => None,
                    }
                });
        let (authority_discovery_worker, _service) =
            sc_authority_discovery::new_worker_and_service_with_config(
                sc_authority_discovery::WorkerConfig {
                    publish_non_global_ips: auth_disc_publish_non_global_ips,
                    ..Default::default()
                },
                client.clone(),
                network.clone(),
                Box::pin(dht_event_stream),
                authority_discovery_role,
                prometheus_registry.clone(),
            );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
            Some("networking"),
            authority_discovery_worker.run(),
        );
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if role.is_authority() {
        Some(keystore_container.sync_keystore())
    } else {
        None
    };

    let config = grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: std::time::Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role,
        telemetry: telemetry.as_ref().map(|x| x.handle()),
        protocol_name: grandpa_protocol_name,
    };

    if enable_grandpa {
        // start the full GRANDPA voter
        // NOTE: non-authorities could run the GRANDPA observer protocol, but at
        // this point the full voter should provide better guarantees of block
        // and vote data availability than the observer. The observer has not
        // been tested extensively yet and having most nodes in a network run it
        // could lead to finality stalls.
        let grandpa_config = grandpa::GrandpaParams {
            config,
            link: grandpa_link,
            network: network.clone(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            voting_rule: grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state,
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            grandpa::run_grandpa_voter(grandpa_config)?,
        );
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        client,
        network,
        transaction_pool,
        rpc_handlers,
    })
}

type TaskResult = Result<TaskManager, ServiceError>;

/// Create a new Testnet service for a full node.
pub fn testnet_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

/// Create a new General node service for a full node.
pub fn general_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

/// Create a new Mainnet service for a full node.
pub fn mainnet_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

pub type NewChainOps<R, D> = (
    Arc<FullClient<R, D>>,
    Arc<FullBackend>,
    FullBabeImportQueue<R, D>,
    TaskManager,
);

/// Builds a new object suitable for chain operations.
pub fn chain_ops<R, D>(config: &mut Configuration) -> Result<NewChainOps<R, D>, ServiceError>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
{
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let FullServiceComponents {
        client,
        backend,
        import_queue,
        task_manager,
        ..
    } = new_partial::<R, D>(config)?;
    Ok((client, backend, import_queue, task_manager))
}

pub fn testnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor>, ServiceError> {
    chain_ops::<_, _>(config)
}

pub fn general_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_develop::RuntimeApi, GeneralExecutor>, ServiceError> {
    chain_ops::<_, _>(config)
}

pub fn mainnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor>, ServiceError> {
    chain_ops::<_, _>(config)
}
