//! Polymesh CLI binary.
#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![feature(associated_type_bounds)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> Result<(), sc_cli::Error> {
    command::run()
}
