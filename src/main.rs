//! Polymesh CLI binary.
#![feature(box_syntax)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

fn main() -> Result<(), sc_cli::Error> {
    command::run()
}
