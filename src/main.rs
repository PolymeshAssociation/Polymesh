//! Polymesh CLI library.

#![warn(missing_docs)]
#![warn(unused_extern_crates)]

mod chain_spec;
#[macro_use]
mod service;
mod cli;

pub use sc_cli::{error, IntoExit, VersionInfo};

fn main() {
    let version = VersionInfo {
        name: "Polymesh Node",
        commit: env!("VERGEN_SHA_SHORT"),
        version: env!("CARGO_PKG_VERSION"),
        executable_name: "polymesh",
        author: "Anonymous",
        description: "Polymesh Node",
        support_url: "https://polymath.network/",
    };

    if let Err(e) = cli::run(::std::env::args(), cli::Exit, version) {
        eprintln!("Fatal error: {}\n\n{:?}", e, e);
        std::process::exit(1)
    }
}
