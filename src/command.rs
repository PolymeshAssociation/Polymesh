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
use crate::service;
use crate::service::IsAlcyoneNetwork;
use log::info;
use polymesh_primitives::Block;
pub use sc_cli::{Result, SubstrateCli};
use sc_service::config::Role;
use sc_cli::{ChainSpec, RuntimeVersion};

#[cfg(feature = "runtime-benchmarks")]
use polymesh_runtime::runtime;

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
                    _ => service::alcyone_new_full(config)
                })
            } else {
                runtime.run_node_until_exit(|config| match config.role {
                    Role::Light => service::general_new_light(config),
                    _ => service::general_new_full(config)
                })
            }
        }
        Some(Subcommand::Base(subcommand)) => {
            let runtime = cli.create_runner(subcommand)?;
            let chain_spec = &runtime.config().chain_spec;

            if chain_spec.is_alcyone_network() {
                runtime.run_subcommand(subcommand, |config| {
                    service::chain_ops::<
                        service::polymesh_runtime_testnet::RuntimeApi,
                        service::AlcyoneExecutor,
                        service::polymesh_runtime_testnet::UncheckedExtrinsic,
                    >(config)
                })
            } else {
                runtime.run_subcommand(subcommand, |config| {
                    service::chain_ops::<
                        service::polymesh_runtime_develop::RuntimeApi,
                        service::GeneralExecutor,
                        service::polymesh_runtime_develop::UncheckedExtrinsic,
                    >(config)
                })
            }
        }
        Some(Subcommand::Benchmark(cmd)) => {
            let runtime = cli.create_runner(cmd)?;
            let chain_spec = &runtime.config().chain_spec;

            if chain_spec.is_alcyone_network() {
                runtime.sync_run(|config| cmd.run::<Block, service::AlcyoneExecutor>(config))
            } else {
                runtime.sync_run(|config| cmd.run::<Block, service::GeneralExecutor>(config))
            }
        }
    }
}
