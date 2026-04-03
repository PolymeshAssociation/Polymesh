// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh Association

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "native"), no_main)]

#[cfg(feature = "polkavm")]
polkavm_derive::min_stack_size!(1);
#[cfg(feature = "polkavm")]
polkavm_derive::min_stack_size!(128 * 1024);
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
const SCRATCH_SIZE: usize = 2 * 1024 * 1024;

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

use polymesh_worker_protocol_common::{Error as CommonError, WorkRequest, WorkResponse};

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

/// Dart work request.
#[derive(Encode, Decode, Clone)]
pub enum DartWorkRequest {
    VerifyProof(VerifyDartAssetRequest),
    GenerateProof(GenerateDartProofRequest),
}

impl DartWorkRequest {
    pub fn execute_work(req: &WorkRequest) -> Result<DartWorkResponse, CommonError> {
        let dart_req: Self = req.decode()?;
        match dart_req {
            Self::VerifyProof(req) => {
                let res = req.verify()?;
                Ok(DartWorkResponse::VerifyProof(res))
            }
            Self::GenerateProof(_req) => {
                #[cfg(feature = "testing")]
                {
                    let res = _req.generate()?;
                    Ok(DartWorkResponse::GenerateProof(res))
                }
                #[cfg(not(feature = "testing"))]
                Err(Error::GenerateProofFailed.into())
            }
        }
    }
}

/// Dart work response.
#[derive(Encode, Decode, Clone)]
pub enum DartWorkResponse {
    VerifyProof(VerifyDartProofResponse),
    GenerateProof(GenerateDartProofResponse),
}

pub type AssetTreeRoot = CompressedCurveTreeRoot<ASSET_TREE_L, ASSET_TREE_M, AssetTreeConfig>;
pub type AccountTreeRoot =
    CompressedCurveTreeRoot<ACCOUNT_TREE_L, ACCOUNT_TREE_M, AccountTreeConfig>;
pub type FeeAccountTreeRoot =
    CompressedCurveTreeRoot<FEE_ACCOUNT_TREE_L, FEE_ACCOUNT_TREE_M, FeeAccountTreeConfig>;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    VerifyFailed,
    GenerateProofFailed,
    DecodingFailed,
    CurveTreeUpdateError,
    AssetStateError,
    InvalidWorkResult,
}

impl From<polymesh_dart::Error> for Error {
    fn from(_e: polymesh_dart::Error) -> Self {
        Self::VerifyFailed
    }
}

impl From<Error> for CommonError {
    fn from(e: Error) -> Self {
        CommonError::protocol_error(e)
    }
}

pub fn execute_work_request(req: &WorkRequest) -> WorkResponse {
    match DartWorkRequest::execute_work(req) {
        Ok(res) => WorkResponse::new(res),
        Err(err) => WorkResponse::Error(err.into()),
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
pub fn initialize() -> Result<(), Error> {
    // Initialize the parameters on native by calling init_params directly, as we don't have a scratch pad to pass data through.
    init_params()?;
    Ok(())
}

/// Initialize the module with the given parameters.
///
/// If `params_len` is 0, the module will initialize the parameters and return the length of the initialized parameters.
/// If `params_len` is greater than 0, the module will load the parameters from the scratch pad and return the length of the loaded parameters.
/// If `save` is true, the module will save the initialized parameters to the scratch pad before returning.
#[cfg(not(feature = "native"))]
#[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
#[unsafe(no_mangle)]
pub extern "C" fn initialize(params_len: u32, save: u32) -> u32 {
    let params_len = params_len;
    if params_len > 0 {
        let params_bytes = &scratch()[..params_len as usize];
        if load_params(params_bytes).is_ok() {
            return params_len;
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
                return len;
            }
            return len as u32;
        }
    }
    0
}

#[cfg(not(feature = "native"))]
#[cfg_attr(feature = "polkavm", polkavm_derive::polkavm_export)]
#[unsafe(no_mangle)]
pub extern "C" fn execute(req_len: u32) -> u32 {
    let req_bytes = &scratch()[..req_len as usize];
    let req: WorkRequest = match Decode::decode(&mut &req_bytes[..]) {
        Ok(req) => req,
        Err(_) => return 0,
    };
    let res = execute_work_request(&req);

    // Encode the response back to the scratch pad and return the length of the response.
    let res_bytes = res.encode();
    let res_len = res_bytes.len();
    let scratch = mut_scratch();
    scratch[..res_len].copy_from_slice(&res_bytes);

    return res_len as u32;
}
