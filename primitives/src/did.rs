use core::fmt::{Display, Formatter};
use parity_scale_codec::{Decode, Encode};
use rstd::prelude::*;
use hex;

const _POLY_DID_PREFIX: &'static [u8] = b"did:poly:";
const _POLY_DID_PREFIX_LEN: usize = _POLY_DID_PREFIX.len();
const _UUID_LEN: usize = 32usize;
const _POLY_DID_LEN: usize = _POLY_DID_PREFIX_LEN + _UUID_LEN;

/// Polymesh Distributed ID.
#[derive(Encode, Decode, Default, PartialEq, Eq, Clone, Debug)]
pub struct Did(u128);

impl Did {
    /// Generate a ramdomized Did.
    /// TODO It is not random
    pub fn generate() -> Self {
        // let v = rand::random::<u128>();
        Did(0u128)
    }
}

impl Display for Did {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        write!(f, "did:poly:{:32x}", self.0)
    }
}

impl From<u128> for Did {
    fn from(id: u128) -> Self {
        Did(id)
    }
}

use rstd::convert::TryFrom;
use srml_support::ensure;

impl TryFrom<&[u8]> for Did {
    type Error = &'static str;

    fn try_from(did: &[u8]) -> Result<Self, Self::Error> {
        ensure!(did.len() == _POLY_DID_LEN, "Invalid length of did");

        let prefix = &did[.._POLY_DID_PREFIX_LEN];
        ensure!(prefix == _POLY_DID_PREFIX, "Missing 'did:poly:' prefix");
        let did_code_hex = &did[_POLY_DID_PREFIX_LEN.._POLY_DID_LEN];
        let did_code = hex::decode(did_code_hex).map_err(|_| "DID code is not a valid hex")?;
        if did_code.len() == 16 {
            let mut uuid_fixed = [0u8; 16];
            uuid_fixed.copy_from_slice(&did_code);

            let uuid = u128::from_ne_bytes(uuid_fixed);
            Ok(Did(uuid))
        } else {
            Err("DID code is not a valid")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use srml_support::assert_err;
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        assert!(Did::try_from("did:poly:a4a7d08f2c4d4d1e863aced28cdf9edd".as_bytes()).is_ok());

        assert_err!(
            Did::try_from("did:OOLY:a4a7d08f2c4d4d1e863aced28cdf9edd".as_bytes()),
            "Missing 'did:poly:' prefix"
        );
        assert_err!(
            Did::try_from("did:poly:a4a7".as_bytes()),
            "Invalid length of did"
        );

        let mut non_utf8: Vec<u8> = b"did:poly:a4a7d08f2c4d4d1e863aced28cdf".to_vec();
        non_utf8.append(&mut [0, 159, 146, 150].to_vec());
        assert_err!(
            Did::try_from(non_utf8.as_slice()),
            "DID code is not a valid hex"
        );

        assert_err!(
            Did::try_from("did:poly:a1a7".as_bytes()),
            "Invalid length of did"
        );
        assert_err!(
            Did::try_from("did:poly:a4a7d08f2c4d4d1e863aced28cdf9edX".as_bytes()),
            "DID code is not a valid hex"
        );
    }
}
