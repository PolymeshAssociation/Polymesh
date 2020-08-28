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
use sc_executor::NativeExecutionDispatch;

#[cfg(feature = "runtime-benchmarks")]
use polymesh_runtime::runtime;

impl SubstrateCli for Cli {
    fn impl_name() -> &'static str {
        "Polymesh Node"
    }

    fn impl_version() -> &'static str {
        env!("CARGO_PKG_VERSION")
    }

    fn description() -> &'static str {
        env!("CARGO_PKG_DESCRIPTION")
    }

    fn author() -> &'static str {
        env!("CARGO_PKG_AUTHORS")
    }

    fn support_url() -> &'static str {
        "https://github.com/PolymathNetwork/polymesh/issues/new"
    }

    fn copyright_start_year() -> i32 {
        2017
    }

    fn executable_name() -> &'static str {
        "polymesh"
    }

    fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
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
                runtime.run_node(
                    service::alcyone_new_light,
                    service::alcyone_new_full,
                    service::AlcyoneExecutor::native_version().runtime_version,
                )
            } else {
                runtime.run_node(
                    service::general_new_light,
                    service::general_new_full,
                    service::GeneralExecutor::native_version().runtime_version,
                )
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
