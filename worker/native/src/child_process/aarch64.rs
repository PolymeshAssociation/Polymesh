pub(super) fn testing_module_bytes() -> Option<Vec<u8>> {
    Some(polymesh_worker::decompress_module_code(include_bytes!(
        "../../../modules/testing/v0/polymesh-worker-protocol-testing.native.aarch64.so.zst"
    ))?)
}

pub(super) fn dart_module_bytes() -> Option<Vec<u8>> {
    #[cfg(feature = "testing")]
    let library = include_bytes!(
        "../../../modules/dart/v1/polymesh-worker-protocol-dart-v1.testing.native.aarch64.so.zst"
    );
    #[cfg(not(feature = "testing"))]
    let library = include_bytes!(
        "../../../modules/dart/v1/polymesh-worker-protocol-dart-v1.native.aarch64.so.zst"
    );
    polymesh_worker::decompress_module_code(library)
}
