//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

pub use codec::Codec;
use grandpa::{
    self, FinalityProofProvider as GrandpaFinalityProofProvider, StorageAndProofProvider,
};
pub use polymesh_primitives::{
    AccountId, AccountKey, Balance, Block, BlockNumber, Hash, IdentityId, Index as Nonce, Moment,
    Signatory, SigningItem, Ticker,
};
pub use polymesh_runtime_develop;
pub use polymesh_runtime_testnet_v1;
use prometheus_endpoint::Registry;
pub use sc_client::CallExecutor;
use sc_client::{self, Client, LongestChain};
pub use sc_client_api::backend::Backend;
use sc_consensus_babe;
use sc_executor::native_executor_instance;
pub use sc_executor::{NativeExecutionDispatch, NativeExecutor};
pub use sc_service::{
    config::{full_version_from_strs, DatabaseConfig, PrometheusConfig},
    error::Error as ServiceError,
    AbstractService, Error, PruningMode, Roles, RuntimeGenesis, ServiceBuilder,
    ServiceBuilderCommand, TFullBackend, TFullCallExecutor, TFullClient, TLightBackend,
    TLightCallExecutor, TLightClient, TransactionPoolOptions,
};
pub use sp_api::{ConstructRuntimeApi, Core as CoreApi, ProvideRuntimeApi, StateBackend};
pub use sp_consensus::SelectChain;
use sp_inherents::InherentDataProviders;
pub use sp_runtime::traits::BlakeTwo256;
use std::{convert::From, sync::Arc};

pub type Configuration =
    sc_service::Configuration<polymesh_runtime_testnet_v1::config::GenesisConfig>;

// Our native executor instance.
native_executor_instance!(
    pub V1Executor,
    polymesh_runtime_testnet_v1::api::dispatch,
    polymesh_runtime_testnet_v1::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

// Our native executor instance.
native_executor_instance!(
    pub GeneralExecutor,
    polymesh_runtime_develop::api::dispatch,
    polymesh_runtime_develop::native_version,
    frame_benchmarking::benchmarking::HostFunctions,
);

/// A set of APIs that polkadot-like runtimes must implement.
pub trait RuntimeApiCollection<Extrinsic: codec::Codec + Send + Sync + 'static>:
    sp_transaction_pool::runtime_api::TaggedTransactionQueue<Block>
    + sp_api::ApiExt<Block, Error = sp_blockchain::Error>
    + sp_consensus_babe::BabeApi<Block>
    + sp_block_builder::BlockBuilder<Block>
    + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
    + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
    + sp_api::Metadata<Block>
    + sp_offchain::OffchainWorkerApi<Block>
    + sp_session::SessionKeys<Block>
    + sp_authority_discovery::AuthorityDiscoveryApi<Block>
    + pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
    + pallet_staking_rpc_runtime_api::StakingApi<Block>
    + node_rpc_runtime_api::pips::PipsApi<Block, AccountId, Balance>
    + pallet_identity_rpc_runtime_api::IdentityApi<
        Block,
        IdentityId,
        Ticker,
        AccountKey,
        SigningItem,
        Signatory,
        Moment,
    > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
    + node_rpc_runtime_api::asset::AssetApi<Block, AccountId, Balance>
    + pallet_group_rpc_runtime_api::GroupApi<Block>
    + pallet_compliance_manager_rpc_runtime_api::ComplianceManagerApi<Block, AccountId, Balance>
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
        + sp_block_builder::BlockBuilder<Block>
        + frame_system_rpc_runtime_api::AccountNonceApi<Block, AccountId, Nonce>
        + pallet_transaction_payment_rpc_runtime_api::TransactionPaymentApi<Block, Balance, Extrinsic>
        + sp_api::Metadata<Block>
        + sp_offchain::OffchainWorkerApi<Block>
        + sp_session::SessionKeys<Block>
        + sp_authority_discovery::AuthorityDiscoveryApi<Block>
        + pallet_contracts_rpc_runtime_api::ContractsApi<Block, AccountId, Balance, BlockNumber>
        + pallet_staking_rpc_runtime_api::StakingApi<Block>
        + node_rpc_runtime_api::pips::PipsApi<Block, AccountId, Balance>
        + pallet_identity_rpc_runtime_api::IdentityApi<
            Block,
            IdentityId,
            Ticker,
            AccountKey,
            SigningItem,
            Signatory,
            Moment,
        > + pallet_protocol_fee_rpc_runtime_api::ProtocolFeeApi<Block>
        + node_rpc_runtime_api::asset::AssetApi<Block, AccountId, Balance>
        + pallet_group_rpc_runtime_api::GroupApi<Block>
        + pallet_compliance_manager_rpc_runtime_api::ComplianceManagerApi<Block, AccountId, Balance>,
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

        type RpcExtension = jsonrpc_core::IoHandler<sc_rpc::Metadata>;
        let mut import_setup = None;
        let inherent_data_providers = sp_inherents::InherentDataProviders::new();

        let builder = sc_service::ServiceBuilder::new_full::<
            polymesh_primitives::Block,
            $runtime,
            $executor,
        >($config)?
        .with_select_chain(|_config, backend| Ok(sc_client::LongestChain::new(backend.clone())))?
        .with_transaction_pool(|config, client, _fetcher| {
            let pool_api = sc_transaction_pool::FullChainApi::new(client.clone());
            Ok(sc_transaction_pool::BasicPool::new(
                config,
                std::sync::Arc::new(pool_api),
            ))
        })?
        .with_import_queue(|_config, client, mut select_chain, _transaction_pool| {
            let select_chain = select_chain
                .take()
                .ok_or_else(|| sc_service::Error::SelectChainRequired)?;
            let (grandpa_block_import, grandpa_link) =
                grandpa::block_import(client.clone(), &(client.clone() as Arc<_>), select_chain)?;

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
            )?;

            import_setup = Some((block_import, grandpa_link, babe_link));
            Ok(import_queue)
        })?
        .with_rpc_extensions(|builder| -> Result<RpcExtension, _> {
            use contracts_rpc::{Contracts, ContractsApi};
            use node_rpc::{
                asset::{Asset, AssetApi},
                pips::{Pips, PipsApi},
            };
            use pallet_compliance_manager_rpc::{ComplianceManager, ComplianceManagerApi};
            use pallet_group_rpc::{Group, GroupApi};
            use pallet_identity_rpc::{Identity, IdentityApi};
            use pallet_protocol_fee_rpc::{ProtocolFee, ProtocolFeeApi};
            use pallet_staking_rpc::{Staking, StakingApi};
            use pallet_transaction_payment_rpc::{TransactionPayment, TransactionPaymentApi};

            // register contracts RPC extension
            let mut io = jsonrpc_core::IoHandler::default();
            io.extend_with(ContractsApi::to_delegate(Contracts::new(
                builder.client().clone(),
            )));
            io.extend_with(TransactionPaymentApi::to_delegate(TransactionPayment::new(
                builder.client().clone(),
            )));
            io.extend_with(StakingApi::to_delegate(Staking::new(
                builder.client().clone(),
            )));
            io.extend_with(PipsApi::to_delegate(Pips::new(builder.client().clone())));
            io.extend_with(IdentityApi::to_delegate(Identity::new(
                builder.client().clone(),
            )));
            io.extend_with(ProtocolFeeApi::to_delegate(ProtocolFee::new(
                builder.client().clone(),
            )));
            io.extend_with(AssetApi::to_delegate(Asset::new(builder.client().clone())));
            io.extend_with(GroupApi::to_delegate(Group::from(builder.client().clone())));
            io.extend_with(ComplianceManagerApi::to_delegate(ComplianceManager::new(
                builder.client().clone(),
            )));

            Ok(io)
        })?;

        (builder, import_setup, inherent_data_providers)
    }};
}

/// Builds a new service for a full client.
pub fn new_full<Runtime, Dispatch, Extrinsic>(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = Runtime,
        Backend = TFullBackend<Block>,
        SelectChain = LongestChain<TFullBackend<Block>, Block>,
        CallExecutor = TFullCallExecutor<Block, Dispatch>,
    >,
    ServiceError,
>
where
    Runtime:
        ConstructRuntimeApi<Block, TFullClient<Block, Runtime, Dispatch>> + Send + Sync + 'static,
    Runtime::RuntimeApi: RuntimeApiCollection<
        Extrinsic,
        StateBackend = sc_client_api::StateBackendFor<TFullBackend<Block>, Block>,
    >,
    Dispatch: NativeExecutionDispatch + 'static,
    Extrinsic: RuntimeExtrinsic,
    // Rust bug: https://github.com/rust-lang/rust/issues/24159
    <Runtime::RuntimeApi as sp_api::ApiExt<Block>>::StateBackend: sp_api::StateBackend<BlakeTwo256>,
{
    use futures::prelude::*;
    use sc_client_api::ExecutorProvider;
    use sc_network::Event;

    let is_authority = config.roles.is_authority();
    let force_authoring = config.force_authoring;
    let name = config.name.clone();
    let disable_grandpa = config.disable_grandpa;
    let sentry_nodes = config.network.sentry_nodes.clone();

    // sentry nodes announce themselves as authorities to the network
    // and should run the same protocols authorities do, but it should
    // never actively participate in any consensus process.
    let participates_in_consensus = is_authority && !config.sentry_mode;

    let (builder, mut import_setup, inherent_data_providers) =
        new_full_start!(config, Runtime, Dispatch);

    let service = builder
        .with_finality_proof_provider(|client, backend| {
            // GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
            let provider = client as Arc<dyn grandpa::StorageAndProofProvider<_, _>>;
            Ok(Arc::new(grandpa::FinalityProofProvider::new(backend, provider)) as _)
        })?
        .build()?;

    let (block_import, grandpa_link, babe_link) = import_setup.take().expect(
        "Link Half and Block Import are present for Full Services or setup failed before. qed",
    );

    if participates_in_consensus {
        let proposer =
            sc_basic_authorship::ProposerFactory::new(service.client(), service.transaction_pool());

        let client = service.client();
        let select_chain = service
            .select_chain()
            .ok_or(ServiceError::SelectChainRequired)?;
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
        service.spawn_essential_task("babe-proposer", babe);

        let network = service.network();
        let dht_event_stream = network
            .event_stream()
            .filter_map(|e| async move {
                match e {
                    Event::Dht(e) => Some(e),
                    _ => None,
                }
            })
            .boxed();
        let authority_discovery = sc_authority_discovery::AuthorityDiscovery::new(
            service.client(),
            network,
            sentry_nodes,
            service.keystore(),
            dht_event_stream,
        );

        service.spawn_task("authority-discovery", authority_discovery);
    }

    // if the node isn't actively participating in consensus then it doesn't
    // need a keystore, regardless of which protocol we use below.
    let keystore = if participates_in_consensus {
        Some(service.keystore())
    } else {
        None
    };

    let config = grandpa::Config {
        // FIXME #1578 make this available through chain_spec
        gossip_duration: std::time::Duration::from_millis(333),
        justification_period: 512,
        name: Some(name),
        observer_enabled: false,
        keystore,
        is_authority,
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
        };

        // the GRANDPA voter task is considered infallible, i.e.
        // if it fails we take down the service with it.
        service.spawn_essential_task("grandpa-voter", grandpa::run_grandpa_voter(grandpa_config)?);
    } else {
        grandpa::setup_disabled_grandpa(
            service.client(),
            &inherent_data_providers,
            service.network(),
        )?;
    }

    Ok(service)
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

pub type TLocalLightClient<Runtime, Dispatch> = Client<
    sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
    sc_client::light::call_executor::GenesisCallExecutor<
        sc_client::light::backend::Backend<sc_client_db::light::LightStorage<Block>, BlakeTwo256>,
        sc_client::LocalCallExecutor<
            sc_client::light::backend::Backend<
                sc_client_db::light::LightStorage<Block>,
                BlakeTwo256,
            >,
            sc_executor::NativeExecutor<Dispatch>,
        >,
    >,
    Block,
    Runtime,
>;

/// Builds a new service for a light client.
pub fn new_light<Runtime, Dispatch, Extrinsic>(
    mut config: Configuration,
) -> Result<
    impl AbstractService<
        Block = Block,
        RuntimeApi = Runtime,
        Backend = TLightBackend<Block>,
        SelectChain = LongestChain<TLightBackend<Block>, Block>,
        CallExecutor = TLightCallExecutor<Block, Dispatch>,
    >,
    ServiceError,
>
where
    Runtime: Send + Sync + 'static,
    Runtime::RuntimeApi: RuntimeApiCollection<
        Extrinsic,
        StateBackend = sc_client_api::StateBackendFor<TLightBackend<Block>, Block>,
    >,
    Dispatch: NativeExecutionDispatch + 'static,
    Extrinsic: RuntimeExtrinsic,
    Runtime: sp_api::ConstructRuntimeApi<Block, TLocalLightClient<Runtime, Dispatch>>,
{
    set_prometheus_registry(&mut config)?;

    let inherent_data_providers = InherentDataProviders::new();

    let service = ServiceBuilder::new_light::<Block, Runtime, Dispatch>(config)?
        .with_select_chain(|_config, backend| Ok(LongestChain::new(backend.clone())))?
        .with_transaction_pool(|config, client, fetcher| {
            let fetcher = fetcher
                .ok_or_else(|| "Trying to start light transaction pool without active fetcher")?;
            let pool_api = sc_transaction_pool::LightChainApi::new(client.clone(), fetcher.clone());
            let pool = sc_transaction_pool::BasicPool::with_revalidation_type(
                config,
                Arc::new(pool_api),
                sc_transaction_pool::RevalidationType::Light,
            );
            Ok(pool)
        })?
        .with_import_queue_and_fprb(
            |_config, client, backend, fetcher, _select_chain, _tx_pool| {
                let fetch_checker = fetcher
                    .map(|fetcher| fetcher.checker().clone())
                    .ok_or_else(|| {
                        "Trying to start light import queue without active fetch checker"
                    })?;
                let grandpa_block_import = grandpa::light_block_import(
                    client.clone(),
                    backend,
                    &(client.clone() as Arc<_>),
                    Arc::new(fetch_checker),
                )?;

                let finality_proof_import = grandpa_block_import.clone();
                let finality_proof_request_builder =
                    finality_proof_import.create_finality_proof_request_builder();

                let (babe_block_import, babe_link) = sc_consensus_babe::block_import(
                    sc_consensus_babe::Config::get_or_compute(&*client)?,
                    grandpa_block_import,
                    client.clone(),
                )?;

                let import_queue = sc_consensus_babe::import_queue(
                    babe_link,
                    babe_block_import,
                    None,
                    Some(Box::new(finality_proof_import)),
                    client.clone(),
                    inherent_data_providers.clone(),
                )?;

                Ok((import_queue, finality_proof_request_builder))
            },
        )?
        .with_finality_proof_provider(|client, backend| {
            // GenesisAuthoritySetProvider is implemented for StorageAndProofProvider
            let provider = client as Arc<dyn StorageAndProofProvider<_, _>>;
            Ok(Arc::new(GrandpaFinalityProofProvider::new(backend, provider)) as _)
        })?
        .build()?;

    Ok(service)
}
