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

    use std::{ thread, sync::{Arc, Weak}};
    use sc_service::{ AbstractService, Configuration };
    use tokio::sync::{ mpsc::{self, Receiver, Sender } };
    use futures::{future::{ self, Future, FutureExt, TryFutureExt }, select, pin_mut, };

    use polymesh_runtime_common::{ BlockHashCount };
    use polymesh_runtime_develop::{ Runtime };
    use pallet_asset as asset;
    use polymesh_primitives::{ IdentityId, AccountKey, Ticker, Index };

    use test_client::AccountKeyring;
    use pallet_identity_rpc_runtime_api::DidRecords as RpcDidRecords;
    use frame_system_rpc_runtime_api::runtime_decl_for_AccountNonceApi::AccountNonceApi;
    use frame_system::offchain::CreateTransaction;
    use sp_runtime::{
        MultiSigner, MultiSignature,
        traits::{ Extrinsic, Block as BlockTT, Verify },
        generic::BlockId,
        testing::TestXt,
    };
    use codec::Encode;
    use frame_support::{ debug };

    use sp_core::sr25519::Signature;
    use sp_core::H256;
    use sp_arithmetic::traits::SaturatedConversion;

    use sp_application_crypto::RuntimePublic;
    use sp_runtime::traits::StaticLookup;
    // use sp_transaction_pool::TransactionSource;

    use std::convert::TryFrom;

    pub type Indices = pallet_indices::Module<Runtime>;
    pub type System = frame_system::Module<Runtime>;

    type SignedExtra = ();
    type SignedPayload = sp_runtime::generic::SignedPayload<polymesh_runtime_develop::runtime::Call, SignedExtra>;


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
        // transaction_pool: Arc<PolymeshTransactionPool<PolymeshBlock>>,
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
                let (transaction_pool_tx, transaction_pool_rx) = std::sync::mpsc::channel();

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
                        let _ = transaction_pool_tx.send( service.transaction_pool().clone());

                        Ok(service)
                    })
                });

                let client = client_rx.recv().unwrap();
                let transaction_pool = transaction_pool_rx.recv().unwrap();

                Arc::new(Node{
                    client,
                    // transaction_pool,
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
        let alice_info = rpc_identity.get_did_records( alice_id, None).unwrap();

        match alice_info {
            RpcDidRecords::Success{ master_key, signing_items } => {
                let expected_master_key = AccountKey::from([120, 146, 52, 0, 78, 45, 225, 240, 2, 28, 95, 151, 117, 76, 229, 243, 216, 140, 121, 42, 6, 204, 168, 233, 102, 231, 87, 97, 81, 153, 13, 41]);

                assert_eq!( master_key, expected_master_key);
            },
            _ => panic!( "Invalid DID")
        }

        Ok(())
    }

    // type Extrinsic = TestXt<Call<Runtime>, ()>;
    // type Signer = <MultiSignature as Verify>::Signer;
    type TSigner = <Signature as Verify>::Signer;
    // type Signature = Signature;

    #[test]
    fn it_2() -> Result<(), String> {
        let node_guard = init();
        let client = node_guard.client.clone();

        // fn sign_and_submit(call: impl Into<Call>, public: PublicOf<T, Call, Self>) -> Result<(), ()> {
        let ticker = Ticker::try_from( &b"4242"[..]).map_err(|e| e.to_string())?;
        let call = polymesh_runtime_develop::runtime::Call::Asset( asset::Call::register_ticker( ticker));

        let id = AccountKeyring::Alice.to_account_id();
        // let pub_key = MultiSigner::Sr25519(AccountKeyring::Alice.public());
        let alice = AccountKeyring::Alice;
        // let mut account = Account::<Runtime>::get(&id);
        // let nonce = Runtime::account_nonce(id.clone());
        let nonce :Index = 0;

        // Runtime::CreateTransaction
        /*let (call, signature_data) = Runtime::create_transaction::<Signer>(call, pub_key, id.clone(), nonce)
            .ok_or(())
            .map_err(|_| "Create transaction error")?;
        let (address, signature, extra) = signature_data;*/
        let (call, signature_data) = {
		// take the biggest period possible.
		let extra: SignedExtra = SignedExtra::default();
		let raw_payload = SignedPayload::new(call, extra)
                    .map_err(|e| {
			debug::warn!("Unable to create signed payload: {:?}", e);
                    "Transaction validity error".to_string()
		})?;
		// let signature = TSigner::sign(pub_key, &raw_payload)?;
                let signature = alice.sign( &raw_payload.encode());
		let address = Indices::unlookup(id);
		let (call, extra, _) = raw_payload.deconstruct();

		(call, (address, signature, extra))
        };

        // increment the nonce. This is fine, since the code should always
        // be running in off-chain context, so we NEVER persists data.
        // account.nonce += One::one();
        // Account::<Runtime>::insert(&id, account);

        // let xt = <PolymeshBlock as BlockTT>::Extrinsic::new(call, Some(signature_data)).ok_or(())
        //     .map_err( |_| "Extrinsic new error".to_owned())?;
        // let xt = TestXt::new( call, Some((signature, extra)));
        // let xt = (call, signature_data).encode();

        /*sp_io::offchain::submit_transaction(xt)
            .map_err(|_| "Submit transaction error".to_owned())
        */
        /*let tp = node_guard.transaction_pool.clone();
        let at = BlockId::number(0);
        let result_fut = tp.submit_one(&at, TransactionSource::External, xt);*/
        Ok(())
    }

    use hyper::rt;
    use node_primitives::Hash;
    use sc_rpc::author::{
        AuthorClient,
        hash::ExtrinsicOrHash,
    };
    use jsonrpc_core_client::{
        transports::http,
        RpcError,
    };

    fn it_3() {
        let node_guard = init();

        rt::run(rt::lazy(async || {
            let uri = "http://localhost:9933";

            let conn = http::connect(uri).await;
            // conn.and_then(|client: AuthorClient<Hash, Hash>| {
                let extrinsic = b"abcd".to_vec();
                // client.submit_extrinsic( extrinsic.into());
                conn.submit_extrinsic( extrinsic.into());
            // })
            // .map_err(|e| {
                println!("Error: {:?}", e);
            // })
        }))
    }
}
