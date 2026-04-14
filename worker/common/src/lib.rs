#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use sp_std::vec::Vec;

pub mod error;
pub use error::*;

pub mod config;
pub use config::*;

/// Seed used for verifying or generating proofs.
pub type WorkSeed = [u8; 32];

pub type WorkerVersion = u8;
pub type WorkerSessionId = u32;

/// Backend bitmask type.
pub type BackendBitmask = u32;

/// Backend module code hash type.
pub type BackendCodeHash = [u8; 32];

/// Backend module context hash type.
pub type BackendContextHash = [u8; 32];

/// For a give protocol, version and backend kind, the code hash and context hash of the module to be loaded.
#[derive(Clone, Debug, Encode, Decode)]
pub struct BackendCodeAndContextHash {
    /// The code hash for loading the module code bytes.
    pub code_hash: BackendCodeHash,
    /// If given the context is also loaded for faster initization.
    pub context_hash: Option<BackendContextHash>,
}

/// The maximum module code size limit, used for decompression safety (e.g. to prevent decompression bombs).
pub const MODULE_CODE_SIZE_LIMIT: usize = 10 * 1024 * 1024; // 10 MB

/// Pack a fat pointer (ptr and length) into a u64.
pub fn pack_fat_pointer(ptr: u32, len: u32) -> u64 {
    let ptr_val = ptr as u64;
    let len_val = len as u64;
    (len_val << 32) | ptr_val
}

/// Unpack a fat pointer (ptr and length) from a u64.
pub fn unpack_fat_pointer(fat_ptr: u64) -> (u32, u32) {
    let ptr = (fat_ptr & 0xFFFFFFFF) as u32;
    let len = (fat_ptr >> 32) as u32;
    (ptr, len)
}

/// A `ProtocolError` can be encoded as a fat pointer where the `len` is set to `u32::MAX` to indicate an error, and the `ptr` is the error code as a `u32`.
pub fn unpack_fat_results(fat_ptr: u64) -> Result<(u32, u32), ProtocolError> {
    let (ptr, len) = unpack_fat_pointer(fat_ptr);
    if len == u32::MAX {
        Err(ProtocolError::from_u32(ptr))
    } else {
        Ok((ptr, len))
    }
}

/// The backend kind.
///
/// This is used to allow disabling certain backends in the future if they are found to be insecure or have other issues.
/// It also allows us to have multiple backends for the same protocol if needed.
#[derive(Clone, Copy, Debug)]
#[repr(u32)]
pub enum BackendKind {
    Native = 0,
    PolkaVM = 1,
    Wasmtime = 2,
    Wasmer = 3,
}

impl BackendKind {
    /// All supported backends.
    pub fn all_mask() -> BackendBitmask {
        BackendKind::Native | BackendKind::PolkaVM | BackendKind::Wasmtime | BackendKind::Wasmer
    }

    /// Convert from a u32 to a backend kind.
    pub const fn from_u32(value: u32) -> Option<Self> {
        match value {
            0 => Some(Self::Native),
            1 => Some(Self::PolkaVM),
            2 => Some(Self::Wasmtime),
            3 => Some(Self::Wasmer),
            _ => None,
        }
    }

    /// Convert the backend kind to a u32.
    pub const fn as_u32(&self) -> u32 {
        match self {
            BackendKind::Native => 0,
            BackendKind::PolkaVM => 1,
            BackendKind::Wasmtime => 2,
            BackendKind::Wasmer => 3,
        }
    }

    /// Convert the backend kind to a bitmask.
    pub const fn as_bitmask(&self) -> BackendBitmask {
        1 << self.as_u32()
    }

    /// Is supported by the given backend bitmask.
    pub const fn is_supported_by(&self, supported: BackendBitmask) -> bool {
        let mask = self.as_bitmask();
        (supported & mask) != 0
    }

    /// Backend kind to storage key suffix.
    ///
    /// This is used to generate the storage key for loading the module from Substrate on-chain storage.
    pub fn to_storage_key_suffix(&self) -> &'static [u8] {
        match self {
            BackendKind::Native => b"native",
            BackendKind::PolkaVM => b"polkavm",
            BackendKind::Wasmtime | BackendKind::Wasmer => b"wasm",
        }
    }

    /// Append backend kind to buffer for storage key generation.
    pub fn append_to_buf(&self, buf: &mut Vec<u8>) {
        buf.extend_from_slice(self.to_storage_key_suffix());
    }
}

impl core::ops::BitOr for BackendKind {
    type Output = BackendBitmask;

    fn bitor(self, rhs: Self) -> Self::Output {
        self.as_bitmask() | rhs.as_bitmask()
    }
}

impl core::ops::BitOr<BackendBitmask> for BackendKind {
    type Output = BackendBitmask;

    fn bitor(self, rhs: BackendBitmask) -> Self::Output {
        self.as_bitmask() | rhs
    }
}

impl core::ops::BitOr<BackendKind> for BackendBitmask {
    type Output = BackendBitmask;

    fn bitor(self, rhs: BackendKind) -> Self::Output {
        self | rhs.as_bitmask()
    }
}

/// Protocol id.
#[derive(
    Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq, PartialOrd, Ord
)]
pub struct ProtocolId(u8);

pub type ProtocolNumber = u32;

/// The protocol id for the P-DART protocol.
///
/// 0x50 is chosen as it is the ASCII code for 'P', which stands for Polymesh.
pub const PROTOCOL_PDART: ProtocolId = ProtocolId(0x50);

/// The protocol id and version.
#[derive(
    Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq, PartialOrd, Ord
)]
pub struct Protocol {
    pub id: ProtocolId,
    pub version: ProtocolVersion,
}

impl Protocol {
    /// A protocol with id 0 and version 0, which is used to indicate no protocol.
    ///
    /// This is mainly used when creating a session without pre-loading a specific protocol, and the protocol will be set later when the first work request is made.
    pub fn none() -> Self {
        Self {
            id: ProtocolId(0),
            version: ProtocolVersion::default(),
        }
    }

    /// Is this protocol is `none`, which means no protocol.
    pub fn is_none(&self) -> bool {
        self.id.0 == 0 && self.version == ProtocolVersion::default()
    }

    /// Create a new protocol with the given id and version.
    pub fn new(id: ProtocolId, version: ProtocolVersion) -> Self {
        Self { id, version }
    }

    /// Create a new protocol from a protocol number.
    pub const fn from_number(num: ProtocolNumber) -> Self {
        let id = ProtocolId((num >> 24) as u8);
        let version = ProtocolVersion {
            major: ((num >> 16) & 0xFF) as u8,
            minor: ((num >> 8) & 0xFF) as u8,
            patch: (num & 0xFF) as u8,
        };
        Self { id, version }
    }

    /// Convert the protocol to a number for easier handling in the host <-> runtime communication.
    pub const fn to_number(&self) -> ProtocolNumber {
        ((self.id.0 as ProtocolNumber) << 24)
            | ((self.version.major as ProtocolNumber) << 16)
            | ((self.version.minor as ProtocolNumber) << 8)
            | (self.version.patch as ProtocolNumber)
    }

    /// Append protocol to buffer for storage key generation.
    pub fn append_to_buf(&self, buf: &mut Vec<u8>) {
        buf.push(self.id.0);
        buf.push(b':');
        buf.push(self.version.major);
        buf.push(b'.');
        buf.push(self.version.minor);
        buf.push(b'.');
        buf.push(self.version.patch);
    }
}

/// The protocol version is used load the correct module for the given protocol and version.
#[derive(
    Clone, Copy, Debug, Default, Encode, Decode, PartialEq, Eq, PartialOrd, Ord
)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
    pub patch: u8,
}

impl ProtocolVersion {
    /// Create a new protocol version with the given major, minor and patch version.
    pub const fn new(major: u8, minor: u8, patch: u8) -> Self {
        Self {
            major,
            minor,
            patch,
        }
    }
}

pub type WorkRequestId = u32;

pub type WorkFlags = u16;

/// Work request status.
#[derive(Clone, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkStatus {
    /// Unknown status, used when the host returns an invalid status or when parsing fails.
    Unknown = 0,
    /// The work request is being processed.
    Pending,
    /// The work request has been completed successfully.
    Completed,
    /// The work request failed to execute on the host, the runtime should
    /// fallback to executing it in the runtime.
    ExecutionFailedFallbackToRuntime,
    /// Session not found.  This shouldn't happen because the session should be checked before submitting work, but we return this just in case.
    SessionNotFound,
}

impl WorkStatus {
    /// Convert the work status to a u8 for easier handling in the host <-> runtime communication.
    pub const fn as_u8(&self) -> u8 {
        match self {
            WorkStatus::Unknown => 0,
            WorkStatus::Pending => 1,
            WorkStatus::Completed => 2,
            WorkStatus::ExecutionFailedFallbackToRuntime => 3,
            WorkStatus::SessionNotFound => 4,
        }
    }

    /// Convert from a u8 to a work status.
    pub const fn from_u8(value: u8) -> Self {
        match value {
            1 => Self::Pending,
            2 => Self::Completed,
            3 => Self::ExecutionFailedFallbackToRuntime,
            4 => Self::SessionNotFound,
            _ => Self::Unknown,
        }
    }
}

/// Combine work status (8 bits), flags (16 bits) and id (32 bits) into a single u64 for easier handling in the host <-> runtime communication.
///
/// The layout of the u64 is as follows:
/// - bits 0-7: work status (as u8)
/// - bits 8-23: flags (as u16)
/// - bits 24-31: reserved for future use (currently unused)
/// - bits 32-63: work request id (as u32)
///
pub type WorkStatusFlagsAndId = u64;

pub const FALLBACK_TO_RUNTIME: WorkStatusFlagsAndId =
    work_status_flags_and_id(WorkStatus::ExecutionFailedFallbackToRuntime, 0, 0);

pub const fn work_status_flags_and_id(
    status: WorkStatus,
    flags: WorkFlags,
    id: WorkRequestId,
) -> WorkStatusFlagsAndId {
    let status_u64 = status.as_u8() as u64;
    let flags_u64 = flags as u64;
    let id_u64 = id as u64;
    (id_u64 << 32) | (flags_u64 << 8) | status_u64
}

pub fn parse_work_status_flags_and_id(
    value: WorkStatusFlagsAndId,
) -> (WorkStatus, WorkFlags, WorkRequestId) {
    let status_u8 = (value & 0xFF) as u8;
    let flags_u16 = ((value >> 8) & 0xFFFF) as u16;
    let id_u32 = (value >> 32) as u32;
    let status = WorkStatus::from_u8(status_u8);
    (status, flags_u16, id_u32)
}

/// Work request for a specific protocol.
///
/// The work request is send from the runtime to the host for a specific protocol and version.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkRequest(pub Vec<u8>);

impl WorkRequest {
    /// Create a new work request for the given protocol and request data.
    pub fn new<T: Encode>(req: T) -> Self {
        let work = req.encode();
        Self(work)
    }

    /// Decode the work request data into the given protocol-specific request type.
    pub fn decode<T: Decode>(&self) -> Result<T, ProtocolError> {
        T::decode(&mut &self.0[..]).map_err(|_| ProtocolError::DecodingFailed)
    }
}

pub type WorkResponseResult = Result<WorkResponse, ProtocolError>;

/// Response to a work request.
#[derive(Clone, Debug, Encode, Decode)]
pub struct WorkResponse(pub Vec<u8>);

impl WorkResponse {
    /// Create a new work response for the given protocol-specific response data.
    pub fn new<T: Encode>(res: T) -> Self {
        let res = res.encode();
        Self(res)
    }

    /// Decode the work response data into the given protocol-specific response type.
    pub fn decode<T: Decode>(&self) -> Result<T, ProtocolError> {
        T::decode(&mut &self.0[..]).map_err(|_| ProtocolError::DecodingFailed)
    }
}
