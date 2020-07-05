//! Polymesh CLI binary.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

#[cfg(feature = "runtime-benchmarks")]
mod analysis;
#[cfg(feature = "runtime-benchmarks")]
mod benchmarking_cli;
mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> command::Result<()> {
	command::run()
}
