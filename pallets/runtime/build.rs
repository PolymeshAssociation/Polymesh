use wasm_builder_runner::{build_current_project_with_rustflags, WasmBuilderSource};

fn main() {
    build_current_project_with_rustflags(
        "wasm_binary.rs",
        WasmBuilderSource::Crates("1.0.8"),
        "--verbose",
    );
}
