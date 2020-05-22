//! Polymesh CLI binary.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![feature(box_syntax)]

#[cfg(feature = "runtime-benchmarks")]
mod analysis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking_cli;
mod chain_spec;
#[macro_use]
mod service;
mod cli;
pub mod command;
mod load_chain_spec;

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "Polymesh Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "polymesh",
        author: "Anonymous",
        description: "Polymesh Node",
        support_url: "https://polymath.network/",
        copyright_start_year: 2017,
    };

    command::run(std::env::args(), version)
}


#[cfg(test)]
mod test {
    use crate::{ cli::Cli, load_chain_spec::load_spec };
    use super::*;
    use sp_core::H256;
    use std::{ thread, sync::{Arc, Weak}};
    use sc_service::{ AbstractService, Configuration };
    use tokio::sync::{ mpsc::{self, Receiver, Sender } };
    use futures::{Future, future, select, pin_mut, future::FutureExt};


    use polymesh_runtime_develop::Runtime;
    use polymesh_primitives::IdentityId;


    type PolymeshBlock = sp_runtime::generic::Block<
        sp_runtime::generic::Header<u32, sp_runtime::traits::BlakeTwo256>,
        sp_runtime::OpaqueExtrinsic>;

    type Backend<BlockT> = sc_client_db::Backend<BlockT>;
    type Executor<BackEndT, ExecutorT > =
        sc_client::LocalCallExecutor<
            BackEndT,
            sc_executor::NativeExecutor<ExecutorT>
    >;
    type PolymeshExecutor<BlockT> = Executor<Backend<BlockT>, service::GeneralExecutor>;

    type PolymeshClient<BlockT> = sc_client::Client<
        Backend<BlockT>,
        PolymeshExecutor<BlockT>,
        BlockT,
        polymesh_runtime_develop::runtime::RuntimeApi
    >;

    type PolymeshTransactionPool<BlockT> = sc_transaction_pool::BasicPool<
        sc_transaction_pool::FullChainApi< PolymeshClient<BlockT>, BlockT >,
        BlockT,
    >;

    type PolymeshOffchainWorkers<BlockT> = sc_offchain::OffchainWorkers<
        PolymeshClient<BlockT>,
        sc_client_db::offchain::LocalStorage,
        BlockT
    >;

    type PolymeshService<BlockT> = sc_service::Service<
        BlockT,
        PolymeshClient<BlockT>,
        sc_client::LongestChain< Backend<BlockT>, BlockT>,
        sc_service::NetworkStatus<BlockT>,
        sc_network::NetworkService< BlockT, H256>,
        PolymeshTransactionPool<BlockT>,
        PolymeshOffchainWorkers<BlockT>,
    >;


    struct Node
    {
        client: Arc<PolymeshClient<PolymeshBlock>>,
        exit_tx: Sender<usize>,
    }

    impl Drop for Node
    {
        fn drop(&mut self) {
            // TODO kill node and wait.
            /*if let Some(thread) = self.thread.take() {
                let _ = thread.join();
            }*/
            let _ = self.exit_tx.send(1);
        }
    }

    /*
    lazy_static! {
        static ref NODE: Arc<Mutex<Weak<Node>>> = Arc::new(Mutex::new(Weak::new()));
    }*/
    fn build_runtime() -> Result<tokio::runtime::Runtime, std::io::Error> {
        tokio::runtime::Builder::new()
            .thread_name("main-tokio-")
            .threaded_scheduler()
            .enable_all()
            .build()
    }

    async fn main<F, E>(
        func: F,
        mut exit_rx: Receiver<usize> ) -> Result<(), Box<dyn std::error::Error>>
    where
        F: Future<Output = Result<(), E>> + future::FusedFuture,
        E: 'static + std::error::Error,
    {
        let t1 = exit_rx.recv().fuse();
        let t2 = func;

        pin_mut!(t1, t2);
        select! {
            _ = t1 => {},
            res = t2 => res?,
        }

        Ok(())
    }


    pub fn run_service_until_exit<T, G, E, F>(
        mut config: Configuration<G, E>,
        exit_rx: Receiver<usize>,
        service_builder: F,
    ) -> Result<(), String>
    where
        F: FnOnce(Configuration<G, E>) -> Result<T, sc_service::error::Error>,
        T: AbstractService + Unpin,
        {
            let mut runtime = build_runtime().map_err(|e| e.to_string())?;

            config.task_executor = {
                let runtime_handle = runtime.handle().clone();
                Some(Arc::new(move |fut| { runtime_handle.spawn(fut); }))
            };

            let service = service_builder(config).map_err(|e| e.to_string())?;

            // let informant_future = sc_informant::build(&service, sc_informant::OutputFormat::Coloured);
            // let _informant_handle = runtime.spawn(informant_future);

            // we eagerly drop the service so that the internal exit future is fired,
            // but we need to keep holding a reference to the global telemetry guard
            // and drop the runtime first.
            let _telemetry = service.telemetry();

            let f = service.fuse();
            pin_mut!(f);

            runtime.block_on(main(f, exit_rx)).map_err(|e| e.to_string())?;
            drop(runtime);

            Ok(())
        }

    fn init()  -> Arc<Node> {
        // let mut node_weak = NODE.lock().unwrap();
        let node_weak = Weak::<Node>::new();
        let node = match node_weak.upgrade() {
            Some(node) => node,
            None => {
                let (exit_tx, exit_rx) = mpsc::channel(10);
                let (client_tx, client_rx) = std::sync::mpsc::channel();

                let _node_main_thread = thread::spawn( move || {
                    let version = sc_cli::VersionInfo {
                        name: "Polymesh Node",
                        commit: env!("VERGEN_SHA_SHORT"),
                        version: env!("CARGO_PKG_VERSION"),
                        executable_name: "polymesh",
                        author: "Anonymous",
                        description: "Polymesh Node",
                        support_url: "https://polymath.network/",
                        copyright_start_year: 2017,
                    };

                    let mut config = sc_service::Configuration::<polymesh_runtime_testnet_v1::config::GenesisConfig>::from_version(&version);

                    let opt = sc_cli::from_iter::<Cli, _>(vec!["--dev"], &version);
                    opt.run.init(&version).unwrap();
                    opt.run.update_config(&mut config, load_spec, &version)
                        .expect("Update config failed");


                    run_service_until_exit(config, exit_rx, |config| {
                        let service = service::new_full::<
                            service::polymesh_runtime_develop::RuntimeApi,
                            service::GeneralExecutor,
                            service::polymesh_runtime_develop::UncheckedExtrinsic,
                            >(config).unwrap();

                        let _ = client_tx.send( service.client().clone());

                        Ok(service)
                    })
                });

                let client = client_rx.recv().unwrap();

                Arc::new(Node{
                    client,
                    // service: box(service),
                    // thread: Some(child)
                    exit_tx
                })
            }
        };

        // *node_weak = Arc::downgrade(&node);
        node
    }


    use pallet_identity_rpc::IdentityApi;

    #[test]
    fn it_1() -> Result<(), u8> {
        let node_guard = init();

        let rpc_identity = pallet_identity_rpc::Identity::<PolymeshClient<PolymeshBlock>, PolymeshBlock>::new( node_guard.client.clone()) ;
        let alice_id = IdentityId::from(2u128);
        let alice_info = rpc_identity.get_did_records( alice_id, None);

        let _alice_info = alice_info.unwrap();
        // assert_ok!(alice_info);


        Ok(())
    }

    #[test]
    fn it_2() -> Result<(), u8> {
        // let _node_guard = init();

        Ok(())
    }
}
