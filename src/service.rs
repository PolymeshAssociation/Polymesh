//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use polymesh_node_rpc as node_rpc;
use polymesh_primitives::{
    AccountId, Balance, Block, BlockNumber, IdentityId, Moment, Nonce, Ticker,
};
use polymesh_runtime_develop;
use polymesh_runtime_mainnet;
use polymesh_runtime_testnet;

use crate::Cli;
use codec::Encode;
use frame_benchmarking_cli::SUBSTRATE_REFERENCE_HARDWARE;
use frame_system_rpc_runtime_api::AccountNonceApi;
use futures::prelude::*;
use prometheus_endpoint::Registry;
use sc_client_api::{Backend, BlockBackend};
use sc_consensus_babe::{self, SlotProportion};
use sc_consensus_beefy as beefy;
use sc_consensus_grandpa as grandpa;
use sc_network::{
    event::Event, service::traits::NetworkService, NetworkBackend, NetworkEventStream,
};
use sc_network_sync::{strategy::warp::WarpSyncConfig, SyncingService};
use sc_service::ChainSpec;
use sc_service::{
    config::{Configuration, PrometheusConfig},
    error::Error as ServiceError,
    RpcHandlers, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sc_transaction_pool::TransactionPoolHandle;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
use sp_api::{ConstructRuntimeApi, ProvideRuntimeApi};
use sp_consensus_babe::inherents::BabeCreateInherentDataProviders;
use sp_consensus_beefy as beefy_primitives;
use sp_core::crypto::Pair;
use sp_runtime::{generic, traits::Block as BlockT, SaturatedConversion};
use std::{path::Path, sync::Arc};

/// Known networks based on name.
pub enum Network {
    /// Mainnet network.
    Mainnet,
    /// Testnet network.
    Testnet,
    /// Develop network.
    Other,
}

/// Trait to identify the network from the chain spec.
pub trait IsNetwork {
    /// Returns the network type.
    fn network(&self) -> Network;
}

impl IsNetwork for dyn ChainSpec {
    fn network(&self) -> Network {
        if self.name().starts_with("Polymesh Mainnet") {
            return Network::Mainnet;
        }

        if self.name().starts_with("Polymesh Testnet") {
            return Network::Testnet;
        }

        Network::Other
    }
}

/// The Beefy Authority Id type.
pub type BeefyId = beefy_primitives::ecdsa_crypto::AuthorityId;

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block>
    + sp_consensus_babe::BabeApi<Block>
    + grandpa::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
    + mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>
    + sp_consensus_beefy::BeefyApi<Block, BeefyId>
    + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + pallet_staking_runtime_api::StakingApi<Block, Balance, AccountId>
    + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
    + node_rpc_runtime_api::identity::IdentityApi<Block, IdentityId, Ticker, AccountId, Moment>
    + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
    + node_rpc_runtime_api::asset::AssetApi<Block>
    + pallet_group_rpc_runtime_api::GroupApi<Block>
    + node_rpc_runtime_api::nft::NFTApi<Block>
    + node_rpc_runtime_api::settlement::SettlementApi<Block>
{
}

impl<Api> RuntimeApiCollection for Api where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block>
        + sp_consensus_babe::BabeApi<Block>
        + grandpa::GrandpaApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
        + mmr_rpc::MmrRuntimeApi<Block, <Block as sp_runtime::traits::Block>::Hash, BlockNumber>
        + sp_consensus_beefy::BeefyApi<Block, BeefyId>
        + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + pallet_staking_runtime_api::StakingApi<Block, Balance, AccountId>
        + node_rpc_runtime_api::pips::PipsApi<Block, AccountId>
        + node_rpc_runtime_api::identity::IdentityApi<Block, IdentityId, Ticker, AccountId, Moment>
        + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
        + node_rpc_runtime_api::asset::AssetApi<Block>
        + pallet_group_rpc_runtime_api::GroupApi<Block>
        + node_rpc_runtime_api::nft::NFTApi<Block>
        + node_rpc_runtime_api::settlement::SettlementApi<Block>
{
}

/// Host functions available to the runtime.
#[cfg(not(feature = "runtime-benchmarks"))]
pub type HostFunctions = (
    sp_io::SubstrateHostFunctions,
    polymesh_worker_extension::native_polymesh_worker::HostFunctions,
    polymesh_native_crypto::HostFunctions,
);

/// Host functions available to the runtime.
#[cfg(feature = "runtime-benchmarks")]
pub type HostFunctions = (
    sp_io::SubstrateHostFunctions,
    frame_benchmarking::benchmarking::HostFunctions,
    polymesh_primitives::crypto::native_schnorrkel::HostFunctions,
    polymesh_worker_extension::native_polymesh_worker::HostFunctions,
    polymesh_native_crypto::HostFunctions,
);

/// A specialized `WasmExecutor` intended to use across substrate node. It provides all required HostFunctions.
pub type RuntimeExecutor = sc_executor::WasmExecutor<HostFunctions>;

/// The Full client type definition
pub type FullClient<R> = sc_service::TFullClient<Block, R, RuntimeExecutor>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<R> =
    grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<R>, FullSelectChain>;
type FullBeefyBlockImport<InnerBlockImport, R> = beefy::import::BeefyBlockImport<
    Block,
    FullBackend,
    FullClient<R>,
    InnerBlockImport,
    beefy_primitives::ecdsa_crypto::AuthorityId,
>;

/// The transaction pool type definition.
pub type TransactionPool<R> = sc_transaction_pool::TransactionPoolHandle<Block, FullClient<R>>;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

/// Fetch the nonce of the given `account` from the chain state.
///
/// Note: Should only be used for tests.
pub fn fetch_nonce(
    client: &FullClient<polymesh_runtime_develop::RuntimeApi>,
    account: sp_core::sr25519::Pair,
) -> u32 {
    let best_hash = client.chain_info().best_hash;
    client
        .runtime_api()
        .account_nonce(best_hash, account.public().into())
        .expect("Fetching account nonce works; qed")
}

/// Create a transaction using the given `call`.
///
/// The transaction will be signed by `sender`. If `nonce` is `None` it will be fetched from the
/// state of the best block.
///
/// Note: Should only be used for tests.
pub fn create_extrinsic(
    client: &FullClient<polymesh_runtime_develop::RuntimeApi>,
    sender: sp_core::sr25519::Pair,
    function: impl Into<polymesh_runtime_develop::RuntimeCall>,
    nonce: Option<u32>,
) -> polymesh_runtime_develop::UncheckedExtrinsic {
    let function = function.into();
    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let best_hash = client.chain_info().best_hash;
    let best_block = client.chain_info().best_number;
    let nonce = nonce.unwrap_or_else(|| fetch_nonce(client, sender.clone()));

    let period = polymesh_runtime_common::BlockHashCount::get()
        .checked_next_power_of_two()
        .map(|c| c / 2)
        .unwrap_or(2) as u64;
    let tx_ext: polymesh_runtime_develop::TxExtension = (
        (
            frame_system::AuthorizeCall::new(),
            frame_system::CheckNonZeroSender::new(),
            frame_system::CheckSpecVersion::new(),
            frame_system::CheckTxVersion::new(),
            frame_system::CheckGenesis::new(),
        ),
        frame_system::CheckEra::from(generic::Era::mortal(period, best_block.saturated_into())),
        frame_system::CheckNonce::from(nonce),
        frame_system::CheckWeight::new(),
        polymesh_transaction_payment::ChargeTransactionPayment::from(0),
        pallet_permissions::StoreCallMetadata::new(),
        frame_metadata_hash_extension::CheckMetadataHash::new(false),
        pallet_revive::evm::tx_extension::SetOrigin::default(),
        frame_system::WeightReclaim::new(),
    );

    let raw_payload = polymesh_runtime_develop::runtime::SignedPayload::from_raw(
        function.clone(),
        tx_ext.clone(),
        (
            (
                (),
                (),
                polymesh_runtime_develop::runtime::VERSION.spec_version,
                polymesh_runtime_develop::runtime::VERSION.transaction_version,
                genesis_hash,
            ),
            best_hash,
            (),
            (),
            (),
            (),
            None,
            (),
            (),
        ),
    );
    let signature = raw_payload.using_encoded(|e| sender.sign(e));

    generic::UncheckedExtrinsic::new_signed(
        function,
        sp_runtime::AccountId32::from(sender.public()).into(),
        polymesh_primitives::Signature::Sr25519(signature),
        tx_ext,
    )
    .into()
}

/// Sets the registry with a `polymesh` prefix.
fn set_prometheus_registry(config: &mut Configuration) -> Result<(), ServiceError> {
    if let Some(PrometheusConfig { registry, .. }) = config.prometheus_config.as_mut() {
        *registry = Registry::new_custom(Some("polymesh".into()), None)?;
    }

    Ok(())
}

/// Creates a new partial node.
pub fn new_partial<R>(
    config: &mut Configuration,
) -> Result<
    sc_service::PartialComponents<
        FullClient<R>,
        FullBackend,
        FullSelectChain,
        sc_consensus::DefaultImportQueue<Block>,
        sc_transaction_pool::TransactionPoolHandle<Block, FullClient<R>>,
        (
            impl Fn(sc_rpc::SubscriptionTaskExecutor) -> Result<jsonrpsee::RpcModule<()>, ServiceError>,
            (
                sc_consensus_babe::BabeBlockImport<
                    Block,
                    FullClient<R>,
                    FullBeefyBlockImport<FullGrandpaBlockImport<R>, R>,
                    BabeCreateInherentDataProviders<Block>,
                    FullSelectChain,
                >,
                grandpa::LinkHalf<Block, FullClient<R>, FullSelectChain>,
                sc_consensus_babe::BabeLink<Block>,
                beefy::BeefyVoterLinks<Block, beefy_primitives::ecdsa_crypto::AuthorityId>,
            ),
            grandpa::SharedVoterState,
            Option<Telemetry>,
        ),
    >,
    ServiceError,
>
where
    R: ConstructRuntimeApi<Block, FullClient<R>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection,
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

    let executor = sc_service::new_wasm_executor(&config.executor);

    let (client, backend, keystore_container, task_manager) =
        sc_service::new_full_parts::<Block, R, _>(
            config,
            telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
            executor,
            vec![Arc::new(grandpa::GrandpaPruningFilter)],
        )?;
    let client = Arc::new(client);

    let telemetry = telemetry.map(|(worker, telemetry)| {
        task_manager
            .spawn_handle()
            .spawn("telemetry", None, worker.run());
        telemetry
    });

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let transaction_pool = Arc::from(
        sc_transaction_pool::Builder::new(
            task_manager.spawn_essential_handle(),
            client.clone(),
            config.role.is_authority().into(),
        )
        .with_options(config.transaction_pool.clone())
        .with_prometheus(config.prometheus_registry())
        .build(),
    );

    let (grandpa_block_import, grandpa_link) = grandpa::block_import(
        client.clone(),
        GRANDPA_JUSTIFICATION_PERIOD,
        &(client.clone() as Arc<_>),
        select_chain.clone(),
        telemetry.as_ref().map(|x| x.handle()),
    )?;
    let justification_import = grandpa_block_import.clone();

    let (beefy_block_import, beefy_voter_links, beefy_rpc_links) =
        beefy::beefy_block_import_and_links(
            grandpa_block_import,
            backend.clone(),
            client.clone(),
            config.prometheus_registry().cloned(),
        );

    let babe_config = sc_consensus_babe::configuration(&*client)?;
    let slot_duration = babe_config.slot_duration();
    let (block_import, babe_link) = sc_consensus_babe::block_import(
        babe_config,
        beefy_block_import,
        client.clone(),
        Arc::new(move |_, _| async move {
            let timestamp = sp_timestamp::InherentDataProvider::from_system_time();
            let slot =
            sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                *timestamp,
                slot_duration,
            );
            Ok((slot, timestamp))
        }) as BabeCreateInherentDataProviders<Block>,
        select_chain.clone(),
        OffchainTransactionPoolFactory::new(transaction_pool.clone()),
    )?;

    let (import_queue, babe_worker_handle) =
        sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
            link: babe_link.clone(),
            block_import: block_import.clone(),
            justification_import: Some(Box::new(justification_import)),
            client: client.clone(),
            slot_duration,
            spawner: &task_manager.spawn_essential_handle(),
            registry: config.prometheus_registry(),
            telemetry: telemetry.as_ref().map(|x| x.handle()),
        })?;

    let import_setup = (block_import, grandpa_link, babe_link, beefy_voter_links);

    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link, _, _) = &import_setup;

        let justification_stream = grandpa_link.justification_stream();
        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = grandpa::SharedVoterState::empty();
        let shared_voter_state2 = shared_voter_state.clone();

        let finality_proof_provider = grandpa::FinalityProofProvider::new_for_service(
            backend.clone(),
            Some(shared_authority_set.clone()),
        );

        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let keystore = keystore_container.keystore();
        let chain_spec = config.chain_spec.cloned_box();

        let rpc_backend = backend.clone();
        let rpc_extensions_builder =
            move |subscription_executor: node_rpc::SubscriptionTaskExecutor| {
                let deps = node_rpc::FullDeps {
                    client: client.clone(),
                    pool: pool.clone(),
                    select_chain: select_chain.clone(),
                    chain_spec: chain_spec.cloned_box(),
                    babe: node_rpc::BabeDeps {
                        keystore: keystore.clone(),
                        babe_worker_handle: babe_worker_handle.clone(),
                    },
                    grandpa: node_rpc::GrandpaDeps {
                        shared_voter_state: shared_voter_state.clone(),
                        shared_authority_set: shared_authority_set.clone(),
                        justification_stream: justification_stream.clone(),
                        subscription_executor: subscription_executor.clone(),
                        finality_provider: finality_proof_provider.clone(),
                    },
                    beefy: node_rpc::BeefyDeps::<beefy_primitives::ecdsa_crypto::AuthorityId> {
                        beefy_finality_proof_stream: beefy_rpc_links
                            .from_voter_justif_stream
                            .clone(),
                        beefy_best_block_stream: beefy_rpc_links
                            .from_voter_best_beefy_stream
                            .clone(),
                        subscription_executor,
                    },
                    backend: rpc_backend.clone(),
                };

                node_rpc::create_full(deps).map_err(Into::into)
            };

        (rpc_extensions_builder, shared_voter_state2)
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

/// A structure that exposes the full Polymesh node service.
pub struct NewFullBase<R>
where
    R: ConstructRuntimeApi<Block, FullClient<R>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection,
{
    /// The task manager of the node.
    pub task_manager: TaskManager,
    /// The client instance of the node.
    pub client: Arc<FullClient<R>>,
    /// The networking service of the node.
    pub network: Arc<dyn NetworkService>,
    /// The syncing service of the node.
    pub sync: Arc<SyncingService<Block>>,
    /// The transaction pool of the node.
    pub transaction_pool: Arc<TransactionPoolHandle<Block, FullClient<R>>>,
    /// The rpc handlers of the node.
    pub rpc_handlers: RpcHandlers,
}

/// Creates a full service from the configuration.
pub fn new_full_base<N, R>(
    mut config: Configuration,
    enable_beefy: bool,
    disable_hardware_benchmarks: bool,
    with_startup_data: impl FnOnce(
        &sc_consensus_babe::BabeBlockImport<
            Block,
            FullClient<R>,
            FullBeefyBlockImport<FullGrandpaBlockImport<R>, R>,
            BabeCreateInherentDataProviders<Block>,
            FullSelectChain,
        >,
        &sc_consensus_babe::BabeLink<Block>,
    ),
) -> Result<NewFullBase<R>, ServiceError>
where
    N: NetworkBackend<Block, <Block as BlockT>::Hash>,
    R: ConstructRuntimeApi<Block, FullClient<R>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection,
{
    let is_offchain_indexing_enabled = config.offchain_worker.indexing_enabled;
    let role = config.role;
    let force_authoring = config.force_authoring;
    let backoff_authoring_blocks =
        Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
    let name = config.network.node_name.clone();
    let enable_grandpa = !config.disable_grandpa;
    let enable_offchain_worker = config.offchain_worker.enabled;

    let hwbench = (!disable_hardware_benchmarks)
        .then(|| {
            config.database.path().map(|database_path| {
                let _ = std::fs::create_dir_all(&database_path);
                sc_sysinfo::gather_hwbench(Some(database_path), &SUBSTRATE_REFERENCE_HARDWARE)
            })
        })
        .flatten();

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

    let prometheus_registry = config.prometheus_registry().cloned();
    let metrics = N::register_notification_metrics(
        config.prometheus_config.as_ref().map(|cfg| &cfg.registry),
    );
    let shared_voter_state = rpc_setup;
    let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;
    let auth_disc_public_addresses = config.network.public_addresses.clone();

    let mut net_config = sc_network::config::FullNetworkConfiguration::<_, _, N>::new(
        &config.network,
        config
            .prometheus_config
            .as_ref()
            .map(|cfg| cfg.registry.clone()),
    );

    let genesis_hash = client
        .block_hash(0)
        .ok()
        .flatten()
        .expect("Genesis block exists; qed");
    let peer_store_handle = net_config.peer_store_handle();

    let grandpa_protocol_name = grandpa::protocol_standard_name(&genesis_hash, &config.chain_spec);
    let (grandpa_protocol_config, grandpa_notification_service) =
        grandpa::grandpa_peers_set_config::<_, N>(
            grandpa_protocol_name.clone(),
            metrics.clone(),
            Arc::clone(&peer_store_handle),
        );
    net_config.add_notification_protocol(grandpa_protocol_config);

    let beefy_gossip_proto_name =
        beefy::gossip_protocol_name(&genesis_hash, config.chain_spec.fork_id());
    // `beefy_on_demand_justifications_handler` is given to `beefy-gadget` task to be run,
    // while `beefy_req_resp_cfg` is added to `config.network.request_response_protocols`.
    let (beefy_on_demand_justifications_handler, beefy_req_resp_cfg) =
        beefy::communication::request_response::BeefyJustifsRequestHandler::new::<_, N>(
            &genesis_hash,
            config.chain_spec.fork_id(),
            client.clone(),
            prometheus_registry.clone(),
        );

    let (beefy_notification_config, beefy_notification_service) =
        beefy::communication::beefy_peers_set_config::<_, N>(
            beefy_gossip_proto_name.clone(),
            metrics.clone(),
            Arc::clone(&peer_store_handle),
        );

    if enable_beefy {
        net_config.add_notification_protocol(beefy_notification_config);
        net_config.add_request_response_protocol(beefy_req_resp_cfg);
    }

    let warp_sync = Arc::new(grandpa::warp_proof::NetworkProvider::new(
        backend.clone(),
        import_setup.1.shared_authority_set().clone(),
        Vec::default(),
    ));

    let (network, system_rpc_tx, tx_handler_controller, sync_service) =
        sc_service::build_network(sc_service::BuildNetworkParams {
            config: &config,
            net_config,
            client: client.clone(),
            transaction_pool: transaction_pool.clone(),
            spawn_handle: task_manager.spawn_handle(),
            spawn_essential_handle: task_manager.spawn_essential_handle(),
            import_queue,
            block_announce_validator_builder: None,
            warp_sync_config: Some(WarpSyncConfig::WithProvider(warp_sync)),
            block_relay: None,
            metrics,
        })?;

    let net_config_path = config.network.net_config_path.clone();
    let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
        config,
        backend: backend.clone(),
        client: client.clone(),
        keystore: keystore_container.keystore(),
        network: network.clone(),
        rpc_builder: Box::new(rpc_builder),
        transaction_pool: transaction_pool.clone(),
        task_manager: &mut task_manager,
        system_rpc_tx,
        tx_handler_controller,
        sync_service: sync_service.clone(),
        telemetry: telemetry.as_mut(),
        tracing_execute_block: None,
    })?;

    if let Some(hwbench) = hwbench {
        sc_sysinfo::print_hwbench(&hwbench);
        match SUBSTRATE_REFERENCE_HARDWARE.check_hardware(&hwbench, false) {
            Err(err) if role.is_authority() => {
                log::warn!(
					"⚠️  The hardware does not meet the minimal requirements {} for role 'Authority'.",
					err
				);
            }
            _ => {}
        }

        if let Some(ref mut telemetry) = telemetry {
            let telemetry_handle = telemetry.handle();
            task_manager.spawn_handle().spawn(
                "telemetry_hwbench",
                None,
                sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
            );
        }
    }

    let (block_import, grandpa_link, babe_link, beefy_links) = import_setup;

    (with_startup_data)(&block_import, &babe_link);

    if let sc_service::config::Role::Authority { .. } = &role {
        let proposer = sc_basic_authorship::ProposerFactory::new(
            task_manager.spawn_handle(),
            client.clone(),
            transaction_pool.clone(),
            prometheus_registry.as_ref(),
            telemetry.as_ref().map(|x| x.handle()),
        );

        let slot_duration = babe_link.config().slot_duration();
        let babe_config = sc_consensus_babe::BabeParams {
            keystore: keystore_container.keystore(),
            client: client.clone(),
            select_chain,
            env: proposer,
            block_import,
            sync_oracle: sync_service.clone(),
            justification_sync_link: sync_service.clone(),
            create_inherent_data_providers: move |_parent, ()| async move {
                let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

                let slot =
                        sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
                            *timestamp,
                            slot_duration,
                        );

                Ok((slot, timestamp))
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
                    public_addresses: auth_disc_public_addresses,
                    persisted_cache_directory: net_config_path,
                    ..Default::default()
                },
                client.clone(),
                Arc::new(network.clone()),
                Box::pin(dht_event_stream),
                authority_discovery_role,
                prometheus_registry.clone(),
                task_manager.spawn_handle(),
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
        Some(keystore_container.keystore())
    } else {
        None
    };

    if enable_beefy {
        // beefy is enabled if its notification service exists
        let network_params = beefy::BeefyNetworkParams {
            network: Arc::new(network.clone()),
            sync: sync_service.clone(),
            gossip_protocol_name: beefy_gossip_proto_name,
            justifications_protocol_name: beefy_on_demand_justifications_handler.protocol_name(),
            notification_service: beefy_notification_service,
            _phantom: core::marker::PhantomData::<Block>,
        };
        let beefy_params = beefy::BeefyParams {
            client: client.clone(),
            backend: backend.clone(),
            payload_provider: sp_consensus_beefy::mmr::MmrRootProvider::new(client.clone()),
            runtime: client.clone(),
            key_store: keystore.clone(),
            network_params,
            min_block_delta: 8,
            prometheus_registry: prometheus_registry.clone(),
            links: beefy_links,
            on_demand_justifications_handler: beefy_on_demand_justifications_handler,
            is_authority: role.is_authority(),
        };

        let beefy_gadget = beefy::start_beefy_gadget::<_, _, _, _, _, _, _, _>(beefy_params);
        // BEEFY is part of consensus, if it fails we'll bring the node down with it to make sure it
        // is noticed.
        task_manager
            .spawn_essential_handle()
            .spawn_blocking("beefy-gadget", None, beefy_gadget);
        // When offchain indexing is enabled, MMR gadget should also run.
        if is_offchain_indexing_enabled {
            task_manager.spawn_essential_handle().spawn_blocking(
                "mmr-gadget",
                None,
                mmr_gadget::MmrGadget::start(
                    client.clone(),
                    backend.clone(),
                    sp_mmr_primitives::INDEXING_PREFIX.to_vec(),
                ),
            );
        }
    }

    let grandpa_config = grandpa::Config {
        // FIXME #1578 make this available through chainspec
        gossip_duration: std::time::Duration::from_millis(333),
        justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
        name: Some(name),
        observer_enabled: false,
        keystore,
        local_role: role.clone(),
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
        let grandpa_params = grandpa::GrandpaParams {
            config: grandpa_config,
            link: grandpa_link,
            network: network.clone(),
            sync: Arc::new(sync_service.clone()),
            notification_service: grandpa_notification_service,
            telemetry: telemetry.as_ref().map(|x| x.handle()),
            voting_rule: grandpa::VotingRulesBuilder::default().build(),
            prometheus_registry: prometheus_registry.clone(),
            shared_voter_state,
            offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        task_manager.spawn_essential_handle().spawn_blocking(
            "grandpa-voter",
            None,
            grandpa::run_grandpa_voter(grandpa_params)?,
        );
    }

    if enable_offchain_worker {
        let offchain_workers =
            sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
                runtime_api_provider: client.clone(),
                keystore: Some(keystore_container.keystore()),
                offchain_db: backend.offchain_storage(),
                transaction_pool: Some(OffchainTransactionPoolFactory::new(
                    transaction_pool.clone(),
                )),
                network_provider: Arc::new(network.clone()),
                is_validator: role.is_authority(),
                enable_http_requests: true,
                custom_extensions: move |_| vec![],
            })?;

        task_manager.spawn_handle().spawn(
            "offchain-workers-runner",
            "offchain-work",
            offchain_workers
                .run(client.clone(), task_manager.spawn_handle())
                .boxed(),
        );
    }

    Ok(NewFullBase {
        task_manager,
        client,
        network,
        sync: sync_service,
        transaction_pool,
        rpc_handlers,
    })
}

type TaskResult = Result<TaskManager, ServiceError>;

/// Create a new service for a full node, based on the network specified in the chain spec.
pub fn new_full(config: Configuration, cli: Cli) -> TaskResult {
    let enable_beefy = !cli.no_beefy;
    let database_path = config.database.path().map(Path::to_path_buf);

    let task_manager = match config.network.network_backend {
        sc_network::config::NetworkBackendType::Libp2p => {
            network_new_full_base::<sc_network::NetworkWorker<_, _>>(
                config,
                enable_beefy,
                cli.no_hardware_benchmarks,
            )?
        }
        sc_network::config::NetworkBackendType::Litep2p => {
            network_new_full_base::<sc_network::Litep2pNetworkBackend>(
                config,
                enable_beefy,
                cli.no_hardware_benchmarks,
            )?
        }
    };

    if let Some(database_path) = database_path {
        sc_storage_monitor::StorageMonitorService::try_spawn(
            cli.storage_monitor,
            database_path,
            &task_manager.spawn_essential_handle(),
        )
        .map_err(|e| ServiceError::Application(e.into()))?;
    }

    Ok(task_manager)
}

/// Create a new service for a full node, based on the network specified in the chain spec.
fn network_new_full_base<N>(
    config: Configuration,
    enable_beefy: bool,
    disable_hardware_benchmarks: bool,
) -> TaskResult
where
    N: sc_network::NetworkBackend<Block, <Block as BlockT>::Hash>,
{
    let network = config.chain_spec.network();
    match network {
        // Run full node for Testnet
        Network::Testnet => new_full_base::<N, polymesh_runtime_testnet::RuntimeApi>(
            config,
            enable_beefy,
            disable_hardware_benchmarks,
            |_, _| (),
        )
        .map(|data| data.task_manager),
        // Run full node for Mainnet
        Network::Mainnet => new_full_base::<N, polymesh_runtime_mainnet::RuntimeApi>(
            config,
            enable_beefy,
            disable_hardware_benchmarks,
            |_, _| (),
        )
        .map(|data| data.task_manager),
        // Run full node for develop/general networks
        Network::Other => new_full_base::<N, polymesh_runtime_develop::RuntimeApi>(
            config,
            enable_beefy,
            disable_hardware_benchmarks,
            |_, _| (),
        )
        .map(|data| data.task_manager),
    }
}

pub(crate) type NewChainOps<R> = (
    Arc<FullClient<R>>,
    Arc<FullBackend>,
    sc_consensus::DefaultImportQueue<Block>,
    TaskManager,
);

/// Builds a new object suitable for chain operations.
pub(crate) fn chain_ops<R>(config: &mut Configuration) -> Result<NewChainOps<R>, ServiceError>
where
    R: ConstructRuntimeApi<Block, FullClient<R>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection,
{
    let sc_service::PartialComponents {
        client,
        backend,
        import_queue,
        task_manager,
        ..
    } = new_partial::<R>(config)?;

    Ok((client, backend, import_queue, task_manager))
}

pub(crate) fn testnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_testnet::RuntimeApi>, ServiceError> {
    chain_ops::<_>(config)
}

pub(crate) fn general_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_develop::RuntimeApi>, ServiceError> {
    chain_ops::<_>(config)
}

pub(crate) fn mainnet_chain_ops(
    config: &mut Configuration,
) -> Result<NewChainOps<polymesh_runtime_mainnet::RuntimeApi>, ServiceError> {
    chain_ops::<_>(config)
}
