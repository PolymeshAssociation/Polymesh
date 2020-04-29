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
mod load_chain_spec;

fn main() -> sc_cli::Result<()> {
    let version = sc_cli::VersionInfo {
        name: "Polymesh Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "polymesh",
        author: "Anonymous",
        description: "Polymesh Node",
        support_url: "https://polymath.network/",
        copyright_start_year: 2017,
    };

    command::run(std::env::args(), version)
}
