#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_std::vec::Vec;

/// Seed used for verifying or generating proofs.
pub type WorkSeed = [u8; 32];

/// Protocol id.
#[derive(Clone, Copy, Debug, Encode, Decode, PartialEq, Eq)]
pub struct ProtocolId(u8);

/// The protocol id for the P-DART protocol.
pub const PROTOCOL_PDART: ProtocolId = ProtocolId(0x50);

/// The protocol id and version.
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct Protocol {
    pub id: ProtocolId,
    pub version: ProtocolVersion,
}

/// The protocol version is used load the correct module for the given protocol and version.
#[derive(Clone, Copy, Debug, Encode, Decode)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

/// The common types for all worker protocols.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Error {
    VerifyProofFailed,
    GenerateProofFailed,
    DecodingFailed,
    InvalidModule,
    ModuleMemoryError,
    NoBackendAvailable,
    ProtocolError([u8; 4]),
}

impl Error {
    pub fn protocol_error<T: Encode>(err: T) -> Self {
        let mut encoded = [0u8; 4];
        let err_encoded = err.encode();
        let err_len = err_encoded.len().min(4);
        encoded[..err_len].copy_from_slice(&err_encoded[..err_len]);
        Self::ProtocolError(encoded)
    }
}

pub type WorkRequestId = u32;

/// Work request for a specific protocol.
///
/// The work request is send from the runtime to the host for a specific protocol and version.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkRequest {
    pub protocol: Protocol,
    pub work: Vec<u8>,
}

impl WorkRequest {
    /// Create a new work request for the given protocol and request data.
    pub fn new<T: Encode>(protocol: Protocol, req: T) -> Self {
        let work = req.encode();
        Self { protocol, work }
    }

    /// Decode the work request data into the given protocol-specific request type.
    pub fn decode<T: Decode>(&self) -> Result<T, Error> {
        T::decode(&mut &self.work[..]).map_err(|_| Error::DecodingFailed)
    }
}

/// Response to a work request.
#[derive(Clone, Debug, Encode, Decode)]
pub enum WorkResponse {
    Success(Vec<u8>),
    Error(Error),
}

impl WorkResponse {
    /// Create a new work response for the given protocol-specific response data.
    pub fn new<T: Encode>(res: T) -> Self {
        let res = res.encode();
        Self::Success(res)
    }

    /// Decode the work response data into the given protocol-specific response type.
    pub fn decode<T: Decode>(&self) -> Result<T, Error> {
        match self {
            Self::Success(res) => T::decode(&mut &res[..]).map_err(|_| Error::DecodingFailed),
            Self::Error(err) => Err(err.clone()),
        }
    }
}
