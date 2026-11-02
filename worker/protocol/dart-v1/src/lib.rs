// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2023 Polymesh Association

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "native"), no_main)]
#![recursion_limit = "256"]

polymesh_worker_common::impl_module_runtime! {
    heap_size: 20 * 1024 * 1024,
    scratch_size: 10 * 1024 * 1024,
    error: Error,
    init_params: init_params,
    load_params: load_params,
    save_params: save_params,
    execute: execute_work_request,
}

polymesh_worker_common::impl_work_request!(DartWorkRequest, DartWorkResponse, PROTOCOL);

use polymesh_worker_common::{PROTOCOL_PDART, Protocol, ProtocolError, ProtocolVersion};

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

macro_rules! env_num {
    ($num_ty:ty, $env_var:expr) => {
        match <$num_ty>::from_str_radix(env!($env_var), 10) {
            Ok(num) => num,
            Err(_) => 0,
        }
    };
}

pub const PROTOCOL: Protocol = Protocol {
    id: PROTOCOL_PDART,
    version: ProtocolVersion::new(
        env_num!(u16, "CARGO_PKG_VERSION_MAJOR"),
        env_num!(u16, "CARGO_PKG_VERSION_MINOR"),
        env_num!(u16, "CARGO_PKG_VERSION_PATCH"),
    ),
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
