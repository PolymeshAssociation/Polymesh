// This file is part of Substrate.

// Copyright (C) 2017-2021 Parity Technologies (UK) Ltd.
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

use crate::chain_spec;
use crate::cli::{Cli, Subcommand};
use crate::service::{
    self, ci_chain_ops, general_chain_ops, mainnet_chain_ops, new_partial, testnet_chain_ops,
    CIExecutor, FullClient, FullServiceComponents, GeneralExecutor, IsNetwork, MainnetExecutor,
    Network, NewChainOps, TestnetExecutor,
};
use frame_benchmarking_cli::*;
use sc_cli::{ChainSpec, Result, RuntimeVersion, SubstrateCli};
use sc_service::{Configuration, TaskManager};

use core::future::Future;
use log::info;
use polymesh_primitives::Block;
use std::sync::Arc;

impl SubstrateCli for Cli {
    fn impl_name() -> String {
        "Polymesh Node".into()
    }

    fn impl_version() -> String {
        env!("CARGO_PKG_VERSION").into()
    }

    fn description() -> String {
        env!("CARGO_PKG_DESCRIPTION").into()
    }

    fn author() -> String {
        env!("CARGO_PKG_AUTHORS").into()
    }

    fn support_url() -> String {
        "https://github.com/PolymathNetwork/polymesh/issues/new".into()
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn executable_name() -> String {
        "polymesh".into()
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::general::develop_config()),
            "local" => Box::new(chain_spec::general::local_config()),
            "ci-dev" => Box::new(chain_spec::ci::develop_config()),
            "ci-local" => Box::new(chain_spec::ci::local_config()),
            "testnet-dev" => Box::new(chain_spec::testnet::develop_config()),
            "testnet-local" => Box::new(chain_spec::testnet::local_config()),
            "testnet-bootstrap" => Box::new(chain_spec::testnet::bootstrap_config()),
            "mainnet-dev" => Box::new(chain_spec::mainnet::develop_config()),
            "mainnet-local" => Box::new(chain_spec::mainnet::local_config()),
            "mainnet-bootstrap" => Box::new(chain_spec::mainnet::bootstrap_config()),
            "MAINNET" | "mainnet" => Box::new(chain_spec::mainnet::ChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/mainnet_raw.json")[..],
            )?),
            "TESTNET" | "testnet" => Box::new(chain_spec::mainnet::ChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/testnet_raw.json")[..],
            )?),
            path => Box::new(chain_spec::mainnet::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match chain_spec.network() {
            Network::Testnet => &polymesh_runtime_testnet::runtime::VERSION,
            Network::Other => &polymesh_runtime_develop::runtime::VERSION,
            Network::CI => &polymesh_runtime_ci::runtime::VERSION,
            Network::Mainnet => &polymesh_runtime_mainnet::runtime::VERSION,
        }
    }
}

/// Parses Polymesh specific CLI arguments and run the service.
pub fn run() -> Result<()> {
    let mut cli = Cli::from_args();
    if cli.run.operator {
        cli.run.base.validator = true;
    }
    match &cli.subcommand {
        None => {
            let runner = cli.create_runner(&cli.run.base)?;
            let network = runner.config().chain_spec.network();

            //let authority_discovery_enabled = cli.run.authority_discovery_enabled;
            info!(
                "Reserved nodes: {:?}",
                cli.run.base.network_params.reserved_nodes
            );

            runner.run_node_until_exit(|config| async move {
                match network {
                    Network::Testnet => service::testnet_new_full(config),
                    Network::Other => service::general_new_full(config),
                    Network::CI => service::ci_new_full(config),
                    Network::Mainnet => service::mainnet_new_full(config),
                }
                .map_err(sc_cli::Error::Service)
            })
        }
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
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
        ),
        Some(Subcommand::ExportBlocks(cmd)) => async_run(
            &cli,
            cmd,
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
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
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
        ),
        Some(Subcommand::ImportBlocks(cmd)) => async_run(
            &cli,
            cmd,
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
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
                let aux_revert = Box::new(|client: Arc<FullClient<_, _>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_, _>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_, _>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
            |(c, b, _, tm), _| {
                let aux_revert = Box::new(|client: Arc<FullClient<_, _>>, backend, blocks| {
                    sc_consensus_babe::revert(client.clone(), backend, blocks)?;
                    grandpa::revert(client, blocks)?;
                    Ok(())
                });
                Ok((cmd.run(c, b, Some(aux_revert)), tm))
            },
        ),
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

                        cmd.run::<Block, service::GeneralExecutor>(config)
                    }
                    (BenchmarkCmd::Block(cmd), Network::Other) => {
                        let FullServiceComponents { client, .. } = new_partial::<
                            polymesh_runtime_develop::RuntimeApi,
                            GeneralExecutor,
                            _,
                        >(
                            &mut config
                        )?;
                        cmd.run(client)
                    }
                    (BenchmarkCmd::Storage(cmd), Network::Other) => {
                        let FullServiceComponents {
                            client, backend, ..
                        } = new_partial::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _>(
                            &mut config,
                        )?;
                        let db = backend.expose_db();
                        let storage = backend.expose_storage();

                        cmd.run(config, client, db, storage)
                    }
                    (BenchmarkCmd::Overhead(_cmd), Network::Other) => {
                        unimplemented!();
                        /*
                                    let FullServiceComponents { client, .. } = new_partial::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _>(&mut config)?;
                                    let ext_builder = BenchmarkExtrinsicBuilder::new(client.clone());

                        cmd.run(config, client, inherent_benchmark_data()?, Arc::new(ext_builder))
                        */
                    }
                    (BenchmarkCmd::Machine(cmd), Network::Other) => {
                        cmd.run(&config, SUBSTRATE_REFERENCE_HARDWARE.clone())
                    }
                    (_, _) => Err("Benchmarking is only supported with the `develop` runtime.")?,
                }
            })
        }
    }
}

fn async_run<F, G, H, I>(
    cli: &impl sc_cli::SubstrateCli,
    cmd: &impl sc_cli::CliConfiguration,
    testnet: impl FnOnce(
        NewChainOps<polymesh_runtime_testnet::RuntimeApi, TestnetExecutor>,
        Configuration,
    ) -> sc_cli::Result<(F, TaskManager)>,
    general: impl FnOnce(
        NewChainOps<polymesh_runtime_develop::RuntimeApi, GeneralExecutor>,
        Configuration,
    ) -> sc_cli::Result<(G, TaskManager)>,
    ci: impl FnOnce(
        NewChainOps<polymesh_runtime_ci::RuntimeApi, CIExecutor>,
        Configuration,
    ) -> sc_cli::Result<(H, TaskManager)>,
    mainnet: impl FnOnce(
        NewChainOps<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor>,
        Configuration,
    ) -> sc_cli::Result<(I, TaskManager)>,
) -> sc_service::Result<(), sc_cli::Error>
where
    F: Future<Output = sc_cli::Result<()>>,
    G: Future<Output = sc_cli::Result<()>>,
    H: Future<Output = sc_cli::Result<()>>,
    I: Future<Output = sc_cli::Result<()>>,
{
    let runner = cli.create_runner(cmd)?;
    match runner.config().chain_spec.network() {
        Network::Testnet => {
            runner.async_run(|mut config| testnet(testnet_chain_ops(&mut config)?, config))
        }
        Network::Other => {
            runner.async_run(|mut config| general(general_chain_ops(&mut config)?, config))
        }
        Network::CI => runner.async_run(|mut config| ci(ci_chain_ops(&mut config)?, config)),
        Network::Mainnet => {
            runner.async_run(|mut config| mainnet(mainnet_chain_ops(&mut config)?, config))
        }
    }
}
