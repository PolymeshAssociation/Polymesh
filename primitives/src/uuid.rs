// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

/// It contains some constants to work with UUID.

/// Fields' indexes of the UUID, encoded as 16 octects.
/// ```ignore
///    0                   1                   2                   3
///    0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1 2 3 4 5 6 7 8 9 0 1
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                          time_low                             |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |       time_mid                |         time_hi_and_version   |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |clk_seq_hi_res |  clk_seq_low  |         node (0-1)            |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
///   |                         node (2-5)                            |
///   +-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+-+
/// ```
///
pub mod index {
    /// Index of variant field
    pub const VARIANT: usize = 8;
    /// Index of version field. It is inside `time_hi_and_version` field.
    pub const VERSION: usize = 6;
}

/// The reserved variants of UUIDs.
#[derive(Clone, Copy)]
pub enum Variant {
    /// Reserved by the NCS for backward compatibility.
    NCS,
    /// As described in the RFC4122 Specification (default).
    RFC4122,
    /// Reserved by Microsoft for backward compatibility.
    Microsoft,
    /// Reserved for future expansion.
    Future,
}

/// Version of UUID
#[derive(Clone, Copy)]
pub enum Version {
    /// Version 1: Time-based.
    V1 = 1,
    /// Version 2: DCE Security.
    V2,
    /// Version 3: MD5 hash.
    V3,
    /// Version 4: Random.
    V4,
    /// Version 5: SHA-1 hash.
    V5,
}

/// Set the variant into `uuid`.
///
/// The following table lists the contents of the variant field, where the letter "x" indicates a "don't-care" value.
///
/// ```ignore
/// Msb0  Msb1  Msb2  Description
///    0     x     x    Reserved, NCS backward compatibility.
///    1     0     x    The variant specified in this document.
///    1     1     0    Reserved, Microsoft Corporation backward compatibility
///    1     1     1    Reserved for future definition.
/// ```
pub fn set_variant(uuid: &mut [u8; 16], variant: Variant) {
    let byte = uuid[index::VARIANT];

    uuid[index::VARIANT] = match variant {
        Variant::NCS => byte & 0x7f,
        Variant::RFC4122 => (byte & 0x3f) | 0x80,
        Variant::Microsoft => (byte & 0x1f) | 0xc0,
        Variant::Future => (byte & 0x1f) | 0xe0,
    };
}

/// Set the version of the UUID.
///
/// The following table lists the currently-defined versions for this UUID variant.
///
/// ```ignore
///   Msb0  Msb1  Msb2  Msb3   Version  Description
///   0     0     0     1        1     The time-based version.
///   0     0     1     0        2     DCE Security version, with embedded POSIX UIDs.
///   0     0     1     1        3     The name-based version that uses MD5 hashing.
///   0     1     0     0        4     The randomly or pseudo-randomly version.
///   0     1     0     1        5     The name-based version that uses SHA-1 hashing.
/// ```
pub fn set_version(uuid: &mut [u8; 16], version: Version) {
    let byte = uuid[index::VERSION];
    uuid[index::VERSION] = (byte & 0x0f) | ((version as u8) << 4);
}
