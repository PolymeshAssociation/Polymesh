use polymesh_worker::backend::BackendModule;

#[cfg(target_arch = "x86_64")]
#[path = "child_process/x86_64.rs"]
mod artifacts;
#[cfg(target_arch = "aarch64")]
#[path = "child_process/aarch64.rs"]
mod artifacts;

pub use polymesh_worker::native_process::run_child_if_requested;

fn process_module(library: Option<Vec<u8>>) -> Option<Box<dyn BackendModule>> {
    Some(Box::new(
        polymesh_worker::native_process::NativeProcessModule::new(library?),
    ))
}

pub fn dart_module() -> Option<Box<dyn BackendModule>> {
    process_module(artifacts::dart_module_bytes())
}

pub fn testing_module() -> Option<Box<dyn BackendModule>> {
    process_module(artifacts::testing_module_bytes())
}
