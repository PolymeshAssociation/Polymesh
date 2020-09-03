//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

pub use crate::chain_spec::{AlcyoneChainSpec, GeneralChainSpec};
pub use codec::Codec;
pub use pallet_confidential::native_rng;
pub use polymesh_primitives::{
    AccountId, Balance, Block, BlockNumber, Hash, IdentityId, Index as Nonce, Moment, SecondaryKey,
    Signatory, Ticker,
};
use sc_client_api::ExecutorProvider;
use grandpa::{FinalityProofProvider as GrandpaFinalityProofProvider, StorageAndProofProvider};
pub use polymesh_runtime_develop;
pub use polymesh_runtime_testnet;
use prometheus_endpoint::Registry;
pub use sc_client_api::{RemoteBackend, backend::Backend};
pub use sc_consensus::LongestChain;
use sc_executor::native_executor_instance;
pub use sc_executor::{NativeExecutionDispatch, NativeExecutor};
pub use sc_service::{
    config::{Role, DatabaseConfig, PrometheusConfig},
    error::Error as ServiceError,
    ChainSpec, Configuration, Error, PruningMode, RuntimeGenesis,
    TFullBackend, TFullCallExecutor, TFullClient, TLightBackend,
    TLightCallExecutor, TLightClient, TransactionPoolOptions,
};
pub use sp_api::{ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi, StateBackend};
pub use sp_consensus::SelectChain;
use sp_inherents::InherentDataProviders;
pub use sp_runtime::traits::BlakeTwo256;
use std::sync::Arc;
use sc_network::{Event, NetworkService};
use sc_service::{TaskManager, RpcHandlers, ServiceComponents};
use sp_core::traits::BareCryptoStorePtr;
use sp_runtime::traits::Block as BlockT;

pub trait IsAlcyoneNetwork {
    fn is_alcyone_network(&self) -> bool;
}

impl IsAlcyoneNetwork for dyn ChainSpec {
    fn is_alcyone_network(&self) -> bool {
        self.name().starts_with("Polymesh Alcyone")
    }
}

// Our native executor instance.
native_executor_instance!(
    pub AlcyoneExecutor,
    polymesh_runtime_testnet::api::dispatch,
    polymesh_runtime_testnet::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

// Our native executor instance.
native_executor_instance!(
    pub GeneralExecutor,
    polymesh_runtime_develop::api::dispatch,
    polymesh_runtime_develop::native_version,
    (frame_benchmarking::benchmarking::HostFunctions, native_rng::HostFunctions)
);

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: codec::Codec + Send + Sync + 'static>:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block, Error = sp_blockchain::Error>
    + sp_consensus_babe::BabeApi<Block>
    + grandpa_primitives::GrandpaApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
    + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block, Balance, Extrinsic>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
    + pallet_staking_rpc_runtime_api::StakingApi<Block>
    + node_rpc_runtime_api::pips::PipsApi<Block, AccountId, Balance>
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
    + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId, Balance>
where
    Extrinsic: RuntimeExtrinsic,
    <Self as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
}

impl<Api, Extrinsic> RuntimeApiCollection<Extrinsic> for Api
where
    Api: sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
        + sp_api::ApiExt<Block, Error = sp_blockchain::Error>
        + sp_consensus_babe::BabeApi<Block>
        + grandpa_primitives::GrandpaApi<Block>
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
        + node_rpc_runtime_api::transaction_payment::TransactionPaymentApi<Block, Balance, Extrinsic>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
        + pallet_staking_rpc_runtime_api::StakingApi<Block>
        + node_rpc_runtime_api::pips::PipsApi<Block, AccountId, Balance>
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
        + node_rpc_runtime_api::compliance_manager::ComplianceManagerApi<Block, AccountId, Balance>,
    Extrinsic: RuntimeExtrinsic,
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

use polymesh_node_rpc as node_rpc;

type BabeLink = sc_consensus_babe::BabeLink<Block>;
type IoHandler = jsonrpc_core::IoHandler<sc_rpc::Metadata>;

type FullLinkHalf<R, D> = grandpa::LinkHalf<Block, FullClient<R, D>, FullSelectChain>;
type FullClient<R, E> = sc_service::TFullClient<Block, R, E>;
type FullBackend = sc_service::TFullBackend<Block>;
type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<R, E> = grandpa::GrandpaBlockImport<FullBackend, Block, FullClient<R, E>, FullSelectChain>;
type FullBabeImportQueue<R, E> = sc_consensus_babe::BabeImportQueue<Block, FullClient<R, E>>;
type FullStateBackend = sc_client_api::StateBackendFor<FullBackend, Block>;
type FullPool<R, E> = sc_transaction_pool::FullPool<Block, FullClient<R, E>>;
type FullServiceParams<R, E> = sc_service::ServiceParams<
    Block,
    FullClient<R, E>,
    FullBabeImportQueue<R, E>,
    FullPool<R, E>,
    IoHandler,
    FullBackend,
>;
type FullBabeBlockImport<R, E> = sc_consensus_babe::BabeBlockImport<
    Block,
    FullClient<R, E>,
    FullGrandpaBlockImport<R, E>,
>;

pub fn new_full_params<R, D, E>(
    config: Configuration,
) -> Result<
    (
        FullServiceParams<R, D>,
        (FullBabeBlockImport<R, D>, FullLinkHalf<R, D>, BabeLink),
        grandpa::SharedVoterState,
        FullSelectChain,
        InherentDataProviders,
    ),
    ServiceError,
>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    let (client, backend, keystore, task_manager) =
        sc_service::new_full_parts::<Block, R, D>(&config)?;
    let client = Arc::new(client);

    let select_chain = sc_consensus::LongestChain::new(backend.clone());

    let pool_api =
        sc_transaction_pool::FullChainApi::new(client.clone(), config.prometheus_registry());
    let transaction_pool = sc_transaction_pool::BasicPool::new_full(
        config.transaction_pool.clone(),
        std::sync::Arc::new(pool_api),
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
        None,
        client.clone(),
        select_chain.clone(),
        inherent_data_providers.clone(),
        &task_manager.spawn_handle(),
        config.prometheus_registry(),
    )?;

    let import_setup = (block_import, grandpa_link, babe_link);

    let (rpc_extensions_builder, rpc_setup) = {
        let (_, grandpa_link, babe_link) = &import_setup;

        let shared_authority_set = grandpa_link.shared_authority_set().clone();
        let shared_voter_state = grandpa::SharedVoterState::empty();

        let rpc_setup = shared_voter_state.clone();

        let babe_config = babe_link.config().clone();
        let shared_epoch_changes = babe_link.epoch_changes().clone();

        let client = client.clone();
        let pool = transaction_pool.clone();
        let select_chain = select_chain.clone();
        let keystore = keystore.clone();

        let rpc_extensions_builder = Box::new(move |deny_unsafe| {
            let deps = node_rpc::FullDeps {
                client: client.clone(),
                pool: pool.clone(),
                select_chain: select_chain.clone(),
                deny_unsafe,
                babe: node_rpc::BabeDeps {
                    babe_config: babe_config.clone(),
                    shared_epoch_changes: shared_epoch_changes.clone(),
                    keystore: keystore.clone(),
                },
                grandpa: node_rpc::GrandpaDeps {
                    shared_voter_state: shared_voter_state.clone(),
                    shared_authority_set: shared_authority_set.clone(),
                },
            };

            node_rpc::create_full(deps)
        });

        (rpc_extensions_builder, rpc_setup)
    };

    let provider = client.clone() as Arc<dyn grandpa::StorageAndProofProvider<_, _>>;
    let finality_proof_provider = Arc::new(grandpa::FinalityProofProvider::new(
        backend.clone(),
        provider,
    ));

    let params = sc_service::ServiceParams {
        config,
        backend,
        client,
        import_queue,
        keystore,
        task_manager,
        rpc_extensions_builder,
        transaction_pool,
        block_announce_validator_builder: None,
        finality_proof_request_builder: None,
        finality_proof_provider: Some(finality_proof_provider),
        on_demand: None,
        remote_blockchain: None,
    };

    Ok((
        params,
        import_setup,
        rpc_setup,
        select_chain,
        inherent_data_providers,
    ))
}

/// Creates a full service from the configuration.
pub fn new_full_base<R, D, E, F>(
	mut config: Configuration,
	with_startup_data: F,
) -> Result<(
	TaskManager, InherentDataProviders, Arc<FullClient<R, D>>,
	Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
	Arc<FullPool<R, D>>,
), ServiceError>
where
    F: FnOnce(&FullBabeBlockImport<R, D>, &BabeLink),
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    set_prometheus_registry(&mut config)?;

	let (params, import_setup, rpc_setup, select_chain, inherent_data_providers)
		= new_full_params(config)?;

	let (
		role, force_authoring, name, enable_grandpa, prometheus_registry,
		client, transaction_pool, keystore,
	) = {
		let sc_service::ServiceParams {
			config, client, transaction_pool, keystore, ..
		} = &params;

		(
			config.role.clone(),
			config.force_authoring,
			config.network.node_name.clone(),
			!config.disable_grandpa,
			config.prometheus_registry().cloned(),

			client.clone(), transaction_pool.clone(), keystore.clone(),
		)
	};

	let ServiceComponents {
		task_manager, network, telemetry_on_connect_sinks, ..
	} = sc_service::build(params)?;

	let (block_import, grandpa_link, babe_link) = import_setup;
	let shared_voter_state = rpc_setup;

	(with_startup_data)(&block_import, &babe_link);

	if let sc_service::config::Role::Authority { .. } = &role {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
		);

		let can_author_with =
			sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore.clone(),
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: network.clone(),
			inherent_data_providers: inherent_data_providers.clone(),
			force_authoring,
			babe_link,
			can_author_with,
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking("babe-proposer", babe);
	}

	// Spawn authority discovery module.
	if matches!(role, Role::Authority{..} | Role::Sentry {..}) {
		let (sentries, authority_discovery_role) = match role {
			sc_service::config::Role::Authority { ref sentry_nodes } => (
				sentry_nodes.clone(),
				sc_authority_discovery::Role::Authority (
					keystore.clone(),
				),
			),
			sc_service::config::Role::Sentry {..} => (
				vec![],
				sc_authority_discovery::Role::Sentry,
			),
			_ => unreachable!("Due to outer matches! constraint; qed.")
        };

        use futures::stream::StreamExt;
		let dht_event_stream = network.event_stream("authority-discovery")
			.filter_map(|e| async move { match e {
				Event::Dht(e) => Some(e),
				_ => None,
			}}).boxed();
		let authority_discovery = sc_authority_discovery::AuthorityDiscovery::new(
			client.clone(),
			network.clone(),
			sentries,
			dht_event_stream,
			authority_discovery_role,
			prometheus_registry.clone(),
		);

		task_manager.spawn_handle().spawn("authority-discovery", authority_discovery);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore = if role.is_authority() {
		Some(keystore as BareCryptoStorePtr)
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
		is_authority: role.is_network_authority(),
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
			inherent_data_providers: inherent_data_providers.clone(),
			telemetry_on_connect: Some(telemetry_on_connect_sinks.on_connect_stream()),
			voting_rule: grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			grandpa::run_grandpa_voter(grandpa_config)?
		);
	} else {
		grandpa::setup_disabled_grandpa(
			client.clone(),
			&inherent_data_providers,
			network.clone(),
		)?;
	}

	Ok((task_manager, inherent_data_providers, client, network, transaction_pool))
}

type TaskResult = Result<TaskManager, ServiceError>;

/// Create a new Alcyone service for a full node.
pub fn alcyone_new_full(config: Configuration) -> TaskResult {
    new_full_base::<
        polymesh_runtime_testnet::RuntimeApi,
        AlcyoneExecutor,
        _,
        _,
    >(config, |_, _| ()).map(|(task_manager, _, _, _, _)| {
		task_manager
    })
}

/// Create a new General node service for a full node.
pub fn general_new_full(config: Configuration) -> TaskResult {
    new_full_base::<
        polymesh_runtime_develop::RuntimeApi,
        GeneralExecutor,
        _,
        _,
    >(config, |_, _| ()).map(|(task_manager, _, _, _, _)| {
		task_manager
    })
}

/// Builds a new object suitable for chain operations.
pub fn chain_ops<R, D, E>(
    mut config: Configuration,
) -> Result<(Arc<FullClient<R, D>>, Arc<FullBackend>, FullBabeImportQueue<R, D>, TaskManager), ServiceError>
where
    R: ConstructRuntimeApi<Block, FullClient<R, D>> + Send + Sync + 'static,
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = FullStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
{
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    let (FullServiceParams { client, backend, import_queue, task_manager, .. }, ..)
        = new_full_params::<R, D, E>(config)?;
    Ok((client, backend, import_queue, task_manager))
}

type LightStorage = sc_client_db::light::LightStorage<Block>;
type LightBackend = sc_light::backend::Backend<LightStorage, polymesh_primitives::BlakeTwo256>;
type LightClient<R, E> = sc_service::TLightClient<Block, R, E>;
type LightStateBackend = sc_client_api::StateBackendFor<LightBackend, Block>;
type LightPool<R, E> = sc_transaction_pool::LightPool<Block, LightClient<R, E>, sc_network::config::OnDemand<Block>>;

pub fn new_light_base<R, D, E>(config: Configuration) -> Result<(
	TaskManager, Arc<RpcHandlers>, Arc<LightClient<R, D>>,
	Arc<NetworkService<Block, <Block as BlockT>::Hash>>,
	Arc<LightPool<R, D>>,
), ServiceError>
where
    //R: ConstructRuntimeApi<Block, LightClient<R, D>> +
    R::RuntimeApi: RuntimeApiCollection<E, StateBackend = LightStateBackend>,
    D: NativeExecutionDispatch + 'static,
    E: RuntimeExtrinsic,
    //<R::RuntimeApi as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
    R: Send + Sync + 'static + sp_api::ConstructRuntimeApi<
        Block,
        sc_service::TLightClientWithBackend<Block, R, D, LightBackend>,
    >,
{
	let (client, backend, keystore, task_manager, on_demand) =
		sc_service::new_light_parts::<Block, R, D>(&config)?;

	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool_api = Arc::new(sc_transaction_pool::LightChainApi::new(
		client.clone(),
		on_demand.clone(),
	));
	let transaction_pool = Arc::new(sc_transaction_pool::BasicPool::new_light(
		config.transaction_pool.clone(),
		transaction_pool_api,
		config.prometheus_registry(),
		task_manager.spawn_handle(),
	));

	let grandpa_block_import = grandpa::light_block_import(
		client.clone(), backend.clone(), &(client.clone() as Arc<_>),
		Arc::new(on_demand.checker().clone()),
	)?;

	let finality_proof_import = grandpa_block_import.clone();
	let finality_proof_request_builder =
		finality_proof_import.create_finality_proof_request_builder();

	let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::Config::get_or_compute(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;

	let inherent_data_providers = sp_inherents::InherentDataProviders::new();

	let import_queue = sc_consensus_babe::import_queue(
		babe_link,
		babe_block_import,
		None,
		Some(Box::new(finality_proof_import)),
		client.clone(),
		select_chain.clone(),
		inherent_data_providers.clone(),
		&task_manager.spawn_handle(),
		config.prometheus_registry(),
	)?;

	// GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
	let provider = client.clone() as Arc<dyn StorageAndProofProvider<_, _>>;
	let finality_proof_provider =
		Arc::new(GrandpaFinalityProofProvider::new(backend.clone(), provider));

	let light_deps = node_rpc::LightDeps {
		remote_blockchain: backend.remote_blockchain(),
		fetcher: on_demand.clone(),
		client: client.clone(),
		pool: transaction_pool.clone(),
	};

	let rpc_extensions = node_rpc::create_light(light_deps);

	let ServiceComponents { task_manager, rpc_handlers, network, .. } =
		sc_service::build(sc_service::ServiceParams {
			block_announce_validator_builder: None,
			finality_proof_request_builder: Some(finality_proof_request_builder),
			finality_proof_provider: Some(finality_proof_provider),
			on_demand: Some(on_demand),
			remote_blockchain: Some(backend.remote_blockchain()),
			rpc_extensions_builder: Box::new(sc_service::NoopRpcExtensionBuilder(rpc_extensions)),
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			config, import_queue, keystore, backend, task_manager,
		})?;

    Ok((task_manager, rpc_handlers, client, network, transaction_pool))
}

/// Create a new Polymesh service for a light client.
pub fn alcyone_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_testnet::RuntimeApi, AlcyoneExecutor, _>(config)
        .map(|(task_manager, _, _, _, _)| task_manager)
}

/// Create a new Polymesh service for a light client.
pub fn general_new_light(config: Configuration) -> TaskResult {
    new_light_base::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _>(config)
        .map(|(task_manager, _, _, _, _)| task_manager)
}
