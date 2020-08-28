//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

pub use crate::chain_spec::{AlcyoneChainSpec, GeneralChainSpec};
pub use codec::Codec;
use grandpa::{self, FinalityProofProvider as GrandpaFinalityProofProvider};
pub use pallet_confidential::native_rng;
pub use polymesh_primitives::{
    AccountId, Balance, Block, BlockNumber, Hash, IdentityId, Index as Nonce, Moment, Signatory,
    SigningKey, Ticker,
};

pub use polymesh_runtime_develop;
pub use polymesh_runtime_testnet;
use prometheus_endpoint::Registry;
pub use sc_client_api::backend::Backend;
pub use sc_consensus::LongestChain;
use sc_executor::native_executor_instance;
pub use sc_executor::{NativeExecutionDispatch, NativeExecutor};
pub use sc_service::{
    config::{DatabaseConfig, PrometheusConfig},
    error::Error as ServiceError,
    AbstractService, ChainSpec, Configuration, Error, PruningMode, RuntimeGenesis, ServiceBuilder,
    ServiceBuilderCommand, TFullBackend, TFullCallExecutor, TFullClient, TLightBackend,
    TLightCallExecutor, TLightClient, TransactionPoolOptions,
};
pub use sp_api::{ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi, StateBackend};
pub use sp_consensus::SelectChain;
use sp_inherents::InherentDataProviders;
pub use sp_runtime::traits::BlakeTwo256;
use std::{convert::From, sync::Arc};

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
        SigningKey<AccountId>,
        Signatory<AccountId>,
        Moment,
    > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
    + node_rpc_runtime_api::asset::AssetApi<Block, AccountId, Balance>
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
            SigningKey<AccountId>,
            Signatory<AccountId>,
            Moment,
        > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
        + node_rpc_runtime_api::asset::AssetApi<Block, AccountId, Balance>
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

/// Starts a `ServiceBuilder` for a full service.
///
/// Use this macro if you don't actually need the full service, but just the builder in order to
/// be able to perform chain operations.
macro_rules! new_full_start {
    ($config:expr, $runtime:ty, $executor:ty) => {{
        use std::sync::Arc;

        set_prometheus_registry(&mut $config)?;

        let mut import_setup = None;
        let mut rpc_setup = None;
        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<
            polymesh_primitives::Block,
            $runtime,
            $executor,
        >($config)?
        .with_select_chain(|_config, backend| Ok(sc_consensus::LongestChain::new(backend.clone())))?
        .with_transaction_pool(|builder| {
            let pool_api = sc_transaction_pool::FullChainApi::new(builder.client().clone());
            let pool = sc_transaction_pool::BasicPool::new(
                builder.config().transaction_pool.clone(),
                std::sync::Arc::new(pool_api),
                builder.prometheus_registry(),
            );
            Ok(pool)
        })?
        .with_import_queue(
            |_config, client, mut select_chain, _, spawn_task_handle, registry| {
                let select_chain = select_chain
                    .take()
                    .ok_or_else(|| sc_service::Error::SelectChainRequired)?;
                let (grandpa_block_import, grandpa_link) = grandpa::block_import(
                    client.clone(),
                    &(client.clone() as Arc<_>),
                    select_chain,
                )?;

                let justification_import = grandpa_block_import.clone();

                let (block_import, babe_link) = sc_consensus_babe::block_import(
                    sc_consensus_babe::Config::get_or_compute(&*client)?,
                    grandpa_block_import,
                    client.clone(),
                )?;

                let import_queue = sc_consensus_babe::import_queue(
                    babe_link.clone(),
                    block_import.clone(),
                    Some(Box::new(justification_import)),
                    None,
                    client,
                    inherent_data_providers.clone(),
                    spawn_task_handle,
                    registry,
                )?;

                import_setup = Some((block_import, grandpa_link, babe_link));
                Ok(import_queue)
            },
        )?
        .with_rpc_extensions_builder(|builder| {
            let grandpa_link = import_setup
                .as_ref()
                .map(|s| &s.1)
                .expect("GRANDPA LinkHalf is present for full services or set up failed; qed.");

            let shared_authority_set = grandpa_link.shared_authority_set().clone();
            let shared_voter_state = grandpa::SharedVoterState::empty();

            rpc_setup = Some((shared_voter_state.clone()));

            let babe_link = import_setup
                .as_ref()
                .map(|s| &s.2)
                .expect("BabeLink is present for full services or set up failed; qed.");

            let babe_config = babe_link.config().clone();
            let shared_epoch_changes = babe_link.epoch_changes().clone();

            let client = builder.client().clone();
            let pool = builder.pool().clone();
            let select_chain = builder
                .select_chain()
                .cloned()
                .expect("SelectChain is present for full services or set up failed; qed.");
            let keystore = builder.keystore().clone();

            Ok(move |deny_unsafe| {
                let deps = polymesh_node_rpc::FullDeps {
                    client: client.clone(),
                    pool: pool.clone(),
                    select_chain: select_chain.clone(),
                    deny_unsafe,
                    babe: polymesh_node_rpc::BabeDeps {
                        babe_config: babe_config.clone(),
                        shared_epoch_changes: shared_epoch_changes.clone(),
                        keystore: keystore.clone(),
                    },
                    grandpa: polymesh_node_rpc::GrandpaDeps {
                        shared_voter_state: shared_voter_state.clone(),
                        shared_authority_set: shared_authority_set.clone(),
                    },
                };

                polymesh_node_rpc::create_full(deps)
            })
        })?;

        (builder, import_setup, inherent_data_providers, rpc_setup)
    }};
}

/// Builds a new service for a full client.
#[macro_export]
macro_rules! new_full {
    (
        $config:expr,
        $runtime:ty,
        $dispatch:ty,
    ) => {{
        use sc_network::Event;
        use sc_client_api::ExecutorProvider;
        use futures::stream::StreamExt;
        use sp_core::traits::BareCryptoStorePtr;


        let (
            role,
            force_authoring,
            name,
            disable_grandpa,
        ) = (
            $config.role.clone(),
            $config.force_authoring,
            $config.network.node_name.clone(),
            $config.disable_grandpa,
        );

        let _is_authority = role.is_authority();
        let _db_path = match $config.database.path() {
            Some(path) => std::path::PathBuf::from(path),
            None => return Err("Starting a Polkadot service with a custom database isn't supported".to_string().into()),
        };
        //let authority_discovery_enabled = $authority_discovery_enabled;
        //let slot_duration = $slot_duration;

        let (builder, mut import_setup, inherent_data_providers, mut rpc_setup) =
            new_full_start!($config, $runtime, $dispatch);

        let service = builder
            .with_finality_proof_provider(|client, backend| {
                let provider = client as Arc<dyn grandpa::StorageAndProofProvider<_, _>>;
                Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
            })?
            .build_full()?;

        let (block_import, grandpa_link, babe_link) = import_setup.take()
            .expect("Link Half and Block Import are present for Full Services or setup failed before. qed");

        let shared_voter_state = rpc_setup.take()
            .expect("The SharedVoterState is present for Full Services or setup failed before. qed");

            if let sc_service::config::Role::Authority { .. } = &role {
                let proposer = sc_basic_authorship::ProposerFactory::new(
                    service.client(),
                    service.transaction_pool(),
                    service.prometheus_registry().as_ref(),
                );

                let client = service.client();
                let select_chain = service.select_chain()
                    .ok_or(sc_service::Error::SelectChainRequired)?;

                let can_author_with =
                    sp_consensus::CanAuthorWithNativeVersion::new(client.executor().clone());

                let babe_config = sc_consensus_babe::BabeParams {
                    keystore: service.keystore(),
                    client,
                    select_chain,
                    env: proposer,
                    block_import,
                    sync_oracle: service.network(),
                    inherent_data_providers: inherent_data_providers.clone(),
                    force_authoring,
                    babe_link,
                    can_author_with,
                };

                let babe = sc_consensus_babe::start_babe(babe_config)?;
                service.spawn_essential_task_handle().spawn_blocking("babe-proposer", babe);
            }

            // Spawn authority discovery module.
            if matches!(role, sc_service::config::Role::Authority{..} | sc_service::config::Role::Sentry {..}) {
                let (sentries, authority_discovery_role) = match role {
                    sc_service::config::Role::Authority { ref sentry_nodes } => (
                        sentry_nodes.clone(),
                        sc_authority_discovery::Role::Authority (
                            service.keystore(),
                        ),
                    ),
                    sc_service::config::Role::Sentry {..} => (
                        vec![],
                        sc_authority_discovery::Role::Sentry,
                    ),
                    _ => unreachable!("Due to outer matches! constraint; qed.")
                };

                let network = service.network();
                let dht_event_stream = network.event_stream("authority-discovery").filter_map(|e| async move { match e {
                    Event::Dht(e) => Some(e),
                    _ => None,
                }}).boxed();
                let authority_discovery = sc_authority_discovery::AuthorityDiscovery::new(
                    service.client(),
                    network,
                    sentries,
                    dht_event_stream,
                    authority_discovery_role,
                    service.prometheus_registry(),
                );

                service.spawn_task_handle().spawn("authority-discovery", authority_discovery);
            }

            // if the node isn't actively participating in consensus then it doesn't
            // need a keystore, regardless of which protocol we use below.
            let keystore = if role.is_authority() {
                Some(service.keystore() as BareCryptoStorePtr)
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

            let enable_grandpa = !disable_grandpa;
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
                    network: service.network(),
                    inherent_data_providers: inherent_data_providers.clone(),
                    telemetry_on_connect: Some(service.telemetry_on_connect_stream()),
                    voting_rule: grandpa::VotingRulesBuilder::default().build(),
                    prometheus_registry: service.prometheus_registry(),
                    shared_voter_state,
                };

                // the GRANDPA voter task is considered infallible, i.e.
                // if it fails we take down the service with it.
                service.spawn_essential_task_handle().spawn_blocking(
                    "grandpa-voter",
                    grandpa::run_grandpa_voter(grandpa_config)?
                );
            } else {
                grandpa::setup_disabled_grandpa(
                    service.client(),
                    &inherent_data_providers,
                    service.network(),
                )?;
            }

            Ok((service, inherent_data_providers))
        }};
}

/// Create a new Alcyone service for a full node.
pub fn alcyone_new_full(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = polymesh_runtime_testnet::RuntimeApi,
        Backend = TFullBackend<Block>,
    >,
    ServiceError,
> {
    new_full!(
        config,
        polymesh_runtime_testnet::RuntimeApi,
        AlcyoneExecutor,
    )
    .map(|(service, _)| service)
}

/// Create a new General node service for a full node.
pub fn general_new_full(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = polymesh_runtime_develop::RuntimeApi,
        Backend = TFullBackend<Block>,
    >,
    ServiceError,
> {
    new_full!(
        config,
        polymesh_runtime_develop::RuntimeApi,
        GeneralExecutor,
    )
    .map(|(service, _)| service)
}

/// Builds a new object suitable for chain operations.
pub fn chain_ops<Runtime, Dispatch, Extrinsic>(
    mut config: Configuration,
) -> Result<impl ServiceBuilderCommand<Block = Block>, ServiceError>
where
    Runtime:
        ConstructRuntimeApi<Block, TFullClient<Block, Runtime, Dispatch>> + Send + Sync + 'static,
    Runtime::RuntimeApi: RuntimeApiCollection<
        Extrinsic,
        StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
    >,
    Dispatch: NativeExecutionDispatch + 'static,
    Extrinsic: RuntimeExtrinsic,
    <Runtime::RuntimeApi as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
    config.keystore = sc_service::config::KeystoreConfig::InMemory;
    Ok(new_full_start!(config, Runtime, Dispatch).0)
}

/// Builds a new service for a light client.
#[macro_export]
macro_rules! new_light {
    ($config:expr, $runtime:ty, $dispatch:ty) => {{
        set_prometheus_registry(&mut $config)?;
        let inherent_data_providers = InherentDataProviders::new();

        ServiceBuilder::new_light::<Block, $runtime, $dispatch>($config)?
            .with_select_chain(|_, backend| {
                Ok(sc_consensus::LongestChain::new(backend.clone()))
            })?
            .with_transaction_pool(|builder| {
                let fetcher = builder.fetcher()
                    .ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;
                let pool_api = sc_transaction_pool::LightChainApi::new(
                    builder.client().clone(),
                    fetcher,
                );
                let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
                    builder.config().transaction_pool.clone(),
                    Arc::new(pool_api),
                    builder.prometheus_registry(),
                    sc_transaction_pool::RevalidationType::Light,
                );
                Ok(pool)
            })?
            .with_import_queue_and_fprb(|
                _config,
                client,
                backend,
                fetcher,
                _select_chain,
                _,
                spawn_task_handle,
                registry,
            | {
                let fetch_checker = fetcher
                    .map(|fetcher| fetcher.checker().clone())
                    .ok_or_else(|| "Trying to start light import queue without active fetch checker")?;
                let grandpa_block_import = grandpa::light_block_import(
                    client.clone(), backend, &(client.clone() as Arc<_>), Arc::new(fetch_checker)
                )?;

                let finality_proof_import = grandpa_block_import.clone();
                let finality_proof_request_builder =
                    finality_proof_import.create_finality_proof_request_builder();

                let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
                    sc_consensus_babe::Config::get_or_compute(&*client)?,
                    grandpa_block_import,
                    client.clone(),
                )?;

                // FIXME: pruning task isn't started since light client doesn't do `AuthoritySetup`.
                let import_queue = sc_consensus_babe::import_queue(
                    babe_link,
                    babe_block_import,
                    None,
                    Some(Box::new(finality_proof_import)),
                    client,
                    inherent_data_providers.clone(),
                    spawn_task_handle,
                    registry,
                )?;

                Ok((import_queue, finality_proof_request_builder))
            })?
            .with_finality_proof_provider(|client, backend| {
                let provider = client as Arc<dyn grandpa::StorageAndProofProvider<_, _>>;
                Ok(Arc::new(grandpa::FinalityProofProvider::new(backend, provider)) as _)
            })?
            .with_rpc_extensions(|builder| {
                let fetcher = builder.fetcher()
                    .ok_or_else(|| "Trying to start node RPC without active fetcher")?;
                let remote_blockchain = builder.remote_backend()
                    .ok_or_else(|| "Trying to start node RPC without active remote blockchain")?;

                let light_deps = polymesh_node_rpc::LightDeps {
                    remote_blockchain,
                    fetcher,
                    client: builder.client().clone(),
                    pool: builder.pool(),
                };
                Ok(polymesh_node_rpc::create_light(light_deps))
            })?
            .build_light()
    }}
}

/// Create a new Polymesh service for a light client.
pub fn alcyone_new_light(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = polymesh_runtime_testnet::RuntimeApi,
        Backend = TLightBackend<Block>,
        SelectChain = LongestChain<TLightBackend<Block>, Block>,
        CallExecutor = TLightCallExecutor<Block, AlcyoneExecutor>,
    >,
    ServiceError,
> {
    new_light!(
        config,
        polymesh_runtime_testnet::RuntimeApi,
        AlcyoneExecutor
    )
}

/// Create a new Polymesh service for a light client.
pub fn general_new_light(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = polymesh_runtime_develop::RuntimeApi,
        Backend = TLightBackend<Block>,
        SelectChain = LongestChain<TLightBackend<Block>, Block>,
        CallExecutor = TLightCallExecutor<Block, GeneralExecutor>,
    >,
    ServiceError,
> {
    new_light!(
        config,
        polymesh_runtime_develop::RuntimeApi,
        GeneralExecutor
    )
}
