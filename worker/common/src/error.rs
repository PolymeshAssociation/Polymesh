use codec::{Decode, Encode};

use crate::{WorkRequestId, WorkerSessionId};

/// Errors that can occur in the worker protocol implementation.
#[derive(Clone, Debug, Encode, Decode, PartialEq, Eq)]
#[repr(u8)]
pub enum ProtocolError {
    CustomProtocolError([u8; 3]) = 0,
    ExecuteWorkFailed = 1,
    DecodingFailed = 2,
    UnexpectedResponse = 3,
    Unknown(u32),
}

impl ProtocolError {
    /// Convert a `u32` error code into a `ProtocolError`.
    pub fn from_u32(value: u32) -> Self {
        let bytes = value.to_le_bytes();
        match bytes[0] {
            0 => Self::CustomProtocolError([bytes[1], bytes[2], bytes[3]]),
            1 => Self::ExecuteWorkFailed,
            2 => Self::DecodingFailed,
            3 => Self::UnexpectedResponse,
            _ => Self::Unknown(value),
        }
    }

    /// Convert a `ProtocolError` into a `u32` error code.
    pub fn to_u32(&self) -> u32 {
        match self {
            Self::CustomProtocolError(err) => u32::from_le_bytes([0, err[0], err[1], err[2]]),
            Self::ExecuteWorkFailed => 1,
            Self::DecodingFailed => 2,
            Self::UnexpectedResponse => 3,
            Self::Unknown(value) => *value,
        }
    }

    /// Support for custom protocol errors.
    pub fn custom_error(err: [u8; 3]) -> Self {
        Self::CustomProtocolError(err)
    }
}

pub type WorkerErrorNum = u64;

/// Errors that can occur in the host worker implementation.
#[derive(Debug, Clone, Encode, Decode)]
#[repr(u8)]
pub enum WorkerError {
    NoError(WorkerErrorNum) = 0,
    NoBackendAvailable = 1,
    ModuleMemoryError = 2,
    ModuleInitializationFailed = 3,
    ModuleSaveContextFailed = 4,
    ModuleExecutionFailed = 5,
    DecodingFailed = 6,
    SessionNotFound(WorkerSessionId) = 7,
    SessionRequestNotFound(WorkerSessionId, WorkRequestId) = 8,
    BackendNotSupported = 9,
    Unknown(WorkerErrorNum),
}

impl WorkerError {
    /// Convert a `u64` error code into a `WorkerError`.
    ///
    /// The top 8 bits of the `u64` are used for the error type, and the remaining 56 bits are used for error details.
    pub fn from_u64(value: WorkerErrorNum) -> Self {
        let error_type = (value >> 56) as u8;
        let error_details = value & 0x00FFFFFFFFFFFFFF;
        match error_type {
            0 => Self::NoError(error_details),
            1 => Self::NoBackendAvailable,
            2 => Self::ModuleMemoryError,
            3 => Self::ModuleInitializationFailed,
            4 => Self::ModuleSaveContextFailed,
            5 => Self::ModuleExecutionFailed,
            6 => Self::DecodingFailed,
            7 => {
                let session_id = (error_details >> 32) as u32;
                Self::SessionNotFound(session_id)
            }
            8 => {
                let session_id = (error_details >> 32) as u32;
                let request_id = (error_details & 0xFFFFFFFF) as u32;
                Self::SessionRequestNotFound(session_id, request_id)
            }
            9 => Self::BackendNotSupported,
            _ => Self::Unknown(value),
        }
    }

    /// Convert a `WorkerError` into a `u64` error code.
    pub fn to_u64(&self) -> WorkerErrorNum {
        match self {
            Self::NoError(details) => (*details) & 0x00FFFFFFFFFFFFFF,
            Self::NoBackendAvailable => 1 << 56,
            Self::ModuleMemoryError => 2 << 56,
            Self::ModuleInitializationFailed => 3 << 56,
            Self::ModuleSaveContextFailed => 4 << 56,
            Self::ModuleExecutionFailed => 5 << 56,
            Self::DecodingFailed => 6 << 56,
            Self::SessionNotFound(session_id) => {
                let session_id_part = (*session_id as WorkerErrorNum) << 32;
                (7 << 56) | session_id_part
            }
            Self::SessionRequestNotFound(session_id, request_id) => {
                let session_id_part = (*session_id as WorkerErrorNum) << 32;
                let request_id_part = (*request_id as WorkerErrorNum) & 0xFFFFFFFF;
                (8 << 56) | session_id_part | request_id_part
            }
            Self::BackendNotSupported => 9 << 56,
            Self::Unknown(value) => *value,
        }
    }

    pub fn result_from_u64(value: WorkerErrorNum) -> Result<(), Self> {
        let err = Self::from_u64(value);
        err.into()
    }

    pub fn result_id_from_u64(value: WorkerErrorNum) -> Result<u32, Self> {
        let err = Self::from_u64(value);
        err.into()
    }
}

impl From<WorkerError> for Result<(), WorkerError> {
    fn from(err: WorkerError) -> Self {
        match err {
            WorkerError::NoError(_) => Ok(()),
            _ => Err(err),
        }
    }
}

impl From<WorkerError> for Result<u32, WorkerError> {
    fn from(err: WorkerError) -> Self {
        match err {
            WorkerError::NoError(value) => Ok(value as u32),
            _ => Err(err),
        }
    }
}

impl From<WorkerError> for Result<u64, WorkerError> {
    fn from(err: WorkerError) -> Self {
        match err {
            WorkerError::NoError(value) => Ok(value),
            _ => Err(err),
        }
    }
}

impl From<WorkerError> for WorkerErrorNum {
    fn from(err: WorkerError) -> Self {
        err.to_u64()
    }
}

impl From<WorkerErrorNum> for WorkerError {
    fn from(value: WorkerErrorNum) -> Self {
        WorkerError::from_u64(value)
    }
}

impl From<Result<(), WorkerError>> for WorkerError {
    fn from(result: Result<(), WorkerError>) -> Self {
        match result {
            Ok(()) => WorkerError::NoError(0),
            Err(err) => err,
        }
    }
}

impl From<Result<u32, WorkerError>> for WorkerError {
    fn from(result: Result<u32, WorkerError>) -> Self {
        match result {
            Ok(value) => WorkerError::NoError(value as WorkerErrorNum),
            Err(err) => err,
        }
    }
}

impl From<Result<u64, WorkerError>> for WorkerError {
    fn from(result: Result<u64, WorkerError>) -> Self {
        match result {
            Ok(value) => WorkerError::NoError(value),
            Err(err) => err,
        }
    }
}
