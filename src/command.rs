// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::path::PathBuf;
use std::sync::Arc;

use core::future::Future;
use frame_benchmarking_cli::*;
use log::info;
use polymesh_worker::backend::Backends;
use polymesh_worker::BackendKind;
use sc_cli::SubstrateCli;
use sc_service::{Configuration, TaskManager};
use sp_keyring::Sr25519Keyring;
use sp_runtime::traits::HashingFor;

use polymesh_primitives::Block;

use crate::benchmarking::{inherent_benchmark_data, RemarkBuilder, TransferBuilder};
use crate::chain_spec::ci_runtime::ci_chain_spec;
use crate::chain_spec::common::{ChainSpec as GenericChainSpec, ChainSpecMode};
use crate::chain_spec::develop_runtime::develop_chain_spec;
use crate::chain_spec::mainnet_runtime::mainnet_chain_spec;
use crate::chain_spec::testnet_runtime::testnet_chain_spec;
use crate::cli::{Cli, Subcommand};
use crate::service::{self, general_chain_ops, mainnet_chain_ops, new_partial, testnet_chain_ops};
use crate::service::{FullClient, HostFunctions, IsNetwork, Network, NewChainOps};

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Polymesh Node".into()
    }

    fn impl_version() -> String {
        env!("SUBSTRATE_CLI_IMPL_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/PolymeshAssociation/Polymesh/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn executable_name() -> String {
        "polymesh".into()
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "" => {
                return Err(
                    "Please specify which chain you want to run, e.g. --dev or --chain=local"
                        .into(),
                );
            }
            "dev" => {
                if cfg!(feature = "ci-runtime") {
                    Box::new(ci_chain_spec(ChainSpecMode::Development))
                } else {
                    Box::new(develop_chain_spec(ChainSpecMode::Development))
                }
            }
            "local" => {
                if cfg!(feature = "ci-runtime") {
                    Box::new(ci_chain_spec(ChainSpecMode::Local))
                } else {
                    Box::new(develop_chain_spec(ChainSpecMode::Local))
                }
            }
            "testnet-dev" => Box::new(testnet_chain_spec(ChainSpecMode::Development)),
            "testnet-local" => Box::new(testnet_chain_spec(ChainSpecMode::Local)),
            "testnet-bootstrap" => Box::new(testnet_chain_spec(ChainSpecMode::Bootstrap)),
            "mainnet-dev" => Box::new(mainnet_chain_spec(ChainSpecMode::Development)),
            "mainnet-local" => Box::new(mainnet_chain_spec(ChainSpecMode::Local)),
            "mainnet-bootstrap" => Box::new(mainnet_chain_spec(ChainSpecMode::Bootstrap)),
            "MAINNET" | "mainnet" => Box::new(GenericChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/mainnet_raw.json")[..],
            )?),
            "TESTNET" | "testnet" => Box::new(GenericChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/testnet_raw.json")[..],
            )?),
            // STAGING network should be considered unstable and may be replaced at any time.
            "STAGING" | "staging" => Box::new(GenericChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/staging_raw.json")[..],
            )?),
            // Need to re-create devnet
            // // DEVNET network for testing P-DART Confidential assets.
            // "DEVNET" | "devnet" => Box::new(GenericChainSpec::from_json_bytes(
            //     &include_bytes!("./chain_specs/devnet_raw.json")[..],
            // )?),
            path => Box::new(GenericChainSpec::from_json_file(PathBuf::from(path))?),
        })
    }
}

/// Parse command line arguments into Polymesh-specific CLI struct and run the service.
pub fn run() -> sc_cli::Result<()> {
    run_with_args(std::env::args_os())
}

/// Parses Polymesh specific CLI arguments and run the service.
pub fn run_with_args<I>(args: I) -> sc_cli::Result<()>
where
    I: IntoIterator,
    I::Item: Into<std::ffi::OsString> + Clone,
{
    // Add the `AbortGuard` to rayon thread pool.  So panics from threads do not bring down the entire process, but instead are caught and logged, and the work request is rejected.
    rayon::ThreadPoolBuilder::new()
        .spawn_handler(|thread| {
            let mut b = std::thread::Builder::new();
            if let Some(name) = thread.name() {
                b = b.name(name.to_owned());
            }
            if let Some(stack_size) = thread.stack_size() {
                b = b.stack_size(stack_size);
            }
            b.spawn(|| {
                let _guard = sp_panic_handler::AbortGuard::force_unwind();
                thread.run()
            })?;
            Ok(())
        })
        .build_global()
        .unwrap();

    let cli = Cli::from_iter(args);

    // Determine which backends to initialize based on the environment variable, or default to all backends if the variable is not set or invalid.
    let supported_kinds = std::env::var("POLYMESH_WORKER_BACKENDS").ok().and_then(|kinds_str| {
        match BackendKind::from_str_to_kinds(&kinds_str) {
            Ok(kinds) => Some(kinds),
            Err(err) => {
                eprintln!(
                    "Failed to parse POLYMESH_WORKER_BACKENDS environment variable, using all backends: {err:?}"
                );
                None
            }
        }
    }).unwrap_or_else(|| {
        // Default to all backends.
        BackendKind::ALL_KINDS.to_vec()
    });
    // Initialize Polymesh worker backends.
    Backends::init_global_backends::<polymesh_worker_native::NativeBackend>(&supported_kinds);
    // Print the available backends after initialization.
    let backends = Backends::get_available_backends();
    println!("Polymesh Worker: Available backends: {:?}", backends);

    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run)?;

            info!(
                "Reserved nodes: {:?}",
                cli.run.network_params.reserved_nodes
            );

            runner.run_node_until_exit(|config| async move {
                service::new_full(config, cli).map_err(sc_cli::Error::Service)
            })
        }
        Some(Subcommand::ExportChainSpec(cmd)) => {
            let chain_spec = cli.load_spec(&cmd.chain)?;
            cmd.run(chain_spec)
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            let network = runner.config().chain_spec.network();

            runner.sync_run(|mut config| {
                match (cmd, network) {
                    (BenchmarkCmd::Pallet(cmd), Network::Other) => {
                        if !cfg!(feature = "runtime-benchmarks") {
                            return Err("Benchmarking wasn't enabled when building the node. \
                                  You can enable it with `--features runtime-benchmarks`."
                                .into());
                        }

                        cmd.run_with_spec::<HashingFor<Block>, HostFunctions>(Some(
                            config.chain_spec,
                        ))
                    }
                    (BenchmarkCmd::Block(cmd), Network::Other) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_develop::RuntimeApi>(&mut config)?;
                        cmd.run(partial_components.client)
                    }
                    (BenchmarkCmd::Block(cmd), Network::Testnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_testnet::RuntimeApi>(&mut config)?;
                        cmd.run(partial_components.client)
                    }
                    (BenchmarkCmd::Block(cmd), Network::Mainnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_mainnet::RuntimeApi>(&mut config)?;
                        cmd.run(partial_components.client)
                    }
                    #[cfg(not(feature = "runtime-benchmarks"))]
                    (BenchmarkCmd::Storage(_), Network::Other) => Err(
                        "Storage benchmarking can be enabled with `--features runtime-benchmarks`."
                            .into(),
                    ),
                    #[cfg(feature = "runtime-benchmarks")]
                    (BenchmarkCmd::Storage(cmd), Network::Other) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_develop::RuntimeApi>(&mut config)?;
                        let db = partial_components.backend.expose_db();
                        let storage = partial_components.backend.expose_storage();
                        let shared_trie_cache =
                            partial_components.backend.expose_shared_trie_cache();

                        cmd.run(
                            config,
                            partial_components.client,
                            db,
                            storage,
                            shared_trie_cache,
                        )
                    }
                    #[cfg(feature = "runtime-benchmarks")]
                    (BenchmarkCmd::Storage(cmd), Network::Testnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_testnet::RuntimeApi>(&mut config)?;
                        let db = partial_components.backend.expose_db();
                        let storage = partial_components.backend.expose_storage();
                        let shared_trie_cache =
                            partial_components.backend.expose_shared_trie_cache();

                        cmd.run(
                            config,
                            partial_components.client,
                            db,
                            storage,
                            shared_trie_cache,
                        )
                    }
                    #[cfg(feature = "runtime-benchmarks")]
                    (BenchmarkCmd::Storage(cmd), Network::Mainnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_mainnet::RuntimeApi>(&mut config)?;
                        let db = partial_components.backend.expose_db();
                        let storage = partial_components.backend.expose_storage();
                        let shared_trie_cache =
                            partial_components.backend.expose_shared_trie_cache();

                        cmd.run(
                            config,
                            partial_components.client,
                            db,
                            storage,
                            shared_trie_cache,
                        )
                    }
                    (BenchmarkCmd::Overhead(cmd), Network::Other) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_develop::RuntimeApi>(&mut config)?;
                        let ext_builder =
                            RemarkBuilder::<polymesh_runtime_develop::RuntimeApi>::new(
                                partial_components.client.clone(),
                            );
                        cmd.run(
                            config.chain_spec.name().into(),
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                            false,
                        )
                    }
                    (BenchmarkCmd::Overhead(cmd), Network::Testnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_testnet::RuntimeApi>(&mut config)?;
                        let ext_builder =
                            RemarkBuilder::<polymesh_runtime_testnet::RuntimeApi>::new(
                                partial_components.client.clone(),
                            );
                        cmd.run(
                            config.chain_spec.name().into(),
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                            false,
                        )
                    }
                    (BenchmarkCmd::Overhead(cmd), Network::Mainnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_mainnet::RuntimeApi>(&mut config)?;
                        let ext_builder =
                            RemarkBuilder::<polymesh_runtime_mainnet::RuntimeApi>::new(
                                partial_components.client.clone(),
                            );
                        cmd.run(
                            config.chain_spec.name().into(),
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_builder,
                            false,
                        )
                    }
                    (BenchmarkCmd::Extrinsic(cmd), Network::Other) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_develop::RuntimeApi>(&mut config)?;
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::<polymesh_runtime_develop::RuntimeApi>::new(
                                partial_components.client.clone(),
                            )),
                            Box::new(
                                TransferBuilder::<polymesh_runtime_develop::RuntimeApi>::new(
                                    partial_components.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    1,
                                ),
                            ),
                        ]);

                        cmd.run(
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_factory,
                        )
                    }
                    (BenchmarkCmd::Extrinsic(cmd), Network::Testnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_testnet::RuntimeApi>(&mut config)?;
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::<polymesh_runtime_testnet::RuntimeApi>::new(
                                partial_components.client.clone(),
                            )),
                            Box::new(
                                TransferBuilder::<polymesh_runtime_testnet::RuntimeApi>::new(
                                    partial_components.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    1,
                                ),
                            ),
                        ]);

                        cmd.run(
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_factory,
                        )
                    }
                    (BenchmarkCmd::Extrinsic(cmd), Network::Mainnet) => {
                        let partial_components =
                            new_partial::<polymesh_runtime_mainnet::RuntimeApi>(&mut config)?;
                        // Register the *Remark* and *TKA* builders.
                        let ext_factory = ExtrinsicFactory(vec![
                            Box::new(RemarkBuilder::<polymesh_runtime_mainnet::RuntimeApi>::new(
                                partial_components.client.clone(),
                            )),
                            Box::new(
                                TransferBuilder::<polymesh_runtime_mainnet::RuntimeApi>::new(
                                    partial_components.client.clone(),
                                    Sr25519Keyring::Alice.to_account_id(),
                                    1,
                                ),
                            ),
                        ]);

                        cmd.run(
                            partial_components.client,
                            inherent_benchmark_data()?,
                            Vec::new(),
                            &ext_factory,
                        )
                    }
                    (BenchmarkCmd::Machine(cmd), Network::Other) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                    (_, _) => Err("Benchmarking is only supported with the `develop` runtime.")?,
                }
            })
        }
        Some(Subcommand::Key(cmd)) => cmd.run(&cli),
        Some(Subcommand::Sign(cmd)) => cmd.run(),
        Some(Subcommand::Verify(cmd)) => cmd.run(),
        Some(Subcommand::Vanity(cmd)) => cmd.run(),
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::CheckBlock(cmd)) => async_run(
            &cli,
            cmd,
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
        ),
        Some(Subcommand::ExportBlocks(cmd)) => async_run(
            &cli,
            cmd,
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
        ),
        Some(Subcommand::ExportState(cmd)) => async_run(
            &cli,
            cmd,
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
        ),
        Some(Subcommand::ImportBlocks(cmd)) => async_run(
            &cli,
            cmd,
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
        ),
        Some(Subcommand::PurgeChain(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.database))
        }
        Some(Subcommand::Revert(cmd)) => async_run(
            &cli,
            cmd,
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    sc_consensus_grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
        ),
        Some(Subcommand::ChainInfo(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run::<Block>(&config))
        }
    }
}

fn async_run<F, G, H>(
    cli: &impl sc_cli::SubstrateCli,
    cmd: &impl sc_cli::CliConfiguration,
    testnet: impl FnOnce(
        NewChainOps<polymesh_runtime_testnet::RuntimeApi>,
        Configuration,
    ) -> sc_cli::Result<(F, TaskManager)>,
    general: impl FnOnce(
        NewChainOps<polymesh_runtime_develop::RuntimeApi>,
        Configuration,
    ) -> sc_cli::Result<(G, TaskManager)>,
    mainnet: impl FnOnce(
        NewChainOps<polymesh_runtime_mainnet::RuntimeApi>,
        Configuration,
    ) -> sc_cli::Result<(H, TaskManager)>,
) -> sc_service::Result<(), sc_cli::Error>
where
    F: Future<Output = sc_cli::Result<()>>,
    G: Future<Output = sc_cli::Result<()>>,
    H: Future<Output = sc_cli::Result<()>>,
{
    let runner = cli.create_runner(cmd)?;
    match runner.config().chain_spec.network() {
        Network::Testnet => {
            runner.async_run(|mut config| testnet(testnet_chain_ops(&mut config)?, config))
        }
        Network::Other => {
            runner.async_run(|mut config| general(general_chain_ops(&mut config)?, config))
        }
        Network::Mainnet => {
            runner.async_run(|mut config| mainnet(mainnet_chain_ops(&mut config)?, config))
        }
    }
}
