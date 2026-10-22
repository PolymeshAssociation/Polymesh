// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2026 Polymesh Association

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "native"), no_main)]

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
}

#[cfg(not(feature = "native"))]
const HEAP_SIZE: usize = 10 * 1024 * 1024;

#[cfg(not(feature = "native"))]
#[global_allocator]
static mut GLOBAL_ALLOC: picoalloc::Mutex<
    picoalloc::Allocator<picoalloc::ArrayPointer<{ HEAP_SIZE }>>,
> = {
    static mut ARRAY: picoalloc::Array<{ HEAP_SIZE }> = picoalloc::Array([0; HEAP_SIZE]);

    picoalloc::Mutex::new(picoalloc::Allocator::new(unsafe {
        picoalloc::ArrayPointer::new(&raw mut ARRAY)
    }))
};

/// The size of the scratch pad used to hold temporary data to be passed between the host and the module.
#[cfg(not(feature = "native"))]
const SCRATCH_SIZE: usize = 4 * 1024 * 1024;

#[cfg(not(feature = "native"))]
static mut SCRATCH: picoalloc::Array<{ SCRATCH_SIZE }> = picoalloc::Array([0; SCRATCH_SIZE]);

#[cfg(not(feature = "native"))]
#[allow(static_mut_refs)]
pub fn mut_scratch() -> &'static mut [u8] {
    unsafe { &mut SCRATCH.0 }
}

#[cfg(not(feature = "native"))]
#[allow(static_mut_refs)]
pub fn scratch() -> &'static [u8] {
    unsafe { &SCRATCH.0 }
}

#[cfg(not(feature = "native"))]
use polymesh_worker_common::pack_fat_pointer;
#[cfg(not(feature = "impl_protocol"))]
use polymesh_worker_common::{
    WORK_CONFIG_FLAG_USE_CACHE, WorkRequestId, WorkerError, unpack_work_status_flags_and_id,
};

use polymesh_worker_common::{
    PROTOCOL_TESTING, Protocol, ProtocolError, ProtocolVersion, WorkRequest, WorkResponse,
    WorkResponseResult, WorkerSessionId,
};

#[cfg(not(feature = "impl_protocol"))]
use polymesh_worker_extension::*;

use codec::{Decode, DecodeWithMemTracking, Encode, MaxEncodedLen};
use scale_info::TypeInfo;
use sp_std::{vec, vec::Vec};

const fn protocol_version() -> ProtocolVersion {
    #[cfg(feature = "version_v1")]
    {
        ProtocolVersion::new(1, 0, 0)
    }
    #[cfg(feature = "version_v2")]
    {
        ProtocolVersion::new(2, 0, 0)
    }
    #[cfg(not(any(feature = "version_v1", feature = "version_v2")))]
    {
        ProtocolVersion::new(0, 1, 0)
    }
}

pub const PROTOCOL: Protocol = Protocol {
    id: PROTOCOL_TESTING,
    version: protocol_version(),
};

/// Verify version request.
#[derive(
    Clone,
    Debug,
    Encode,
    Decode,
    PartialEq,
    Eq,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen
)]
pub struct VerifyVersionRequest {
    pub protocol: Protocol,
}

impl VerifyVersionRequest {
    pub fn do_verify(&self) -> Result<VerifyVersionResponse, ProtocolError> {
        if self.protocol != PROTOCOL {
            return Err(Error::VerifyFailed.into());
        }
        Ok(VerifyVersionResponse {})
    }
}

/// Test work request.
#[derive(
    Clone,
    Debug,
    Encode,
    Decode,
    PartialEq,
    Eq,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen
)]
pub enum TestWorkRequest {
    VerifyVersion(VerifyVersionRequest),
}

impl TestWorkRequest {
    fn do_execute(self) -> Result<TestWorkResponse, ProtocolError> {
        match self {
            Self::VerifyVersion(req) => {
                let res = req.do_verify()?;
                Ok(TestWorkResponse::VerifyVersion(res))
            }
        }
    }

    pub fn execute_work(req: &WorkRequest) -> Result<TestWorkResponse, ProtocolError> {
        let req: Self = req.decode()?;
        req.do_execute()
    }
}

#[cfg(feature = "impl_protocol")]
impl TestWorkRequest {
    pub fn execute(self) -> Result<TestWorkResponse, ProtocolError> {
        self.do_execute()
    }

    pub fn session_execute_and_wait(
        self,
        _session_id: WorkerSessionId,
    ) -> Result<TestWorkResponse, ProtocolError> {
        self.do_execute()
    }
}

#[cfg(not(feature = "impl_protocol"))]
impl TestWorkRequest {
    /// Execute a work request without a session, and return the results.
    pub fn execute(self) -> Result<TestWorkResponse, ProtocolError> {
        let req = WorkRequest::new(&self);
        let backends = BackendKind::all_mask();

        match native_polymesh_worker::execute_request(PROTOCOL.to_number(), backends, req.0) {
            Ok(Ok(resp)) => Ok(resp.decode()?),
            Ok(Err(err)) => {
                // This is a protocol error (i.e. invalid proof).
                Err(err)
            }
            Err(err) => {
                // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                log::debug!(
                    "Host failed to execute work, falling back to runtime execution: {:?}",
                    err
                );
                self.do_execute()
            }
        }
    }

    /// Execute the work request in the given session, and don't wait for the results.  The results can be retrieved later using `session_get_result` with the returned request id.
    pub fn session_execute(
        self,
        session_id: WorkerSessionId,
    ) -> Result<WorkRequestId, ProtocolError> {
        let req = WorkRequest::new(&self);

        let status_flag_and_id = native_polymesh_worker::session_execute_request(
            session_id,
            WORK_CONFIG_FLAG_USE_CACHE,
            req.0,
        );
        let (status, _flags, request_id) = unpack_work_status_flags_and_id(status_flag_and_id);

        match status {
            WorkStatus::Pending | WorkStatus::Completed => Ok(request_id),
            WorkStatus::ExecutionFailedFallbackToRuntime => {
                // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                log::debug!(
                    "Host failed to execute work, falling back to runtime execution for session id: {}, request id: {}",
                    session_id,
                    request_id
                );
                let result = self.do_execute().map(WorkResponse::new);

                // Push the results back to the session, and return the request id if the push is successful.
                WorkerError::result_from_u64(native_polymesh_worker::session_push_result(
                    session_id, request_id, result,
                ))
                .map_err(|err| {
                    log::error!(
                        "Failed to push result for session id: {}, request id: {}, error: {:?}",
                        session_id,
                        request_id,
                        err
                    );
                    ProtocolError::ExecuteWorkFailed
                })?;

                Ok(request_id)
            }
            WorkStatus::SessionNotFound => Err(ProtocolError::ExecuteWorkFailed),
            WorkStatus::Unknown => Err(ProtocolError::UnexpectedResponse),
        }
    }

    /// Execute the work request in the given session and wait for the results.
    pub fn session_execute_and_wait(
        self,
        session_id: WorkerSessionId,
    ) -> Result<TestWorkResponse, ProtocolError> {
        let req = WorkRequest::new(&self);

        match native_polymesh_worker::session_execute_request_and_wait(
            session_id,
            WORK_CONFIG_FLAG_USE_CACHE,
            req.0,
        ) {
            Ok(Ok(resp)) => Ok(resp.decode()?),
            Ok(Err(err)) => {
                // This is a protocol error (i.e. invalid proof).
                Err(err)
            }
            Err(err) => {
                // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                log::debug!(
                    "Host failed to execute work, falling back to runtime execution: {:?}",
                    err
                );
                self.do_execute()
            }
        }
    }
}

/// Verify version response.
#[derive(
    Clone,
    Debug,
    Encode,
    Decode,
    PartialEq,
    Eq,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen
)]
pub struct VerifyVersionResponse {}

/// Test work response.
#[derive(
    Clone,
    Debug,
    Encode,
    Decode,
    PartialEq,
    Eq,
    TypeInfo,
    DecodeWithMemTracking,
    MaxEncodedLen
)]
pub enum TestWorkResponse {
    VerifyVersion(VerifyVersionResponse),
}

#[cfg(not(feature = "impl_protocol"))]
impl TestWorkResponse {
    /// Get the work response results from the session and decode it into the expected response type.
    pub fn session_get_result(
        session_id: WorkerSessionId,
        request_id: WorkRequestId,
    ) -> Result<Self, ProtocolError> {
        match native_polymesh_worker::session_get_result(session_id, request_id) {
            Ok(Ok(resp)) => Ok(resp.decode()?),
            Ok(Err(err)) => {
                // This is a protocol error (i.e. invalid proof).
                Err(err)
            }
            Err(err) => {
                // Fallback to runtime execution if the host execution fails, to allow older nodes to continue syncing.
                log::debug!(
                    "Host failed to get result for session id: {}, request id: {}, falling back to runtime execution: {:?}",
                    session_id,
                    request_id,
                    err
                );
                Err(ProtocolError::UnexpectedResponse)
            }
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    VerifyFailed,
    InvalidParams,
    UnexpectedResponse,
}

impl From<Error> for ProtocolError {
    fn from(err: Error) -> Self {
        let err_u8 = err as u8;
        ProtocolError::custom_error([err_u8, 0, 0])
    }
}

pub fn execute_work_request(req: &WorkRequest) -> WorkResponseResult {
    let res = TestWorkRequest::execute_work(req)?;
    Ok(WorkResponse::new(res))
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

/// Dummy parameters.
static mut DUMMY_PARAMS: Option<Vec<u8>> = None;

const PARAMS_SIZE: usize = 10 * 1024;

fn init_params() -> Result<usize, Error> {
    // Initialize the parameters with dummy data.
    let params = vec![0u8; PARAMS_SIZE];
    unsafe {
        DUMMY_PARAMS = Some(params);
    }
    Ok(PARAMS_SIZE)
}

fn load_params(params: &[u8]) -> Result<(), Error> {
    // check that the params length is correct
    if params.len() != PARAMS_SIZE {
        return Err(Error::InvalidParams);
    }
    // Save the params to the dummy parameters.
    unsafe {
        DUMMY_PARAMS = Some(params.to_vec());
    }
    Ok(())
}

fn save_params(params: &mut Vec<u8>) -> Result<usize, Error> {
    // Save the dummy parameters to the given params vector.
    #[allow(static_mut_refs)]
    unsafe {
        if let Some(dummy_params) = &DUMMY_PARAMS {
            params.clear();
            params.extend_from_slice(dummy_params);
            Ok(dummy_params.len())
        } else {
            Err(Error::InvalidParams)
        }
    }
}

#[cfg(feature = "native")]
pub fn initialize(load_ctx: Option<&[u8]>) -> Result<u32, Error> {
    if let Some(ctx) = load_ctx {
        load_params(ctx)?;
        Ok(ctx.len() as u32)
    } else {
        Ok(init_params()? as u32)
    }
}

#[cfg(feature = "native")]
pub fn save_context() -> Result<Vec<u8>, Error> {
    let mut params_bytes = Vec::new();
    save_params(&mut params_bytes)?;
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
    let _ = polymesh_worker_common::logger::init();

    let params_len = params_len;
    if params_len > 0 {
        let params_bytes = &scratch()[..params_len as usize];
        if load_params(params_bytes).is_ok() {
            // Only return the length of the parameters to indicate success.
            return pack_fat_pointer(0, params_len as u32) as u64;
        }
    } else {
        if let Ok(len) = init_params() {
            if save != 0 {
                let mut params_bytes = unsafe {
                    let scratch = mut_scratch();
                    Vec::from_raw_parts(scratch.as_mut_ptr(), 0, scratch.len())
                };
                let len = if let Ok(len) = save_params(&mut params_bytes) {
                    len as u32
                } else {
                    0
                };
                // Prevent the Vec from deallocating the scratch memory.
                core::mem::forget(params_bytes);
                // Return the pointer and length of the saved parameters as a fat pointer.
                return pack_fat_pointer(scratch().as_ptr() as u32, len as u32) as u64;
            }
            // Only return the length of the parameters to indicate success.
            return pack_fat_pointer(0, len as u32) as u64;
        }
    }
    0
}

#[cfg(not(feature = "native"))]
#[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
#[unsafe(no_mangle)]
pub extern "C" fn execute(req_len: u32) -> u64 {
    let req_bytes = &scratch()[..req_len as usize];
    let req: WorkRequest = match Decode::decode(&mut &req_bytes[..]) {
        Ok(req) => req,
        Err(_) => return 0,
    };

    // Execute the request and get the response.
    match execute_work_request(&req) {
        Ok(res) => {
            // Write the response to the scratch buffer and return a fat pointer to it.
            let res_bytes = res.0;
            let res_len = res_bytes.len();
            let scratch = mut_scratch();
            if scratch.len() < res_len {
                // The response is too large to fit in the scratch buffer, return an error.
                let err = ProtocolError::UnexpectedResponse.to_u32();
                return pack_fat_pointer(err, u32::MAX) as u64;
            }
            scratch[..res_len].copy_from_slice(&res_bytes);

            pack_fat_pointer(scratch.as_ptr() as u32, res_len as u32) as u64
        }
        Err(err) => {
            let err_u32 = err.to_u32();

            // Return the error code as a fat pointer with `len` set to `u32::MAX` to indicate an error.
            pack_fat_pointer(err_u32, u32::MAX) as u64
        }
    }
}
