#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking_cli;
use sc_cli::RunCmd;
use structopt::StructOpt;

#[derive(Clone, Debug, StructOpt)]
pub struct Cli {
    /// Possible subcommand with parameters.
    #[structopt(subcommand)]
    pub subcommand: Option<Subcommand>,
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub run: RunCmd,
}

/// Possible subcommands of the main binary.
#[derive(Clone, Debug, StructOpt)]
pub enum Subcommand {
    /// A set of base subcommands handled by `sc_cli`.
    #[structopt(flatten)]
    Base(sc_cli::Subcommand),
    /// The custom benchmark subcommmand benchmarking runtime pallets.
    #[cfg(feature = "runtime-benchmarks")]
    #[structopt(name = "benchmark", about = "Benchmark runtime pallets.")]
    Benchmark(benchmarking_cli::BenchmarkCmd),
}
