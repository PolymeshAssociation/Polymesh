// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Ticker symbol
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use polymesh_primitives_derive::{DeserializeU8StrongTyped, SerializeU8StrongTyped};
use scale_info::TypeInfo;

use sp_std::vec::Vec;

/// Ticker length.
pub const TICKER_LEN: usize = 12;

/// Ticker symbol.
///
/// This type stores fixed-length case-sensitive byte strings. Any value of this type that is
/// received by a Substrate module call method has to be converted to canonical uppercase
/// representation using [`Ticker::canonize`].
#[derive(Encode, Decode, TypeInfo)]
#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(
    feature = "std",
    derive(SerializeU8StrongTyped, DeserializeU8StrongTyped)
)]
pub struct Ticker([u8; TICKER_LEN]);

impl Default for Ticker {
    fn default() -> Self {
        Ticker([0u8; TICKER_LEN])
    }
}

impl AsRef<[u8]> for Ticker {
    fn as_ref(&self) -> &[u8] {
        &self.0
    }
}

impl Ticker {
    /// Used to make an unchecked ticker.
    pub const fn new_unchecked(bytes: [u8; TICKER_LEN]) -> Self {
        Ticker(bytes)
    }

    /// Given a number, this function generates a ticker with
    /// A-Z, least number of characters in Lexicographic order.
    pub fn generate(n: u64) -> Vec<u8> {
        fn calc_base26(n: u64, base_26: &mut Vec<u8>) {
            if n >= 26 {
                // Subtracting 1 is not required and shouldn't be done for a proper base_26 conversion
                // However, without this hack, B will be the first char after a bump in number of chars.
                // i.e. the sequence will go A,B...Z,BA,BB...ZZ,BAA. We want the sequence to start with A.
                // Subtracting 1 here means we are doing 1 indexing rather than 0.
                // i.e. A = 1, B = 2 instead of A = 0, B = 1
                calc_base26((n / 26) - 1, base_26);
            }
            let character = n % 26 + 65;
            base_26.push(character as u8);
        }
        let mut base_26 = Vec::new();
        calc_base26(n, &mut base_26);
        base_26
    }

    /// Given a number, this function generates a ticker with
    /// A-Z, least number of characters in Lexicographic order.
    /// Also convert it into the `Ticker` type.
    pub fn generate_into(n: u64) -> Self {
        Ticker::from_slice_truncated(&*Ticker::generate(n))
    }

    /// Computes the effective length of the ticker, that is, the length of the minimal prefix after
    /// which only zeros appear.
    pub fn len(&self) -> usize {
        for i in (0..TICKER_LEN).rev() {
            if self.0[i] != 0 {
                return i + 1;
            }
        }
        0
    }

    /// Returns `true` if the ticker is empty, that is, if it has no prefix of characters other than
    /// `0u8`.
    #[inline]
    pub fn is_empty(&self) -> bool {
        self.0[0] == 0
    }

    /// Returns the ticker as a slice.
    #[inline]
    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }

    /// Returns the ticker as a fixed-size array.
    #[inline]
    pub fn as_bytes(&self) -> &[u8; TICKER_LEN] {
        &(self.0)
    }

    /// Returns an iterator over the ticker.
    #[inline]
    pub fn iter(&self) -> sp_std::slice::Iter<'_, u8> {
        self.0.iter()
    }

    /// Returns truncated ticker.
    pub fn from_slice_truncated(s: &[u8]) -> Self {
        let len = s.len();
        let mut inner = [0u8; TICKER_LEN];
        if len >= TICKER_LEN {
            // return a ticker with the first TICKER_LEN bytes.
            inner[..TICKER_LEN].copy_from_slice(&s[..TICKER_LEN]);
            Ticker(inner)
        } else {
            inner[..len].copy_from_slice(s);
            Ticker(inner)
        }
    }
}

#[cfg(feature = "runtime-benchmarks")]
impl Ticker {
    /// Create ticker by repeating `b` for `TICKER_LEN`
    pub const fn repeating(b: u8) -> Ticker {
        // TODO: replace with u8::to_ascii_uppercase when it's const
        const fn to_ascii_uppercase(b: u8) -> u8 {
            b & !((b.is_ascii_lowercase() as u8) << 5)
        }
        Self([to_ascii_uppercase(b); TICKER_LEN])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialization_deserialization_test() {
        let ticker_name: Vec<u8> = (vec![0x45, 0x32, 0x43]).into();
        let ticker = Ticker::from_slice_truncated(ticker_name.as_slice());
        let serialize = serde_json::to_string(&ticker).unwrap();
        let serialize_data = "\"0x453243000000000000000000\"";
        assert_eq!(serialize_data, serialize);
        let deserialize = serde_json::from_str::<Ticker>(&serialize).unwrap();
        assert_eq!(ticker, deserialize);
    }

    #[test]
    fn ticker_test() {
        // 1. Happy path.
        let s1 = b"ABCDABCDABCD";
        let t1 = Ticker::from_slice_truncated(&s1[..]);
        assert_eq!(t1.len(), 12);
        assert_eq!(t1.as_slice(), b"ABCDABCDABCD");
    }

    #[test]
    fn parity_scale_codec() {
        let s = b"ACME";
        let t = Ticker::from_slice_truncated(&s[..]);

        let t_encoded = t.encode();
        let t2 = Ticker::decode(&mut &t_encoded[..]).unwrap();

        assert_eq!(t, t2);
    }
}
