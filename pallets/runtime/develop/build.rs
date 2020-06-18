use wasm_builder_runner::WasmBuilder;

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    WasmBuilder::new()
        .with_current_project()
        // .with_wasm_builder_from_git( "https://github.com/paritytech/substrate.git", "8c672e107789ed10973d937ba8cac245404377e2")
        .with_wasm_builder_from_path("../../../utils/wasm-builder")
        // .with_wasm_builder_from_path("/home/miguel/project/polymath/repos/Polymesh-2/utils/wasm-builder")
        .export_heap_base()
        .import_memory()
        .build()
}
