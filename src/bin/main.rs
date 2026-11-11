//! Polymesh Node CLI binary.

fn main() -> sc_cli::Result<()> {
    polymesh_worker_native::run_child_if_requested();
    polymesh::command::run()
}
