use wasm_builder_runner::WasmBuilder;

fn main() {
    // println!("cargo:rerun-if-changed=build.rs");
    WasmBuilder::new()
        .with_current_project()
        .with_wasm_builder_from_path("../../../utils/wasm-builder")
        .export_heap_base()
        .import_memory()
        .build()
}
