pub fn build() {
    #[cfg(all(feature = "std", feature = "metadata-hash"))]
    {
        substrate_wasm_builder::WasmBuilder::init_with_defaults()
            .enable_metadata_hash("POLYX", 6)
            .build();
    }

    #[cfg(all(feature = "std", not(feature = "metadata-hash")))]
    {
        substrate_wasm_builder::WasmBuilder::build_using_defaults();
    }
}
