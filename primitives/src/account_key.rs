// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, Encode};
#[cfg(feature = "std")]
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sp_core::sr25519::Public;
use sp_std::{
    cmp::{Ord, PartialOrd},
    convert::TryFrom,
    default::Default,
    prelude::Vec,
};

/// Size of key, when it is u64
const KEY_SIZE_TEST: usize = 8;
const KEY_SIZE: usize = 32;

/// It stores a simple key.
/// It uses fixed size to avoid dynamic memory allocation.
#[derive(Encode, Decode, Default, PartialOrd, Ord, Eq, Copy, Clone, Debug)]
pub struct AccountKey(pub [u8; KEY_SIZE]);

#[cfg(feature = "std")]
impl Serialize for AccountKey {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.using_encoded(|bytes| sp_core::bytes::serialize(bytes, serializer))
    }
}

#[cfg(feature = "std")]
impl<'de> Deserialize<'de> for AccountKey {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let r = sp_core::bytes::deserialize(deserializer)?;
        Decode::decode(&mut &r[..])
            .map_err(|e| serde::de::Error::custom(format!("Decode error: {}", e)))
    }
}

impl AccountKey {
    /// It returns this key as a byte slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
}

impl TryFrom<Vec<u8>> for AccountKey {
    type Error = &'static str;

    #[inline]
    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        AccountKey::try_from(v.as_slice())
    }
}

impl TryFrom<&Vec<u8>> for AccountKey {
    type Error = &'static str;

    #[inline]
    fn try_from(v: &Vec<u8>) -> Result<Self, Self::Error> {
        AccountKey::try_from(v.as_slice())
    }
}

impl TryFrom<&str> for AccountKey {
    type Error = &'static str;

    #[inline]
    fn try_from(s: &str) -> Result<Self, Self::Error> {
        AccountKey::try_from(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for AccountKey {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        let mut k = AccountKey::default();
        match s.len() {
            KEY_SIZE => k.0.copy_from_slice(s),
            KEY_SIZE_TEST => k.0[..KEY_SIZE_TEST].copy_from_slice(s),
            _ => return Err("Invalid size for a key"),
        };
        Ok(k)
    }
}

impl From<[u8; KEY_SIZE]> for AccountKey {
    #[inline]
    fn from(s: [u8; KEY_SIZE]) -> Self {
        AccountKey(s)
    }
}

impl From<Public> for AccountKey {
    #[inline]
    fn from(k: Public) -> Self {
        AccountKey(k.0)
    }
}

impl PartialEq for AccountKey {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&[u8]> for AccountKey {
    fn eq(&self, other: &&[u8]) -> bool {
        match other.len() {
            KEY_SIZE => self.0 == *other,
            KEY_SIZE_TEST => {
                self.0[..KEY_SIZE_TEST] == **other
                    && self.0[KEY_SIZE_TEST..] == [0u8; KEY_SIZE - KEY_SIZE_TEST]
            }
            _ => false,
        }
    }
}

impl PartialEq<Vec<u8>> for AccountKey {
    #[inline]
    fn eq(&self, other: &Vec<u8>) -> bool {
        self == &other.as_slice()
    }
}

impl PartialEq<Public> for AccountKey {
    #[inline]
    fn eq(&self, other: &Public) -> bool {
        self == &&other.0[..]
    }
}

#[cfg(test)]
mod tests {
    use super::{AccountKey, KEY_SIZE};
    use std::convert::TryFrom;

    #[test]
    fn serialize_deserialize_account_key() {
        let secret_string: [u8; KEY_SIZE] = [1u8; KEY_SIZE];
        let account_key = AccountKey::try_from(secret_string).unwrap();
        let serialize = serde_json::to_string(&account_key).unwrap();
        let serialize_data =
            "\"0x0101010101010101010101010101010101010101010101010101010101010101\"";
        assert_eq!(serialize_data, serialize);
        let deserialize = serde_json::from_str::<AccountKey>(&serialize).unwrap();
        assert_eq!(account_key, deserialize);
    }

    #[test]
    fn build_test() {
        let k: [u8; KEY_SIZE] = [1u8; KEY_SIZE];
        let k2 = "ABCDABCD".as_bytes().to_vec();

        assert!(AccountKey::try_from(k).is_ok());
        assert!(AccountKey::try_from(k2.as_slice()).is_ok());
        assert!(AccountKey::try_from(k2).is_ok());

        assert!(AccountKey::try_from("ABCDABCDx".as_bytes()).is_err());
    }
}
