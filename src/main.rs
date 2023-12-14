//! Polymesh Node CLI binary.
#![warn(missing_docs)]

mod chain_spec;
#[macro_use]
mod service;
mod benchmarking;
mod cli;
mod command;

fn main() -> sc_cli::Result<()> {
    command::run()
}
