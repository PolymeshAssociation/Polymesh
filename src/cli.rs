#[cfg(feature = "runtime-benchmarks")]
use crate::benchmarking_cli;
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
#[derive(Clone, Debug, StructOpt)]
pub struct RunCmd {
    #[allow(missing_docs)]
    #[structopt(flatten)]
    pub base: sc_cli::RunCmd,
    /// Enable validator mode.
    ///
    /// It is an alias of the `--validator` flag. User has the choice to use either `--validator` or `--operator` flag both works same.
    #[structopt(long)]
    pub operator: bool,
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
