//! Macros shared by the protocol module crates.
//!
//! The generated items are gated on the calling crate's features: `native` (linked into the host),
//! `native_module` (shared library for the native backend), `polkavm`, `impl_protocol` and `std`.

#[doc(hidden)]
pub mod __private {
    pub use ark_std::vec::Vec;
    pub use codec::Decode;
    pub use log;
}

/// Catch a runtime fallback panic where the runtime target supports unwinding.
/// No-std Wasm targets cannot unwind; bounded requests and protocol validation
/// remain the protection there.
pub fn catch_runtime_fallback<T>(
    execute: impl FnOnce() -> Result<T, crate::ProtocolError>,
) -> Result<T, crate::ProtocolError> {
    #[cfg(feature = "std")]
    {
        std::panic::catch_unwind(std::panic::AssertUnwindSafe(execute))
            .unwrap_or(Err(crate::ProtocolError::ExecuteWorkFailed))
    }
    #[cfg(not(feature = "std"))]
    {
        execute()
    }
}

/// Module runtime: panic handler, picoalloc heap, scratch pad and the `initialize`/`execute`
/// exports used by the worker backends.
///
/// The parameter functions have the signatures `fn() -> Result<usize, E>`,
/// `fn(&[u8]) -> Result<(), E>` and `fn(&mut Vec<u8>) -> Result<usize, E>`, with `E: Into<$error>`.
#[macro_export]
macro_rules! impl_module_runtime {
    (
        heap_size: $heap_size:expr,
        scratch_size: $scratch_size:expr,
        error: $error:ty,
        init_params: $init_params:path,
        load_params: $load_params:path,
        save_params: $save_params:path,
        execute: $execute:path $(,)?
    ) => {
        #[cfg(feature = "polkavm")]
        polkavm_derive::min_stack_size!(1);
        #[cfg(feature = "polkavm")]
        polkavm_derive::min_stack_size!(1024 * 1024);
        #[cfg(feature = "polkavm")]
        polkavm_derive::min_stack_size!(2);

        #[cfg(not(any(feature = "native", feature = "std")))]
        #[panic_handler]
        fn panic(_info: &core::panic::PanicInfo) -> ! {
            #[cfg(target_family = "wasm")]
            {
                core::arch::wasm32::unreachable();
            }

            #[cfg(any(target_arch = "riscv32", target_arch = "riscv64"))]
            unsafe {
                core::arch::asm!("unimp", options(noreturn));
            }

            #[cfg(feature = "native_module")]
            picoalloc::abort();
        }

        #[cfg(not(feature = "native"))]
        const HEAP_SIZE: usize = $heap_size;

        #[cfg(not(any(feature = "native", feature = "native_module")))]
        #[global_allocator]
        static mut GLOBAL_ALLOC: picoalloc::Mutex<
            picoalloc::Allocator<picoalloc::ArrayPointer<{ HEAP_SIZE }>>,
        > = {
            static mut ARRAY: picoalloc::Array<{ HEAP_SIZE }> = picoalloc::Array([0; HEAP_SIZE]);

            picoalloc::Mutex::new(picoalloc::Allocator::new(unsafe {
                picoalloc::ArrayPointer::new(&raw mut ARRAY)
            }))
        };

        /// The heap is set by `native_module_init` before the first allocation.
        #[cfg(feature = "native_module")]
        #[global_allocator]
        static mut GLOBAL_ALLOC: picoalloc::Mutex<
            picoalloc::Allocator<picoalloc::ArrayPointer<{ HEAP_SIZE }>>,
        > = picoalloc::Mutex::new(picoalloc::Allocator::new(unsafe {
            picoalloc::ArrayPointer::new(core::ptr::null_mut())
        }));

        /// The size of the scratch pad used to hold temporary data to be passed between the host and the module.
        #[cfg(not(feature = "native"))]
        const SCRATCH_SIZE: usize = $scratch_size;

        #[cfg(not(any(feature = "native", feature = "native_module")))]
        static mut SCRATCH: picoalloc::Array<{ SCRATCH_SIZE }> =
            picoalloc::Array([0; SCRATCH_SIZE]);

        #[cfg(not(any(feature = "native", feature = "native_module")))]
        #[allow(static_mut_refs)]
        pub fn mut_scratch() -> &'static mut [u8] {
            unsafe { &mut SCRATCH.0 }
        }

        #[cfg(not(any(feature = "native", feature = "native_module")))]
        #[allow(static_mut_refs)]
        pub fn scratch() -> &'static [u8] {
            unsafe { &SCRATCH.0 }
        }

        #[cfg(feature = "native_module")]
        static mut SCRATCH_PTR: *mut u8 = core::ptr::null_mut();

        #[cfg(feature = "native_module")]
        pub fn mut_scratch() -> &'static mut [u8] {
            unsafe { core::slice::from_raw_parts_mut(SCRATCH_PTR, SCRATCH_SIZE) }
        }

        #[cfg(feature = "native_module")]
        pub fn scratch() -> &'static [u8] {
            unsafe { core::slice::from_raw_parts(SCRATCH_PTR, SCRATCH_SIZE) }
        }

        /// The heap and scratch sizes the host must provide, packed as `(heap, scratch)`.
        #[cfg(feature = "native_module")]
        #[unsafe(no_mangle)]
        pub extern "C" fn native_module_memory_sizes() -> u64 {
            $crate::pack_fat_pointer(HEAP_SIZE as u32, SCRATCH_SIZE as u32)
        }

        /// Set the heap and scratch memory.  Must be called once, before any other module export.
        #[cfg(feature = "native_module")]
        #[unsafe(no_mangle)]
        #[allow(static_mut_refs)]
        pub unsafe extern "C" fn native_module_init(heap: *mut u8, scratch: *mut u8) {
            unsafe {
                *GLOBAL_ALLOC.lock() =
                    picoalloc::Allocator::new(picoalloc::ArrayPointer::new(heap.cast()));
                SCRATCH_PTR = scratch;
            }
        }

        /// Get the scratch pad pointer.
        #[cfg(not(feature = "native"))]
        #[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
        #[unsafe(no_mangle)]
        pub extern "C" fn get_scratch_pad() -> u32 {
            scratch().as_ptr() as u32
        }

        /// Get the scratch pad size.
        #[cfg(not(feature = "native"))]
        #[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
        #[unsafe(no_mangle)]
        pub extern "C" fn get_scratch_pad_size() -> u32 {
            scratch().len() as u32
        }

        #[cfg(feature = "native")]
        pub fn initialize(load_ctx: Option<&[u8]>) -> Result<u32, $error> {
            if let Some(ctx) = load_ctx {
                $load_params(ctx)?;
                Ok(ctx.len() as u32)
            } else {
                Ok($init_params()? as u32)
            }
        }

        #[cfg(feature = "native")]
        pub fn save_context() -> Result<$crate::module::__private::Vec<u8>, $error> {
            let mut params_bytes = $crate::module::__private::Vec::new();
            $save_params(&mut params_bytes)?;
            Ok(params_bytes)
        }

        /// Initialize the module with the given parameters.
        ///
        /// If `params_len` is 0, the module will initialize the parameters and return the length of the initialized parameters.
        /// If `params_len` is greater than 0, the module will load the parameters from the scratch pad and return the length of the loaded parameters.
        /// If `save` is true, the module will save the initialized parameters to the scratch pad before returning.
        #[cfg(not(feature = "native"))]
        #[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
        #[unsafe(no_mangle)]
        pub extern "C" fn initialize(params_len: u32, save: u32) -> u64 {
            // Try to initialize the host logger.  Only the first call to `initialize` will be able to initialize the logger.
            let _ = $crate::logger::init();

            if params_len > 0 {
                let params_bytes = &scratch()[..params_len as usize];
                if $load_params(params_bytes).is_ok() {
                    // Only return the length of the parameters to indicate success.
                    return $crate::pack_fat_pointer(0, params_len) as u64;
                }
            } else if let Ok(len) = $init_params() {
                if save != 0 {
                    let mut params_bytes = unsafe {
                        let scratch = mut_scratch();
                        $crate::module::__private::Vec::from_raw_parts(
                            scratch.as_mut_ptr(),
                            0,
                            scratch.len(),
                        )
                    };
                    let len = if let Ok(len) = $save_params(&mut params_bytes) {
                        len as u32
                    } else {
                        0
                    };
                    // Prevent the Vec from deallocating the scratch memory.
                    core::mem::forget(params_bytes);
                    // Return the pointer and length of the saved parameters as a fat pointer.
                    return $crate::pack_fat_pointer(scratch().as_ptr() as u32, len) as u64;
                }
                // Only return the length of the parameters to indicate success.
                return $crate::pack_fat_pointer(0, len as u32) as u64;
            }
            0
        }

        #[cfg(not(feature = "native"))]
        #[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
        #[unsafe(no_mangle)]
        pub extern "C" fn execute(req_len: u32) -> u64 {
            let req_bytes = &scratch()[..req_len as usize];
            let req: $crate::WorkRequest =
                match $crate::module::__private::Decode::decode(&mut &req_bytes[..]) {
                    Ok(req) => req,
                    Err(_) => return 0,
                };

            // Execute the request and get the response.
            match $execute(&req) {
                Ok(res) => {
                    // Write the response to the scratch buffer and return a fat pointer to it.
                    let res_bytes = res.0;
                    let res_len = res_bytes.len();
                    let scratch = mut_scratch();
                    if scratch.len() < res_len {
                        // The response is too large to fit in the scratch buffer, return an error.
                        let err = $crate::ProtocolError::UnexpectedResponse.to_u32();
                        return $crate::pack_fat_pointer(err, u32::MAX) as u64;
                    }
                    scratch[..res_len].copy_from_slice(&res_bytes);

                    $crate::pack_fat_pointer(scratch.as_ptr() as u32, res_len as u32) as u64
                }
                Err(err) => {
                    // Return the error code as a fat pointer with `len` set to `u32::MAX` to indicate an error.
                    $crate::pack_fat_pointer(err.to_u32(), u32::MAX) as u64
                }
            }
        }

        /// A hook run just before a work request is re-executed in the runtime after host execution
        /// failed (i.e. on the fallback to runtime execution).  It is set by the runtime once per
        /// runtime call, and lives in the runtime's globals.
        #[cfg(feature = "std")]
        static RUNTIME_FALLBACK_HOOK: std::sync::OnceLock<fn()> = std::sync::OnceLock::new();

        #[cfg(not(feature = "std"))]
        static mut RUNTIME_FALLBACK_HOOK: Option<fn()> = None;

        /// Set the fallback hook.  The hook is stored in the runtime's globals, which are scoped to
        /// the current runtime call, so it must be registered once per runtime call.
        ///
        /// Available in every build so callers compile uniformly; the hook is only invoked by the
        /// builds that support the fallback to runtime execution (`not(impl_protocol)`).
        pub fn set_runtime_fallback_hook(hook: fn()) {
            #[cfg(feature = "std")]
            {
                let _ = RUNTIME_FALLBACK_HOOK.set(hook);
            }
            #[cfg(not(feature = "std"))]
            {
                unsafe {
                    RUNTIME_FALLBACK_HOOK = Some(hook);
                }
            }
        }

        /// Run the fallback hook, if one has been set.
        #[cfg(not(feature = "impl_protocol"))]
        fn run_runtime_fallback_hook() {
            #[cfg(feature = "std")]
            {
                if let Some(hook) = RUNTIME_FALLBACK_HOOK.get() {
                    hook();
                }
            }
            #[cfg(not(feature = "std"))]
            {
                if let Some(hook) = unsafe { RUNTIME_FALLBACK_HOOK } {
                    hook();
                }
            }
        }
    };
}

/// Work request/response helpers: `execute_work`, the runtime-side `execute`/`session_*` calls
/// (with fallback to runtime execution) and `execute_work_request`.
///
/// The request type must implement `fn do_execute(self) -> Result<$response, ProtocolError>`.
///
/// The calling crate must provide `fn run_runtime_fallback_hook()`, which is invoked just before
/// the request is re-executed in the runtime, i.e. only when host execution failed and the
/// fallback to runtime execution actually happens.  A no-op implementation is fine.
#[macro_export]
macro_rules! impl_work_request {
    ($request:ty, $response:ty, $protocol:expr) => {
        impl $request {
            pub fn execute_work(
                req: &$crate::WorkRequest,
            ) -> Result<$response, $crate::ProtocolError> {
                if req.0.len() > $crate::MAX_WORK_REQUEST_SIZE {
                    return Err($crate::ProtocolError::UnexpectedResponse);
                }
                let req: Self = req.decode()?;
                req.do_execute()
            }
        }

        pub fn execute_work_request(req: &$crate::WorkRequest) -> $crate::WorkResponseResult {
            let res = <$request>::execute_work(req)?;
            let response = $crate::WorkResponse::new(res);
            if response.0.len() > $crate::MAX_WORK_REQUEST_SIZE {
                return Err($crate::ProtocolError::UnexpectedResponse);
            }
            Ok(response)
        }

        #[cfg(feature = "impl_protocol")]
        impl $request {
            pub fn execute(self) -> Result<$response, $crate::ProtocolError> {
                self.do_execute()
            }

            pub fn session_execute_and_wait(
                self,
                _session_id: $crate::WorkerSessionId,
            ) -> Result<$response, $crate::ProtocolError> {
                self.do_execute()
            }
        }

        #[cfg(not(feature = "impl_protocol"))]
        impl $request {
            fn do_execute_fallback(self) -> Result<$response, $crate::ProtocolError> {
                if codec::Encode::encode(&self).len() > $crate::MAX_WORK_REQUEST_SIZE {
                    return Err($crate::ProtocolError::UnexpectedResponse);
                }
                // Run the fallback hook (e.g. to prepare runtime-side state such as the
                // verification parameters) before executing the request in the runtime.
                run_runtime_fallback_hook();
                let response = $crate::module::catch_runtime_fallback(|| self.do_execute())?;
                if codec::Encode::encode(&response).len() > $crate::MAX_WORK_REQUEST_SIZE {
                    return Err($crate::ProtocolError::UnexpectedResponse);
                }
                Ok(response)
            }

            /// Execute a work request without a session, and return the results.
            pub fn execute(self) -> Result<$response, $crate::ProtocolError> {
                let req = $crate::WorkRequest::new(&self);
                let backends = $crate::BackendKind::all_mask();

                match polymesh_worker_extension::native_polymesh_worker::execute_request(
                    ($protocol).to_number(),
                    backends,
                    req.0,
                ) {
                    Ok(Ok(resp)) => Ok(resp.decode()?),
                    // This is a protocol error (i.e. invalid proof).
                    Ok(Err(err)) => Err(err),
                    Err(err) => {
                        // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                        $crate::module::__private::log::debug!(
                            "Host failed to execute work, falling back to runtime execution: {:?}",
                            err
                        );
                        self.do_execute_fallback()
                    }
                }
            }

            /// Execute the work request in the given session, and don't wait for the results.  The results can be retrieved later using `session_get_result` with the returned request id.
            pub fn session_execute(
                self,
                session_id: $crate::WorkerSessionId,
            ) -> Result<$crate::WorkRequestId, $crate::ProtocolError> {
                let req = $crate::WorkRequest::new(&self);

                let status_flag_and_id =
                    polymesh_worker_extension::native_polymesh_worker::session_execute_request(
                        session_id,
                        $crate::WORK_CONFIG_FLAG_USE_CACHE,
                        req.0,
                    );
                let (status, _flags, request_id) =
                    $crate::unpack_work_status_flags_and_id(status_flag_and_id);

                match status {
                    $crate::WorkStatus::Pending | $crate::WorkStatus::Completed => Ok(request_id),
                    $crate::WorkStatus::ExecutionFailedFallbackToRuntime => {
                        // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                        $crate::module::__private::log::debug!(
                            "Host failed to execute work, falling back to runtime execution for session id: {}, request id: {}",
                            session_id,
                            request_id
                        );
                        let result = self.do_execute_fallback().map($crate::WorkResponse::new);

                        // Push the results back to the session, and return the request id if the push is successful.
                        $crate::WorkerError::result_from_u64(
                            polymesh_worker_extension::native_polymesh_worker::session_push_result(
                                session_id, request_id, result,
                            ),
                        )
                        .map_err(|err| {
                            $crate::module::__private::log::error!(
                                "Failed to push result for session id: {}, request id: {}, error: {:?}",
                                session_id,
                                request_id,
                                err
                            );
                            $crate::ProtocolError::ExecuteWorkFailed
                        })?;

                        Ok(request_id)
                    }
                    $crate::WorkStatus::SessionNotFound => Err($crate::ProtocolError::ExecuteWorkFailed),
                    $crate::WorkStatus::Unknown => Err($crate::ProtocolError::UnexpectedResponse),
                }
            }

            /// Execute the work request in the given session and wait for the results.
            pub fn session_execute_and_wait(
                self,
                session_id: $crate::WorkerSessionId,
            ) -> Result<$response, $crate::ProtocolError> {
                let req = $crate::WorkRequest::new(&self);

                match polymesh_worker_extension::native_polymesh_worker::session_execute_request_and_wait(
                    session_id,
                    $crate::WORK_CONFIG_FLAG_USE_CACHE,
                    req.0,
                ) {
                    Ok(Ok(resp)) => Ok(resp.decode()?),
                    // This is a protocol error (i.e. invalid proof).
                    Ok(Err(err)) => Err(err),
                    Err(err) => {
                        // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                        $crate::module::__private::log::debug!(
                            "Host failed to execute work, falling back to runtime execution: {:?}",
                            err
                        );
                        self.do_execute_fallback()
                    }
                }
            }
        }

        #[cfg(not(feature = "impl_protocol"))]
        impl $response {
            /// Get the work response results from the session and decode it into the expected response type.
            pub fn session_get_result(
                session_id: $crate::WorkerSessionId,
                request_id: $crate::WorkRequestId,
            ) -> Result<Self, $crate::ProtocolError> {
                match polymesh_worker_extension::native_polymesh_worker::session_get_result(
                    session_id, request_id,
                ) {
                    Ok(Ok(resp)) => Ok(resp.decode()?),
                    // This is a protocol error (i.e. invalid proof).
                    Ok(Err(err)) => Err(err),
                    Err(err) => {
                        $crate::module::__private::log::debug!(
                            "Host failed to get result for session id: {}, request id: {}, falling back to runtime execution: {:?}",
                            session_id,
                            request_id,
                            err
                        );
                        Err($crate::ProtocolError::UnexpectedResponse)
                    }
                }
            }
        }
    };
}
