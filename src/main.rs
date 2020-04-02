//! Polymesh CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;
mod command;

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

    command::run(version)
}
