use core::fmt::{Display, Formatter};
use core::str;
use parity_scale_codec::{Decode, Encode};
use rstd::prelude::*;

const _POLY_DID_PREFIX: &'static str = "did:poly:";
const _POLY_DID_PREFIX_LEN: usize = 9; // _POLY_DID_PREFIX.len(); // CI does not support: #![feature(const_str_len)]
const _UUID_LEN: usize = 32usize;
const _POLY_DID_LEN: usize = _POLY_DID_PREFIX_LEN + _UUID_LEN;

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
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Copy, Debug)]
pub struct IdentityId(u128);

impl IdentityId {
    /// Generate a randomized `IdentityId`.
    /// # TODO
    /// It is not random yet. The implementation could use hash of accountId + nonce.
    pub fn generate() -> Self {
        // let v = rand::random::<u128>();
        IdentityId(0u128)
    }
}

impl Display for IdentityId {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "did:poly:{:032x}", self.0)
    }
}

impl From<u128> for IdentityId {
    fn from(id: u128) -> Self {
        IdentityId(id)
    }
}

use rstd::convert::TryFrom;
use srml_support::ensure;

impl TryFrom<&str> for IdentityId {
    type Error = &'static str;

    fn try_from(did: &str) -> Result<Self, Self::Error> {
        ensure!(did.len() == _POLY_DID_LEN, "Invalid length of IdentityId");

        // Check prefix
        let prefix = &did[.._POLY_DID_PREFIX_LEN];
        ensure!(prefix == _POLY_DID_PREFIX, "Missing 'did:poly:' prefix");

        // Check hex code
        let did_code = (_POLY_DID_PREFIX_LEN.._POLY_DID_LEN)
            .step_by(2)
            .map(|idx| u8::from_str_radix(&did[idx..idx + 2], 16))
            .collect::<Result<Vec<u8>, _>>()
            .map_err(|_| "DID code is not a valid hex")?;

        if did_code.len() == 16 {
            let mut uuid_fixed = [0u8; 16];
            uuid_fixed.copy_from_slice(&did_code);

            let uuid = u128::from_ne_bytes(uuid_fixed);
            Ok(IdentityId(uuid))
        } else {
            Err("DID code is not a valid")
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

#[cfg(test)]
mod tests {
    use super::*;
    use srml_support::assert_err;
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        assert_eq!(IdentityId::default().0, 0u128);
        assert!(
            IdentityId::try_from("did:poly:a4a7d08f2c4d4d1e863aced28cdf9edd".as_bytes()).is_ok()
        );

        assert_err!(
            IdentityId::try_from("did:OOLY:a4a7d08f2c4d4d1e863aced28cdf9edd".as_bytes()),
            "Missing 'did:poly:' prefix"
        );
        assert_err!(
            IdentityId::try_from("did:poly:a4a7".as_bytes()),
            "Invalid length of IdentityId"
        );

        let mut non_utf8: Vec<u8> = b"did:poly:a4a7d08f2c4d4d1e863aced28cdf".to_vec();
        non_utf8.append(&mut [0, 159, 146, 150].to_vec());
        assert_err!(
            IdentityId::try_from(non_utf8.as_slice()),
            "DID is not valid UTF-8"
        );

        assert_err!(
            IdentityId::try_from("did:poly:a1a7".as_bytes()),
            "Invalid length of IdentityId"
        );
        assert_err!(
            IdentityId::try_from("did:poly:a4a7d08f2c4d4d1e863aced28cdf9edX".as_bytes()),
            "DID code is not a valid hex"
        );
    }
}
