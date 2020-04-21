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
use crate::cli::Cli;
use crate::service;
use sc_cli::SubstrateCli;

impl SubstrateCli for Cli {
	fn impl_name() -> &'static str {
		"Polymesh Node"
	}

	fn impl_version() -> &'static str {
		env!("SUBSTRATE_CLI_IMPL_VERSION")
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
		2020 // TODO: Can be decided by the business dev
	}

	fn executable_name() -> &'static str {
		env!("CARGO_PKG_NAME")
	}

	fn load_spec(&self, id: &str) -> std::result::Result<Box<dyn sc_service::ChainSpec>, String> {
		Ok(match id {
			"v1-dev" => Box::new(service::chain_spec::v1_develop_testnet_config()),
			"v1-local" => Box::new(service::chain_spec::v1_local_testnet_config()),
			"v1-live" => Box::new(service::chain_spec::v1_live_testnet_config()),
			"dev" | "general-dev" => Box::new(service::chain_spec::general_development_testnet_config()),
			"general-local" => Box::new(service::chain_spec::general_local_testnet_config()),
			"general-live" => Box::new(service::chain_spec::general_live_testnet_config()),
			path if self.run.force_kusama => {
				Box::new(service::V1ChainSpec::from_json_file(std::path::PathBuf::from(path))?)
			},
			path => Box::new(service::GeneralChainSpec::from_json_file(std::path::PathBuf::from(path))?),
		})
	}
}

/// Parse and run command line arguments
pub fn run() -> sc_cli::Result<()> {
	let cli = Cli::from_args();

	match &cli.subcommand {
		Some(subcommand) => {
			let runner = cli.create_runner(subcommand)?;
			runner.run_subcommand(subcommand, |config| Ok(new_full_start!(config).0))
		}
		None => {
			let runner = cli.create_runner(&cli.run)?;
			runner.run_node(
				service::new_light,
				service::new_full,
				node_template_runtime::VERSION
			)
		}
	}
}
