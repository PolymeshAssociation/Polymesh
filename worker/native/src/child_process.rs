use polymesh_worker::backend::BackendModule;

pub use polymesh_worker::native_process::run_child_if_requested;

fn process_module(library: Option<Vec<u8>>) -> Option<Box<dyn BackendModule>> {
    Some(Box::new(
        polymesh_worker::native_process::NativeProcessModule::new(library?),
    ))
}

pub fn dart_module() -> Option<Box<dyn BackendModule>> {
    process_module(polymesh_worker::decompress_module_code(include_bytes!(
        concat!(env!("OUT_DIR"), "/polymesh-worker-protocol-dart-v1.so.zst")
    )))
}

pub fn testing_module() -> Option<Box<dyn BackendModule>> {
    process_module(polymesh_worker::decompress_module_code(include_bytes!(
        concat!(env!("OUT_DIR"), "/polymesh-worker-protocol-testing.so.zst")
    )))
}

#[cfg(test)]
mod tests {
    #[test]
    fn embedded_native_modules_match_target() {
        let machine: u16 = if cfg!(target_arch = "aarch64") {
            183
        } else {
            62
        };
        for compressed in [
            include_bytes!(concat!(
                env!("OUT_DIR"),
                "/polymesh-worker-protocol-dart-v1.so.zst"
            ))
            .as_slice(),
            include_bytes!(concat!(
                env!("OUT_DIR"),
                "/polymesh-worker-protocol-testing.so.zst"
            ))
            .as_slice(),
        ] {
            let bytes = polymesh_worker::decompress_module_code(compressed).unwrap();
            assert_eq!(&bytes[..4], b"\x7fELF");
            assert_eq!(bytes[4], 2);
            assert_eq!(bytes[5], 1);
            assert_eq!(u16::from_le_bytes([bytes[16], bytes[17]]), 3);
            assert_eq!(u16::from_le_bytes([bytes[18], bytes[19]]), machine);
        }
    }
}
