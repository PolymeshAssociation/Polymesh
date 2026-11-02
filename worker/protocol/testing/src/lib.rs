// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2026 Polymesh Association

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "native"), no_main)]

polymesh_worker_common::impl_module_runtime! {
    heap_size: 10 * 1024 * 1024,
    scratch_size: 4 * 1024 * 1024,
    error: Error,
    init_params: init_params,
    load_params: load_params,
    save_params: save_params,
    execute: execute_work_request,
}

polymesh_worker_common::impl_work_request!(TestWorkRequest, TestWorkResponse, PROTOCOL);

use polymesh_worker_common::{PROTOCOL_TESTING, Protocol, ProtocolError, ProtocolVersion};

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
    #[cfg(feature = "testing")]
    Trap,
}

impl TestWorkRequest {
    fn do_execute(self) -> Result<TestWorkResponse, ProtocolError> {
        match self {
            Self::VerifyVersion(req) => {
                let res = req.do_verify()?;
                Ok(TestWorkResponse::VerifyVersion(res))
            }
            #[cfg(feature = "testing")]
            Self::Trap => {
                #[cfg(not(feature = "native"))]
                panic!("Testing worker VM trap");
                #[cfg(feature = "native")]
                Ok(TestWorkResponse::TrapFallback)
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
    #[cfg(feature = "testing")]
    TrapFallback,
}

#[cfg(all(test, feature = "testing", feature = "native"))]
mod tests {
    use super::*;

    #[test]
    fn trap_request_succeeds_in_runtime_fallback() {
        assert_eq!(
            TestWorkRequest::Trap.do_execute(),
            Ok(TestWorkResponse::TrapFallback)
        );
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
