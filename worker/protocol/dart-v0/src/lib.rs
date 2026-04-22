// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh Association

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
const HEAP_SIZE: usize = 20 * 1024 * 1024;

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
const SCRATCH_SIZE: usize = 10 * 1024 * 1024;

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

use polymesh_worker_common::{
    PROTOCOL_PDART, Protocol, ProtocolError, ProtocolVersion, WorkRequest, WorkResponse,
    WorkResponseResult, WorkerSessionId,
};

#[cfg(feature = "native")]
use sp_std::vec::Vec;

pub type Did = [u8; 32];

mod verify;
pub use verify::*;

mod curve_tree;
pub use curve_tree::*;

mod asset;
pub use asset::*;

#[cfg(feature = "testing")]
mod testing;

#[cfg(not(feature = "testing"))]
mod testing {
    use codec::{Decode, Encode};

    /// The non-testing version of this enum is empty, as no requests are currently supported
    #[derive(Encode, Decode, Clone)]
    pub enum GenerateDartProofRequest {}

    /// The non-testing version of this enum is empty, as no requests are currently supported
    #[derive(Encode, Decode, Clone)]
    pub enum GenerateDartProofResponse {}
}

pub use testing::*;

use codec::{Decode, Encode};

use polymesh_dart::init::{init_params, load_params, save_params};
use polymesh_dart::{
    ACCOUNT_TREE_L, ACCOUNT_TREE_M, ASSET_TREE_L, ASSET_TREE_M, FEE_ACCOUNT_TREE_L,
    FEE_ACCOUNT_TREE_M,
    curve_tree::{
        AccountTreeConfig, AssetTreeConfig, CompressedCurveTreeRoot, FeeAccountTreeConfig,
    },
};

pub const PROTOCOL: Protocol = Protocol {
    id: PROTOCOL_PDART,
    version: ProtocolVersion::new(0, 1, 0),
};

/// Dart work request.
#[derive(Encode, Decode, Clone)]
pub enum DartWorkRequest {
    VerifyProof(VerifyDartAssetRequest),
    UpdateCurveTree(CurveTreeUpdateRequest),
    UpdateAssetState(UpdateAssetStateRequest),
    GenerateProof(GenerateDartProofRequest),
}

impl DartWorkRequest {
    fn do_execute(self) -> Result<DartWorkResponse, ProtocolError> {
        match self {
            Self::VerifyProof(req) => {
                let res = req.do_verify()?;
                Ok(DartWorkResponse::VerifyProof(res))
            }
            Self::UpdateCurveTree(req) => {
                let res = req.do_update()?;
                Ok(DartWorkResponse::UpdateCurveTree(res))
            }
            Self::UpdateAssetState(req) => {
                let res = req.do_update()?;
                Ok(DartWorkResponse::UpdateAssetState(res))
            }
            Self::GenerateProof(_req) => {
                #[cfg(feature = "testing")]
                {
                    let res = _req.do_generate()?;
                    Ok(DartWorkResponse::GenerateProof(res))
                }
                #[cfg(not(feature = "testing"))]
                Err(Error::GenerateProofFailed.into())
            }
        }
    }

    pub fn execute_work(req: &WorkRequest) -> Result<DartWorkResponse, ProtocolError> {
        let req: Self = req.decode()?;
        req.do_execute()
    }
}

#[cfg(feature = "impl_protocol")]
impl DartWorkRequest {
    pub fn execute(self) -> Result<DartWorkResponse, ProtocolError> {
        self.do_execute()
    }

    pub fn session_execute_and_wait(
        self,
        _session_id: WorkerSessionId,
    ) -> Result<DartWorkResponse, ProtocolError> {
        self.do_execute()
    }
}

#[cfg(not(feature = "impl_protocol"))]
impl DartWorkRequest {
    pub fn execute(self) -> Result<DartWorkResponse, ProtocolError> {
        use polymesh_worker_extension::*;

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

    pub fn session_execute_and_wait(
        self,
        session_id: WorkerSessionId,
    ) -> Result<DartWorkResponse, ProtocolError> {
        use polymesh_worker_extension::*;

        let req = WorkRequest::new(&self);

        match native_polymesh_worker::session_execute_request_and_wait(session_id, 0, req.0) {
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

/// Dart work response.
#[derive(Encode, Decode, Clone)]
pub enum DartWorkResponse {
    VerifyProof(VerifyDartProofResponse),
    UpdateCurveTree(CurveTreeUpdateResponse),
    UpdateAssetState(UpdateAssetStateResult),
    GenerateProof(GenerateDartProofResponse),
}

pub type AssetTreeRoot = CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;
pub type AccountTreeRoot =
    CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;
pub type FeeAccountTreeRoot =
    CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum Error {
    VerifyFailed,
    GenerateProofFailed,
    CurveTreeUpdateError,
    AssetStateError,
    UnexpectedResponse,
}

impl From<polymesh_dart::Error> for Error {
    fn from(_e: polymesh_dart::Error) -> Self {
        Self::VerifyFailed
    }
}

impl From<Error> for ProtocolError {
    fn from(err: Error) -> Self {
        let err_u8 = err as u8;
        ProtocolError::custom_error([err_u8, 0, 0])
    }
}

pub fn execute_work_request(req: &WorkRequest) -> WorkResponseResult {
    let res = DartWorkRequest::execute_work(req)?;
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
                    sp_std::vec::Vec::from_raw_parts(scratch.as_mut_ptr(), 0, scratch.len())
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
