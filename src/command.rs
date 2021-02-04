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
    self, alcyone_chain_ops, general_chain_ops, new_full_base, AlcyoneExecutor, GeneralExecutor,
    IsAlcyoneNetwork, NewChainOps, NewFullBase, new_partial
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
            "dev" => Box::new(chain_spec::general_development_testnet_config()),
            "local" => Box::new(chain_spec::general_local_testnet_config()),
            "live" => Box::new(chain_spec::general_live_testnet_config()),
            "alcyone-dev" => Box::new(chain_spec::alcyone_develop_testnet_config()),
            "alcyone-local" => Box::new(chain_spec::alcyone_local_testnet_config()),
            "alcyone-live" => Box::new(chain_spec::alcyone_live_testnet_config()),
            "Buffron" | "buffron" => Box::new(chain_spec::AlcyoneChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/buffron_raw.json")[..],
            )?),
            "Alcyone" | "alcyone" | "" => Box::new(chain_spec::AlcyoneChainSpec::from_json_bytes(
                &include_bytes!("./chain_specs/alcyone_raw.json")[..],
            )?),
            path => Box::new(chain_spec::GeneralChainSpec::from_json_file(
                std::path::PathBuf::from(path),
            )?),
        })
    }

    fn native_runtime_version(chain_spec: &Box<dyn ChainSpec>) -> &'static RuntimeVersion {
        if chain_spec.is_alcyone_network() {
            &polymesh_runtime_testnet::runtime::VERSION
        } else {
            &polymesh_runtime_develop::runtime::VERSION
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

            if chain_spec.is_alcyone_network() {
                runtime.run_node_until_exit(|config| match config.role {
                    Role::Light => service::alcyone_new_light(config),
                    _ => service::alcyone_new_full(config),
                })
            } else {
                runtime.run_node_until_exit(|config| match config.role {
                    Role::Light => service::general_new_light(config),
                    _ => service::general_new_full(config),
                })
            }
        }
        Some(Subcommand::BuildSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            runner.sync_run(|config| cmd.run(config.chain_spec, config.network))
        }
        Some(Subcommand::BuildSyncSpec(cmd)) => {
            let runner = cli.create_runner(cmd)?;
            if runner.config().chain_spec.is_alcyone_network() {
                runner.async_run(|config| {
                    let chain_spec = config.chain_spec.cloned_box();
                    let network_config = config.network.clone();
                    let NewFullBase { task_manager, client, network_status_sinks, .. }
                        = new_full_base::<polymesh_runtime_testnet::RuntimeApi, AlcyoneExecutor, _, _>(config, |_, _| ())?;
                    Ok((cmd.run(chain_spec, network_config, client, network_status_sinks), task_manager))
                })
            } else {
                runner.async_run(|config| {
                    let chain_spec = config.chain_spec.cloned_box();
                    let network_config = config.network.clone();
                    let NewFullBase { task_manager, client, network_status_sinks, .. }
                        = new_full_base::<polymesh_runtime_develop::RuntimeApi, GeneralExecutor, _, _>(config, |_, _| ())?;
                    Ok((cmd.run(chain_spec, network_config, client, network_status_sinks), task_manager))
                })
            }
        }
        Some(Subcommand::CheckBlock(cmd)) => async_run(
            &cli,
            cmd,
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
            |(c, _, iq, tm), _| Ok((cmd.run(c, iq), tm)),
        ),
        Some(Subcommand::ExportBlocks(cmd)) => async_run(
            &cli,
            cmd,
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.database), tm)),
        ),
        Some(Subcommand::ExportState(cmd)) => async_run(
            &cli,
            cmd,
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
            |(c, .., tm), config| Ok((cmd.run(c, config.chain_spec), tm)),
        ),
        Some(Subcommand::ImportBlocks(cmd)) => async_run(
            &cli,
            cmd,
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
        ),
        Some(Subcommand::Benchmark(cmd)) => {
            if cfg!(feature = "runtime-benchmarks") {
                let runner = cli.create_runner(cmd)?;
                let chain_spec = &runner.config().chain_spec;

                if chain_spec.is_alcyone_network() {
                    runner.sync_run(|config| cmd.run::<Block, service::AlcyoneExecutor>(config))
                } else {
                    runner.sync_run(|config| cmd.run::<Block, service::GeneralExecutor>(config))
                }
            } else {
                Err("Benchmarking wasn't enabled when building the node. \
				You can enable it with `--features runtime-benchmarks`."
                    .into())
            }
        }
        Some(Subcommand::DryRun(cmd)) => {
			let runner = cli.create_runner(cmd)?;
			runner.async_run(|config| {
				let PartialComponents { task_manager, .. } = new_partial(&config)?;
				Ok((cmd.run::<Block, service::AlcyoneExecutor>(config), task_manager))
			})
		}
    }
}

fn async_run<F, G>(
    cli: &impl sc_cli::SubstrateCli,
    cmd: &impl sc_cli::CliConfiguration,
    alcyone: impl FnOnce(
        NewChainOps<polymesh_runtime_testnet::RuntimeApi, AlcyoneExecutor>,
        Configuration,
    ) -> Result<(F, TaskManager)>,
    general: impl FnOnce(
        NewChainOps<polymesh_runtime_develop::RuntimeApi, GeneralExecutor>,
        Configuration,
    ) -> Result<(G, TaskManager)>,
) -> sc_service::Result<(), sc_cli::Error>
where
    F: Future<Output = Result<()>>,
    G: Future<Output = Result<()>>,
{
    let runner = cli.create_runner(cmd)?;
    if runner.config().chain_spec.is_alcyone_network() {
        runner.async_run(|mut config| alcyone(alcyone_chain_ops(&mut config)?, config))
    } else {
        runner.async_run(|mut config| general(general_chain_ops(&mut config)?, config))
    }
}
