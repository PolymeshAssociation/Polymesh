use codec::{Decode, Encode};
use core::fmt::{Display, Formatter};
use core::str;
use rstd::prelude::*;
use runtime_primitives::traits::Printable;
#[cfg(feature = "std")]
use runtime_primitives::{Deserialize, Serialize};
use sr_io;
const _POLY_DID_PREFIX: &'static str = "did:poly:";
const POLY_DID_PREFIX_LEN: usize = 9; // _POLY_DID_PREFIX.len(); // CI does not support: #![feature(const_str_len)]
const POLY_DID_LEN: usize = POLY_DID_PREFIX_LEN + UUID_LEN * 2;
const UUID_LEN: usize = 32usize;

/// Polymesh Identifier ID.
/// It is stored internally as an `u128` but it can be load from string with the following format:
/// "did:poly:<32 Hex characters>".
///
/// # From str
/// The current implementation of `TryFrom<&str>` requires exactly 32 hexadecimal characters for
/// code part of DID.
/// Valid examples are the following:
///  - "did:poly:ab01cd12ef34ab01cd12ef34ab01cd12"
/// Invalid examples:
///  - "did:poly:ab01"
///  - "did:poly:1"
///  - "DID:poly:..."
#[derive(Encode, Decode, Default, PartialOrd, Ord, PartialEq, Eq, Clone, Copy, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct IdentityId([u8; UUID_LEN]);

impl Display for IdentityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "did:poly:")?;
        for byte in &self.0 {
            f.write_fmt(format_args!("{:02x}", byte))?;
        }
        Ok(())
    }
}

impl From<u128> for IdentityId {
    fn from(id: u128) -> Self {
        let mut encoded_id = id.encode();
        encoded_id.resize(32, 0);
        let mut did = [0; 32];
        did.copy_from_slice(&encoded_id);
        IdentityId(did)
    }
}

use rstd::convert::TryFrom;
use srml_support::ensure;

impl TryFrom<&str> for IdentityId {
    type Error = &'static str;

    fn try_from(did: &str) -> Result<Self, Self::Error> {
        ensure!(did.len() == POLY_DID_LEN, "Invalid length of IdentityId");

        // Check prefix
        let prefix = &did[..POLY_DID_PREFIX_LEN];
        ensure!(prefix == _POLY_DID_PREFIX, "Missing 'did:poly:' prefix");

        // Check hex code
        let did_code = (POLY_DID_PREFIX_LEN..POLY_DID_LEN)
            .step_by(2)
            .map(|idx| u8::from_str_radix(&did[idx..idx + 2], 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| "DID code is not a valid hex")?;

        if did_code.len() == UUID_LEN {
            let mut uuid_fixed = [0; 32];
            uuid_fixed.copy_from_slice(&did_code);
            Ok(IdentityId(uuid_fixed))
        } else {
            Err("DID code is not a valid did")
        }
    }
}

impl TryFrom<&[u8]> for IdentityId {
    type Error = &'static str;

    fn try_from(did: &[u8]) -> Result<Self, Self::Error> {
        let did_str = str::from_utf8(did).map_err(|_| "DID is not valid UTF-8")?;
        IdentityId::try_from(did_str)
    }
}

impl From<[u8; UUID_LEN]> for IdentityId {
    fn from(s: [u8; UUID_LEN]) -> Self {
        IdentityId(s)
    }
}

impl Printable for IdentityId {
    fn print(&self) {
        sr_io::print_utf8("did:poly:".as_bytes());
        sr_io::print_hex(&self.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use srml_support::assert_err;
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        assert_eq!(IdentityId::default().0, [0; 32]);
        let valid_did =
            hex::decode("f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976")
                .expect("Decoding failed");
        let mut valid_did_without_prefix = [0; 32];
        valid_did_without_prefix.copy_from_slice(&valid_did);

        assert!(IdentityId::try_from(valid_did_without_prefix).is_ok());

        assert!(IdentityId::try_from(
            "did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976"
        )
        .is_ok());

        assert_err!(
            IdentityId::try_from(
                "did:OOLY:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d0976"
                    .as_bytes()
            ),
            "Missing 'did:poly:' prefix"
        );
        assert_err!(
            IdentityId::try_from("did:poly:a4a7".as_bytes()),
            "Invalid length of IdentityId"
        );

        assert_err!(
            IdentityId::try_from(
                "did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d097X"
                    .as_bytes()
            ),
            "DID code is not a valid hex"
        );

        let mut non_utf8: Vec<u8> =
            b"did:poly:f1d273950ddaf693db228084d63ef18282e00f91997ae9df4f173f09e86d".to_vec();
        non_utf8.append(&mut [0, 159, 146, 150].to_vec());
        assert_err!(
            IdentityId::try_from(non_utf8.as_slice()),
            "DID is not valid UTF-8"
        );
    }
}
