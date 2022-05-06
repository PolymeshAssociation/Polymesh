use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use sp_runtime::traits::{IdentifyAccount, Verify};

/// Alias to an sr25519 or ed25519 key.
pub type AccountId = <<super::Signature as Verify>::Signer as IdentifyAccount>::AccountId;

/// Wrapper to serialize `AccountId` to a 0x-prefixed hex representation.
#[derive(Clone, PartialEq, Eq, Encode, Decode, Default, Debug)]
pub struct HexAccountId(pub [u8; 32]);

#[cfg(feature = "std")]
impl Serialize for HexAccountId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let hex: String = rustc_hex::ToHex::to_hex(&self.0[..]);
        serializer.serialize_str(&format!("0x{}", hex))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for HexAccountId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let base_string = String::deserialize(deserializer)?;
        let offset = if base_string.starts_with("0x") { 2 } else { 0 };
        let s = &base_string[offset..];
        if s.len() != 64 {
            Err(serde::de::Error::custom(
                "Bad length of AccountId (should be 66 including '0x')",
            ))?;
        }
        let raw: Vec<u8> = rustc_hex::FromHex::from_hex(s)
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?;
        let mut r = Self::default();
        r.0.copy_from_slice(&raw);
        Ok(r)
    }
}
