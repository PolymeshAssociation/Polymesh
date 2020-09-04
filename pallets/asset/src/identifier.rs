use codec::{Decode, Encode};
use std::thread::sleep_ms;

#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Identifier {
    CUSIP([u8; 9]),
    CINS([u8; 9]),
    ISIN([u8; 12]),
    LEI([u8; 20]),
}

impl Identifier {
    pub fn cusip(bytes: [u8; 9]) -> Option<Identifier> {
        if luhn_checksum(&bytes[..8]) == bytes[8] - b'0' {
            return Some(Identifier::CUSIP(bytes));
        }
        None
    }

    pub fn cins(bytes: [u8; 9]) -> Option<Identifier> {
        unimplemented!()
    }

    pub fn isin(bytes: [u8; 12]) -> Option<Identifier> {
        if isin_checksum(&bytes[..11]) == bytes[11] - b'0' {
            return Some(Identifier::ISIN(bytes));
        }
        None
    }

    pub fn lei(bytes: [u8; 20]) -> Option<Identifier> {
        unimplemented!()
    }
}

// Luhn algorithm - https://en.wikipedia.org/wiki/Luhn_algorithm
fn luhn_checksum(bytes: &[u8]) -> u8 {
    let mut total = 0;
    for (i, b) in bytes.iter().enumerate() {
        let mut v = byte_value(*b);
        if i % 2 != 0 {
            v *= 2
        }
        total += (v / 10) + v % 10;
    }
    (10 - (total % 10)) % 10
}

fn isin_checksum(bytes: &[u8]) -> u8 {
    let mut total = 0;
    let parity = (bytes.len() - 1) % 2;
    let mut i = 0;
    for b in bytes.iter() {
        let mut v = byte_value(*b);
        if v > 9 {
            let mut v1 = v / 10;
            let mut v2 = v % 10;
            if i % 2 == parity {
                v1 *= 2;
            }
            if (i + 1) % 2 == parity {
                v2 *= 2;
            }
            if v1 > 9 {
                v1 -= 9;
            }
            if v2 > 9 {
                v2 -= 9;
            }
            total += v1 + v2;
            i += 2;
        } else {
            if i % 2 == parity {
                v *= 2
            }
            if v > 9 {
                v -= 9;
            }
            total += v;
            i += 1;
        }
    }
    (10 - (total % 10)) % 10
}

fn byte_value(b: u8) -> u8 {
    match b {
        b'*' => 36,
        b'@' => 37,
        b'#' => 38,
        b'0'..=b'9' => b - b'0',
        b'A'..=b'Z' => b - b'A' + 1 + 9,
        b'a'..=b'z' => b - 0x20 - b'A' + 1 + 9,
        _ => b,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cusip() {
        assert_eq!(
            Identifier::cusip(*b"037833100"),
            Some(Identifier::CUSIP(*b"037833100"))
        );
        assert_eq!(
            Identifier::cusip(*b"17275R102"),
            Some(Identifier::CUSIP(*b"17275R102"))
        );
        assert_eq!(
            Identifier::cusip(*b"38259P508"),
            Some(Identifier::CUSIP(*b"38259P508"))
        );
        assert_eq!(
            Identifier::cusip(*b"594918104"),
            Some(Identifier::CUSIP(*b"594918104"))
        );
        assert_eq!(Identifier::cusip(*b"68389X106"), None);
        assert_eq!(
            Identifier::cusip(*b"68389X105"),
            Some(Identifier::CUSIP(*b"68389X105"))
        );
    }

    // #[test]
    fn cins() {
        assert_eq!(
            Identifier::cins(*b"S08000AA4"),
            Some(Identifier::CINS(*b"S08000AA4"))
        );
        assert_eq!(Identifier::cins(*b"S08000AA2"), None);
    }

    #[test]
    fn isin() {
        assert_eq!(
            Identifier::isin(*b"US0378331005"),
            Some(Identifier::ISIN(*b"US0378331005"))
        );
        assert_eq!(isin_checksum(b"896101950123440000"), 1);
        assert_eq!(isin_checksum(b"950123440000"), 8);
        assert_eq!(Identifier::isin(*b"US0378331006"), None);
        assert_eq!(Identifier::isin(*b"CA0378331005"), None);
    }
}
