pub fn build() {
    #[cfg(feature = "std")]
    {
        substrate_wasm_builder::WasmBuilder::new()
            .with_current_project()
            //.with_wasm_builder_from_git(BUILDER_REPO, BUILDER_REV)
            .export_heap_base()
            .import_memory()
            .build()
    }
}
