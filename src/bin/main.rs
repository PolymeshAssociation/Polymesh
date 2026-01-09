//! Polymesh Node CLI binary.

fn main() -> sc_cli::Result<()> {
    let args = std::env::args_os()
        .map(|s| s.to_string_lossy().to_string())
        .collect::<Vec<_>>();
    polymesh::command::run_with_args(args)
}
