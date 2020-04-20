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

use crate::chain_spec::load_spec;
use crate::cli::{Cli, Subcommand};
use crate::service;
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

    let mut config = sc_service::Configuration::from_version(&version);

    match opt.subcommand {
        Some(Subcommand::Base(subcommand)) => {
            subcommand.init(&version)?;
            subcommand.update_config(&mut config, load_spec, &version)?;
            subcommand.run(config, |config: _| Ok(new_full_start!(config).0))
        }
        #[cfg(feature = "runtime-benchmarks")]
        Some(Subcommand::Benchmark(cmd)) => {
            cmd.init(&version)?;
            cmd.update_config(&mut config, load_spec, &version)?;
            cmd.run::<_, _, runtime::Block, crate::service::Executor>(config)
        }
        None => {
            opt.run.init(&version)?;
            opt.run.update_config(&mut config, load_spec, &version)?;
            opt.run
                .run(config, service::new_light, service::new_full, &version)
        }
    }
}
