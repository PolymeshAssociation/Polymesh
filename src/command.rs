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

use crate::cli::{Cli, Subcommand};
use crate::load_chain_spec::{load_spec, IsV1Network};
use crate::service;
use chrono::prelude::*;
use log::info;
#[cfg(feature = "runtime-benchmarks")]
use polymesh_runtime::runtime;
use sc_cli::VersionInfo;

/// Parse command line arguments into service configuration.
pub fn run<I, T>(args: I, version: VersionInfo) -> sc_cli::Result<()>
where
    I: Iterator<Item = T>,
    T: Into<std::ffi::OsString> + Clone,
{
    let args: Vec<_> = args.collect();
    let opt = sc_cli::from_iter::<Cli, _>(args.clone(), &version);

    let mut config = sc_service::Configuration::<polymesh_runtime_testnet_v1::config::GenesisConfig>::from_version(&version);

    match opt.subcommand {
        Some(Subcommand::Base(subcommand)) => {
            subcommand.init(&version)?;
            subcommand.update_config(&mut config, load_spec, &version)?;

            let is_v1_network = config
                .chain_spec
                .as_ref()
                .map_or(false, |s| s.is_v1_network());

            if is_v1_network {
                subcommand.run(
                    config,
                    service::chain_ops::<
                        service::polymesh_runtime_testnet_v1::RuntimeApi,
                        service::V1Executor,
                        service::polymesh_runtime_testnet_v1::UncheckedExtrinsic,
                    >,
                )
            } else {
                subcommand.run(
                    config,
                    service::chain_ops::<
                        service::polymesh_runtime_develop::RuntimeApi,
                        service::GeneralExecutor,
                        service::polymesh_runtime_develop::UncheckedExtrinsic,
                    >,
                )
            }
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            cmd.init(&version)?;
            cmd.update_config(&mut config, load_spec, &version)?;
            let is_v1_network = config
                .chain_spec
                .as_ref()
                .map_or(false, |s| s.is_v1_network());
            if is_v1_network {
                cmd.run::<_, _, service::Block, service::V1Executor>(config)
            } else {
                cmd.run::<_, _, service::Block, service::GeneralExecutor>(config)
            }
        }
        None => {
            opt.run.init(&version)?;
            opt.run.update_config(&mut config, load_spec, &version)?;

            info!("{}", version.name);
            info!("  version {}", config.full_version());
            info!(
                "  by {}, {}-{}",
                version.author,
                version.copyright_start_year,
                Local::today().year()
            );
            info!("Chain specification: {}", config.expect_chain_spec().name());
            info!("Node name: {}", config.name);
            info!("Roles: {}", config.display_role());
            info!("Reserved nodes: {:?}", config.network.reserved_nodes);

            let is_v1_network = config
                .chain_spec
                .as_ref()
                .map_or(false, |s| s.is_v1_network());
            if is_v1_network {
                match config.roles {
                    service::Roles::LIGHT => sc_cli::run_service_until_exit(config, |config| {
                        service::new_light::<
                            service::polymesh_runtime_testnet_v1::RuntimeApi,
                            service::V1Executor,
                            service::polymesh_runtime_testnet_v1::UncheckedExtrinsic,
                        >(config)
                    }),
                    _ => sc_cli::run_service_until_exit(config, |config| {
                        service::new_full::<
                            service::polymesh_runtime_testnet_v1::RuntimeApi,
                            service::V1Executor,
                            service::polymesh_runtime_testnet_v1::UncheckedExtrinsic,
                        >(config)
                    }),
                }
            } else {
                match config.roles {
                    service::Roles::LIGHT => sc_cli::run_service_until_exit(config, |config| {
                        service::new_light::<
                            service::polymesh_runtime_develop::RuntimeApi,
                            service::GeneralExecutor,
                            service::polymesh_runtime_develop::UncheckedExtrinsic,
                        >(config)
                    }),
                    _ => sc_cli::run_service_until_exit(config, |config| {
                        service::new_full::<
                            service::polymesh_runtime_develop::RuntimeApi,
                            service::GeneralExecutor,
                            service::polymesh_runtime_develop::UncheckedExtrinsic,
                        >(config)
                    }),
                }
            }
        }
    }
}
