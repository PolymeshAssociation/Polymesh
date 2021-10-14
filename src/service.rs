//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

// pub use crate::chain_spec::{
//     testnet::ChainSpec as TestnetChainSpec,
// };
pub use codec::Codec;
use core::marker::PhantomData;
use futures::stream::StreamExt;
use polymesh_node_rpc as node_rpc;
pub use polymesh_primitives::{
    crypto::native_schnorrkel, host_functions::native_rng::native_rng, AccountId, Balance, Block,
    BlockNumber, Hash, IdentityId, Index as Nonce, Moment, SecondaryKey, Signatory, Ticker,
};
pub use polymesh_runtime_ci;
pub use polymesh_runtime_develop;
pub use polymesh_runtime_mainnet;
pub use polymesh_runtime_testnet;
use prometheus_endpoint::Registry;
use sc_client_api::ExecutorProvider;
pub use sc_client_api::{backend::Backend, RemoteBackend};
pub use sc_consensus::LongestChain;
use sc_executor::native_executor_instance;
pub use sc_executor::{NativeExecutionDispatch, NativeExecutor};
use sc_network::{Event, NetworkService};
pub use sc_service::{
    config::{DatabaseConfig, PrometheusConfig, Role},
    error::Error as ServiceError,
    ChainSpec, Configuration, Error, PruningMode, RuntimeGenesis, TFullBackend, TFullCallExecutor,
    TFullClient, TLightBackend, TLightCallExecutor, TLightClient, TransactionPoolOptions,
};
use sc_service::{RpcHandlers, TaskManager};
use sc_telemetry::TelemetryConnectionNotifier;
pub use sp_api::{ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi, StateBackend};
pub use sp_consensus::SelectChain;
use sp_inherents::InherentDataProviders;
pub use sp_runtime::traits::BlakeTwo256;
use sp_runtime::traits::Block as BlockT;
use std::sync::Arc;

/// Known networks based on name.
pub enum Network {
    Mainnet,
    CI,
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
        } else if name.starts_with("Polymesh CI") {
            Network::CI
        } else {
            Network::Other
        }
    }
}

// Our native executor instance.
native_executor_instance!(
    pub TestnetExecutor,
    polymesh_runtime_testnet::api::dispatch,
    polymesh_runtime_testnet::native_version,
    (frame_benchmarking::benchmarking::HostFunctions, native_rng::HostFunctions)
);

// Our native executor instance.
native_executor_instance!(
    pub GeneralExecutor,
    polymesh_runtime_develop::api::dispatch,
    polymesh_runtime_develop::native_version,
    (frame_benchmarking::benchmarking::HostFunctions, native_rng::HostFunctions, native_schnorrkel::HostFunctions)
);

native_executor_instance!(
    pub CIExecutor,
    polymesh_runtime_ci::api::dispatch,
    polymesh_runtime_ci::native_version,
    (frame_benchmarking::benchmarking::HostFunctions, native_rng::HostFunctions)
);

native_executor_instance!(
    pub MainnetExecutor,
    polymesh_runtime_mainnet::api::dispatch,
    polymesh_runtime_mainnet::native_version,
    (frame_benchmarking::benchmarking::HostFunctions, native_rng::HostFunctions)
);

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: RuntimeExtrinsic>:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_api::ApiErrorExt<Error = sp_blockchain::Error>
    + sp_consensus_babe::BabeApi<Block>
    + grandpa::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
    + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block, Extrinsic>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    //+ pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
    + pallet_staking_rpc_runtime_api::StakingApi<Block>
    + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
    + node_rpc_runtime_api::identity::IdentityApi<
        Block,
        IdentityId,
        Ticker,
        AccountId,
        SecondaryKey<AccountId>,
        Signatory<AccountId>,
        Moment,
    > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
    + node_rpc_runtime_api::asset::AssetApi<Block, AccountId>
    + pallet_group_rpc_runtime_api::GroupApi<Block>
    + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId>
where
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api, Extrinsic: RuntimeExtrinsic> RuntimeApiCollection<Extrinsic> for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_api::ApiErrorExt<Error = sp_blockchain::Error>
        + sp_consensus_babe::BabeApi<Block>
        + grandpa::GrandpaApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
        + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block, Extrinsic>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        //+ pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
        + pallet_staking_rpc_runtime_api::StakingApi<Block>
        + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
        + node_rpc_runtime_api::identity::IdentityApi<
            Block,
            IdentityId,
            Ticker,
            AccountId,
            SecondaryKey<AccountId>,
            Signatory<AccountId>,
            Moment,
        > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
        + node_rpc_runtime_api::asset::AssetApi<Block, AccountId>
        + pallet_group_rpc_runtime_api::GroupApi<Block>
        + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId>,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

pub trait RuntimeExtrinsic: codec::Codec + Send + Sync + 'static {}

impl<E> RuntimeExtrinsic for E where E: codec::Codec + Send + Sync + 'static {}

// Using prometheus, use a registry with a prefix of `polymesh`.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
    if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
        *registry = Registry::new_custom(Some("polymesh".into()), None)?;
    }

    Ok(())
}

type BabeLink = sc_consensus_babe::BabeLink<Block>;
type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

type FullLinkHalf<R, D> = grandpa::LinkHalf<Block, FullClient<R, D>, FullSelectChain>;
type FullClient<R, E> = sc_service::TFullClient<Block, R, E>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<R, E> =
    grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<R, E>, FullSelectChain>;
type FullBabeImportQueue<R, E> = sp_consensus::DefaultImportQueue<Block, FullClient<R, E>>;
type FullStateBackend = sc_client_api::StateBackendFor<FullBackend, Block>;
type FullPool<R, E> = sc_transaction_pool::FullPool<Block, FullClient<R, E>>;
type FullServiceComponents<R, E, F> = sc_service::PartialComponents<
    FullClient<R, E>,
    FullBackend,
    FullSelectChain,
    FullBabeImportQueue<R, E>,
    FullPool<R, E>,
    (
        F,
        (FullBabeBlockImport<R, E>, FullLinkHalf<R, E>, BabeLink),
        grandpa::SharedVoterState,
    ),
>;
type FullBabeBlockImport<R, E> =
    sc_consensus_babe::BabeBlockImport<Block, FullClient<R, E>, FullGrandpaBlockImport<R, E>>;

pub fn new_partial<R, D, E>(
    config: &mut Configuration,
) -> Result<
    FullServiceComponents<
        R,
        D,
        impl Fn(sc_rpc::DenyUnsafe, sc_rpc::SubscriptionTaskExecutor) -> IoHandler,
    >,
    Error,
>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    set_prometheus_registry(config)?;

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, R, D>(&config)?;
    let client = Arc::new(client);

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        config.role.is_authority().into(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        client.clone(),
    );

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::Config::get_or_compute(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let inherent_data_providers = sp_inherents::InherentDataProviders::new();

    let import_queue = sc_consensus_babe::import_queue(
        babe_link.clone(),
        block_import.clone(),
        Some(Box::new(justification_import)),
        client.clone(),
        select_chain.clone(),
        inherent_data_providers.clone(),
        &task_manager.spawn_handle(),
        config.prometheus_registry(),
        sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone()),
    )?;

    let import_setup = (block_import, grandpa_link, babe_link);

    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link, babe_link) = &import_setup;

        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = grandpa::SharedVoterState::empty();
        // let finality_proof_provider =
        //     GrandpaFinalityProofProvider::new_for_service(backend.clone(), client.clone());
        // let rpc_setup = (shared_voter_state.clone(), finality_proof_provider.clone());
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

            node_rpc::create_full(deps)
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
        inherent_data_providers,
        other: (rpc_extensions_builder, import_setup, rpc_setup),
    })
}

pub struct NewFullBase<R, D, E>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    pub task_manager: TaskManager,
    pub inherent_data_providers: InherentDataProviders,
    pub client: Arc<FullClient<R, D>>,
    pub network: Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
    pub network_status_sinks: sc_service::NetworkStatusSinks<Block>,
    pub transaction_pool: Arc<FullPool<R, D>>,
    marker: PhantomData<E>,
}

/// Creates a full service from the configuration.
pub fn new_full_base<R, D, E, F>(
    mut config: Configuration,
    with_startup_data: F,
) -> Result<NewFullBase<R, D, E>, ServiceError>
where
    F: FnOnce(&FullBabeBlockImport<R, D>, &BabeLink),
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    let sc_service::PartialComponents {
        client,
        backend,
        mut task_manager,
        import_queue,
        keystore_container,
        select_chain,
        transaction_pool,
        inherent_data_providers,
        other: (rpc_extensions_builder, import_setup, rpc_setup),
    } = new_partial(&mut config)?;

    let shared_voter_state = rpc_setup;

    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config());

    #[cfg(feature = "cli")]
    config.network.request_response_protocols.push(
        sc_finality_grandpa_warp_sync::request_response_config_for_chain(
            &config,
            task_manager.spawn_handle(),
            backend.clone(),
        ),
    );

    let (network, network_status_sinks, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: None,
            block_announce_validator_builder: None,
        })?;

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            backend.clone(),
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

    let (_rpc_handlers, telemetry_connection_notifier) =
        sc_service::spawn_tasks(sc_service::SpawnTasksParams {
            config,
            backend: backend.clone(),
            client: client.clone(),
            keystore: keystore_container.sync_keystore(),
            network: network.clone(),
            rpc_extensions_builder: Box::new(rpc_extensions_builder),
            transaction_pool: transaction_pool.clone(),
            task_manager: &mut task_manager,
            on_demand: None,
            remote_blockchain: None,
            network_status_sinks: network_status_sinks.clone(),
            system_rpc_tx,
        })?;

    let (block_import, grandpa_link, babe_link) = import_setup;

    (with_startup_data)(&block_import, &babe_link);

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
        );

        let can_author_with =
            sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

        let babe_config = sc_consensus_babe::BabeParams {
            keystore: keystore_container.sync_keystore(),
            client: client.clone(),
            select_chain,
            env: proposer,
            block_import,
            sync_oracle: network.clone(),
            inherent_data_providers: inherent_data_providers.clone(),
            force_authoring,
            backoff_authoring_blocks,
            babe_link,
            can_author_with,
        };

        let babe = sc_consensus_babe::start_babe(babe_config)?;
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("babe-proposer", babe);
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
        let (authority_discovery_worker, _service) = sc_authority_discovery::new_worker_and_service(
            client.clone(),
            network.clone(),
            Box::pin(dht_event_stream),
            authority_discovery_role,
            prometheus_registry.clone(),
        );

        task_manager.spawn_handle().spawn(
            "authority-discovery-worker",
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
        is_authority: role.is_authority(),
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
            telemetry_on_connect: telemetry_connection_notifier.map(|x| x.on_connect_stream()),
            voting_rule: grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry,
            shared_voter_state,
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("grandpa-voter", grandpa::run_grandpa_voter(grandpa_config)?);
    }

    network_starter.start_network();
    Ok(NewFullBase {
        task_manager,
        inherent_data_providers,
        client,
        network,
        network_status_sinks,
        transaction_pool,
        marker: PhantomData,
    })
}

type TaskResult = Result<TaskManager, ServiceError>;

/// Create a new Testnet service for a full node.
pub fn testnet_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor, _, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

/// Create a new General node service for a full node.
pub fn general_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

/// Create a new CI service for a full node.
pub fn ci_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_ci::RuntimeApi, CIExecutor, _, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

/// Create a new Mainnet service for a full node.
pub fn mainnet_new_full(config: Configuration) -> TaskResult {
    new_full_base::<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor, _, _>(config, |_, _| ())
        .map(|data| data.task_manager)
}

pub type NewChainOps<R, D> = (
    Arc<FullClient<R, D>>,
    Arc<FullBackend>,
    FullBabeImportQueue<R, D>,
    TaskManager,
);

/// Builds a new object suitable for chain operations.
pub fn chain_ops<R, D, E>(config: &mut Configuration) -> Result<NewChainOps<R, D>, ServiceError>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let FullServiceComponents {
        client,
        backend,
        import_queue,
        task_manager,
        ..
    } = new_partial::<R, D, E>(config)?;
    Ok((client, backend, import_queue, task_manager))
}

pub fn testnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor>, ServiceError> {
    chain_ops::<_, _, polymesh_runtime_testnet::UncheckedExtrinsic>(config)
}

pub fn general_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_develop::RuntimeApi, GeneralExecutor>, ServiceError> {
    chain_ops::<_, _, polymesh_runtime_develop::UncheckedExtrinsic>(config)
}

pub fn ci_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_ci::RuntimeApi, CIExecutor>, ServiceError> {
    chain_ops::<_, _, polymesh_runtime_ci::UncheckedExtrinsic>(config)
}

pub fn mainnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor>, ServiceError> {
    chain_ops::<_, _, polymesh_runtime_mainnet::UncheckedExtrinsic>(config)
}

type LightStorage = sc_client_db::light::LightStorage<Block>;
type LightBackend = sc_light::backend::Backend<LightStorage, BlakeTwo256>;
type LightClient<R, E> = sc_service::TLightClient<Block, R, E>;
type LightStateBackend = sc_client_api::StateBackendFor<LightBackend, Block>;
type LightPool<R, E> =
    sc_transaction_pool::LightPool<Block, LightClient<R, E>, sc_network::config::OnDemand<Block>>;

pub fn new_light_base<R, D, E>(
    mut config: Configuration,
) -> Result<
    (
        TaskManager,
        RpcHandlers,
        Option<TelemetryConnectionNotifier>,
        Arc<LightClient<R, D>>,
        Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
        Arc<LightPool<R, D>>,
    ),
    ServiceError,
>
where
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = LightStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
    R: Send
        + Sync
        + 'static
        + sp_api::ConstructRuntimeApi<
            Block,
            sc_service::TLightClientWithBackend<Block, R, D, LightBackend>,
        >,
{
    let (client, backend, keystore_container, mut task_manager, on_demand) =
        sc_service::new_light_parts::<Block, R, D>(&config)?;

    config
        .network
        .extra_sets
        .push(grandpa::grandpa_peers_set_config());

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
        config.transaction_pool.clone(),
        config.prometheus_registry(),
        task_manager.spawn_handle(),
        client.clone(),
        on_demand.clone(),
    ));

    let (grandpa_block_import, _) = grandpa::block_import(
        client.clone(),
        &(client.clone() as Arc<_>),
        select_chain.clone(),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
        sc_consensus_babe::Config::get_or_compute(&*client)?,
        grandpa_block_import,
        client.clone(),
    )?;

    let inherent_data_providers = sp_inherents::InherentDataProviders::new();

    let import_queue = sc_consensus_babe::import_queue(
        babe_link,
        babe_block_import,
        Some(Box::new(justification_import)),
        client.clone(),
        select_chain.clone(),
        inherent_data_providers.clone(),
        &task_manager.spawn_handle(),
        config.prometheus_registry(),
        sp_consensus::NeverCanAuthor,
    )?;

    let (network, network_status_sinks, system_rpc_tx, network_starter) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            import_queue,
            on_demand: Some(on_demand.clone()),
            block_announce_validator_builder: None,
        })?;
    network_starter.start_network();

    if config.offchain_worker.enabled {
        sc_service::build_offchain_workers(
            &config,
            backend.clone(),
            task_manager.spawn_handle(),
            client.clone(),
            network.clone(),
        );
    }

    let light_deps = node_rpc::LightDeps {
        remote_blockchain: backend.remote_blockchain(),
        fetcher: on_demand.clone(),
        client: client.clone(),
        pool: transaction_pool.clone(),
    };

    let rpc_extensions = node_rpc::create_light(light_deps);

    let (rpc_handlers, telemetry_connection_notifier) =
        sc_service::spawn_tasks(sc_service::SpawnTasksParams {
            on_demand: Some(on_demand),
            remote_blockchain: Some(backend.remote_blockchain()),
            rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            keystore: keystore_container.sync_keystore(),
            config,
            backend,
            network_status_sinks,
            system_rpc_tx,
            network: network.clone(),
            task_manager: &mut task_manager,
        })?;

    Ok((
        task_manager,
        rpc_handlers,
        telemetry_connection_notifier,
        client,
        network,
        transaction_pool,
    ))
}

/// Create a new Polymesh services for light clients.
pub fn testnet_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor, _>(config)
        .map(|(task_manager, _, _, _, _, _)| task_manager)
}

pub fn ci_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_ci::RuntimeApi, CIExecutor, _>(config)
        .map(|(task_manager, _, _, _, _, _)| task_manager)
}

pub fn general_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _>(config)
        .map(|(task_manager, ..)| task_manager)
}

pub fn mainnet_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor, _>(config)
        .map(|(task_manager, ..)| task_manager)
}
