//! Host function shims for protocol modules built as native shared libraries.
//!
//! The module's `extern "C"` host imports resolve to these symbols inside the shared library,
//! which forward to the table registered by the host process.

use crate::NativeHostFunctions;

static mut HOST_FUNCTIONS: Option<NativeHostFunctions> = None;

#[allow(static_mut_refs)]
fn host() -> &'static NativeHostFunctions {
    match unsafe { HOST_FUNCTIONS.as_ref() } {
        Some(host) => host,
        None => abort(),
    }
}

fn abort() -> ! {
    unsafe extern "C" {
        fn abort() -> !;
    }
    unsafe { abort() }
}

/// Register the host functions.  Must be called before any other module export.
#[unsafe(no_mangle)]
pub unsafe extern "C" fn native_module_register_host_functions(fns: *const NativeHostFunctions) {
    unsafe {
        HOST_FUNCTIONS = Some(*fns);
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn host_logger_max_level() -> u32 {
    (host().logger_max_level)()
}

#[unsafe(no_mangle)]
pub extern "C" fn host_logger_enabled(level: u32) -> u32 {
    (host().logger_enabled)(level)
}

#[unsafe(no_mangle)]
pub extern "C" fn host_logger_log(level: u32, target_fat_ptr: u64, msg_fat_ptr: u64) {
    (host().logger_log)(level, target_fat_ptr, msg_fat_ptr)
}

#[unsafe(no_mangle)]
pub extern "C" fn host_msm_unchecked(fat_ptr: u64) -> u32 {
    (host().msm_unchecked)(fat_ptr)
}

#[unsafe(no_mangle)]
pub extern "C" fn host_batch_hash_to_curve(fat_ptr: u64) -> u32 {
    (host().batch_hash_to_curve)(fat_ptr)
}
