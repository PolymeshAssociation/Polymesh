// Copyright 2017-2020 Parity Technologies (UK) Ltd.
// This file is part of Substrate.

// Substrate is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.

// Substrate is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with Substrate.  If not, see <http://www.gnu.org/licenses/>.

use crate::chain_spec;
use crate::cli::{Cli, Subcommand};
use crate::service::{
    self, alcyone_chain_ops, general_chain_ops, mainnet_chain_ops, new_full_base, AlcyoneExecutor,
    GeneralExecutor, IsNetwork, MainnetExecutor, Network, NewChainOps, NewFullBase,
};
use core::future::Future;
use log::info;
use polymesh_primitives::Block;
use sc_cli::{ChainSpec, RuntimeVersion};
pub use sc_cli::{Result, SubstrateCli};
use sc_service::{config::Role, Configuration, TaskManager};

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

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn ChainSpec>, String> {
        Ok(match id {
            "dev" => Box::new(chain_spec::general::develop_config()),
            "local" => Box::new(chain_spec::general::local_config()),
            "alcyone-dev" => Box::new(chain_spec::alcyone_testnet::develop_config()),
            "alcyone-local" => Box::new(chain_spec::alcyone_testnet::local_config()),
            "mainnet-dev" => Box::new(chain_spec::polymesh_mainnet::develop_config()),
            "mainnet-local" => Box::new(chain_spec::polymesh_mainnet::local_config()),
            "mainnet-bootstrap" => Box::new(chain_spec::polymesh_mainnet::bootstrap_config()),
            "Mainnet" | "mainnet" | "" => {
                Box::new(chain_spec::polymesh_mainnet::ChainSpec::from_json_bytes(
                    &include_bytes!("./chain_specs/mainnet_raw.json")[..],
                )?)
            }
            "Buffron" | "buffron" => {
                Box::new(chain_spec::alcyone_testnet::ChainSpec::from_json_bytes(
                    &include_bytes!("./chain_specs/buffron_raw.json")[..],
                )?)
            }
            "Alcyone" | "alcyone" => {
                Box::new(chain_spec::alcyone_testnet::ChainSpec::from_json_bytes(
                    &include_bytes!("./chain_specs/alcyone_raw.json")[..],
                )?)
            }
            path => Box::new(chain_spec::polymesh_mainnet::ChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        match chain_spec.network() {
            Network::Mainnet => &polymesh_runtime_mainnet::runtime::VERSION,
            Network::Testnet => &polymesh_runtime_testnet::runtime::VERSION,
            Network::Other => &polymesh_runtime_develop::runtime::VERSION,
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
            let runtime = cli.create_runner(&cli.run.base)?;
            let chain_spec = &runtime.config().chain_spec;

            //let authority_discovery_enabled = cli.run.authority_discovery_enabled;
            info!(
                "Reserved nodes: {:?}",
                cli.run.base.network_params.reserved_nodes
            );

            let network = chain_spec.network();
            runtime.run_node_until_exit(|config| match (network, &config.role) {
                (Network::Mainnet, Role::Light) => service::mainnet_new_light(config),
                (Network::Mainnet, _) => service::mainnet_new_full(config),
                (Network::Testnet, Role::Light) => service::alcyone_new_light(config),
                (Network::Testnet, _) => service::alcyone_new_full(config),
                (Network::Other, Role::Light) => service::general_new_light(config),
                (Network::Other, _) => service::general_new_full(config),
            })
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::BuildSyncSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            match runner.config().chain_spec.network() {
                Network::Mainnet => {
                    runner.async_run(|config| {
                        let chain_spec = config.chain_spec.cloned_box();
                        let network_config = config.network.clone();
                        let NewFullBase {
                            task_manager,
                            client,
                            network_status_sinks,
                            ..
                        } = new_full_base::<
                            polymesh_runtime_mainnet::RuntimeApi,
                            MainnetExecutor,
                            _,
                            _,
                        >(config, |_, _| ())?;
                        Ok((
                            cmd.run(chain_spec, network_config, client, network_status_sinks),
                            task_manager,
                        ))
                    })
                }
                Network::Testnet => {
                    runner.async_run(|config| {
                        let chain_spec = config.chain_spec.cloned_box();
                        let network_config = config.network.clone();
                        let NewFullBase {
                            task_manager,
                            client,
                            network_status_sinks,
                            ..
                        } = new_full_base::<
                            polymesh_runtime_testnet::RuntimeApi,
                            AlcyoneExecutor,
                            _,
                            _,
                        >(config, |_, _| ())?;
                        Ok((
                            cmd.run(chain_spec, network_config, client, network_status_sinks),
                            task_manager,
                        ))
                    })
                }
                Network::Other => {
                    runner.async_run(|config| {
                        let chain_spec = config.chain_spec.cloned_box();
                        let network_config = config.network.clone();
                        let NewFullBase {
                            task_manager,
                            client,
                            network_status_sinks,
                            ..
                        } = new_full_base::<
                            polymesh_runtime_develop::RuntimeApi,
                            GeneralExecutor,
                            _,
                            _,
                        >(config, |_, _| ())?;
                        Ok((
                            cmd.run(chain_spec, network_config, client, network_status_sinks),
                            task_manager,
                        ))
                    })
                }
            }
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
            |(c, b, _, tm), _| Ok((cmd.run(c, b), tm)),
            |(c, b, _, tm), _| Ok((cmd.run(c, b), tm)),
            |(c, b, _, tm), _| Ok((cmd.run(c, b), tm)),
        ),
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;
                let chain_spec = &runner.config().chain_spec;

                match chain_spec.network() {
                    Network::Mainnet => {
                        runner.sync_run(|config| cmd.run::<Block, service::MainnetExecutor>(config))
                    }
                    Network::Testnet => {
                        runner.sync_run(|config| cmd.run::<Block, service::AlcyoneExecutor>(config))
                    }
                    Network::Other => {
                        runner.sync_run(|config| cmd.run::<Block, service::GeneralExecutor>(config))
                    }
                }
            } else {
                Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                    .into())
            }
        }
    }
}

fn async_run<F, G, H>(
    cli: &impl sc_cli::SubstrateCli,
    cmd: &impl sc_cli::CliConfiguration,
    mainnet: impl FnOnce(
        NewChainOps<polymesh_runtime_mainnet::RuntimeApi, MainnetExecutor>,
        Configuration,
    ) -> Result<(F, TaskManager)>,
    alcyone: impl FnOnce(
        NewChainOps<polymesh_runtime_testnet::RuntimeApi, AlcyoneExecutor>,
        Configuration,
    ) -> Result<(G, TaskManager)>,
    general: impl FnOnce(
        NewChainOps<polymesh_runtime_develop::RuntimeApi, GeneralExecutor>,
        Configuration,
    ) -> Result<(H, TaskManager)>,
) -> sc_service::Result<(), sc_cli::Error>
where
    F: Future<Output = Result<()>>,
    G: Future<Output = Result<()>>,
    H: Future<Output = Result<()>>,
{
    let runner = cli.create_runner(cmd)?;
    match runner.config().chain_spec.network() {
        Network::Mainnet => {
            runner.async_run(|mut config| mainnet(mainnet_chain_ops(&mut config)?, config))
        }
        Network::Testnet => {
            runner.async_run(|mut config| alcyone(alcyone_chain_ops(&mut config)?, config))
        }
        Network::Other => {
            runner.async_run(|mut config| general(general_chain_ops(&mut config)?, config))
        }
    }
}
