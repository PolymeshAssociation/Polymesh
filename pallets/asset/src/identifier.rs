use codec::{Decode, Encode};
use core::convert::TryInto;
use sp_std::vec;

/// Implementation of common asset identifiers
/// https://www.cusip.com/identifiers.html
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq)]
pub enum Identifier {
    CUSIP([u8; 9]),
    CINS([u8; 9]),
    ISIN([u8; 12]),
    LEI([u8; 20]),
    EMPTY,
}

impl Default for Identifier {
    fn default() -> Self {
        Identifier::EMPTY
    }
}

impl Identifier {
    /// Validate `bytes` is a valid CUSIP identifier, returns an instance of `Identifier` if successful
    pub fn cusip(bytes: [u8; 9]) -> Option<Identifier> {
        (cusip_checksum(&bytes[..8]) == bytes[8] - b'0').then_some(Identifier::CUSIP(bytes))
    }

    /// Validate `bytes` is a valid CINS identifier, returns an instance of `Identifier` if successful
    pub fn cins(bytes: [u8; 9]) -> Option<Identifier> {
        Self::cusip(bytes).map(|_| Identifier::CINS(bytes))
    }

    /// Validate `bytes` is a valid ISIN identifier, returns an instance of `Identifier` if successful
    pub fn isin(bytes: [u8; 12]) -> Option<Identifier> {
        let (s1, s2) = bytes
            .iter()
            .copied()
            .map(byte_value)
            .flat_map(|b| if b > 9 { vec![b / 10, b % 10] } else { vec![b] })
            .rev()
            .enumerate()
            .fold((0, 0), |(mut s1, mut s2), (i, digit)| {
                if i % 2 == 0 {
                    s1 += digit;
                } else {
                    s2 += 2 * digit;
                    if digit >= 5 {
                        s2 -= 9;
                    }
                }
                (s1, s2)
            });
        ((s1 + s2) % 10 == 0).then_some(Identifier::ISIN(bytes))
    }

    /// Validate `bytes` is a valid LEI identifier, returns an instance of `Identifier` if successful
    pub fn lei(bytes: [u8; 20]) -> Option<Identifier> {
        bytes[..18]
            .try_into()
            .ok()
            .map(lei_checksum)
            .filter(|hash| *hash == (bytes[18] - b'0') * 10 + (bytes[19] - b'0'))
            .map(|_| Identifier::LEI(bytes))
    }

    /// Ensures the identifier is valid.
    /// Mainly used for validating manual constructions of the enum (user input).
    pub fn validate(self) -> Option<Self> {
        match self {
            Identifier::CUSIP(bytes) => Self::cusip(bytes),
            Identifier::CINS(bytes) => Self::cins(bytes),
            Identifier::ISIN(bytes) => Self::isin(bytes),
            Identifier::LEI(bytes) => Self::lei(bytes),
            Identifier::EMPTY => Some(Identifier::EMPTY),
        }
    }
}

fn cusip_checksum(bytes: &[u8]) -> u8 {
    let total: usize = bytes
        .iter()
        .copied()
        .map(byte_value)
        .enumerate()
        .map(|(i, v)| v << (i % 2))
        .map(|v| (v / 10) + v % 10)
        .map(|x| x as usize)
        .sum();
    ((10 - (total % 10)) % 10) as u8
}

fn lei_checksum(bytes: [u8; 18]) -> u8 {
    let mut i = 0;
    let total = bytes
        .iter()
        .copied()
        .rev()
        .map(byte_value)
        .map(|x| x as u128)
        .fold(0, |mut total, b| {
            total += b * 10u128.pow(i as u32);
            if b > 9 {
                i += 2;
            } else {
                i += 1;
            }
            total
        });
    (98 - (total * 100 % 97)) as u8
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

    #[test]
    fn cins() {
        assert_eq!(
            Identifier::cins(*b"S08000AA9"),
            Some(Identifier::CINS(*b"S08000AA9"))
        );
        assert_eq!(Identifier::cins(*b"S08000AA4"), None);
    }

    #[test]
    fn isin() {
        assert_eq!(
            Identifier::isin(*b"US0378331005"),
            Some(Identifier::ISIN(*b"US0378331005"))
        );
        assert_eq!(
            Identifier::isin(*b"US0004026250"),
            Some(Identifier::ISIN(*b"US0004026250"))
        );
        assert_eq!(
            Identifier::isin(*b"AU0000XVGZA3"),
            Some(Identifier::ISIN(*b"AU0000XVGZA3"))
        );
        assert_eq!(
            Identifier::isin(*b"AU0000VXGZA3"),
            Some(Identifier::ISIN(*b"AU0000VXGZA3"))
        );
        assert_eq!(
            Identifier::isin(*b"FR0000988040"),
            Some(Identifier::ISIN(*b"FR0000988040"))
        );
        assert_eq!(Identifier::isin(*b"US0373831005"), None);
    }

    #[test]
    fn lei() {
        assert_eq!(
            Identifier::lei(*b"YZ83GD8L7GG84979J516"),
            Some(Identifier::LEI(*b"YZ83GD8L7GG84979J516"))
        );
        assert_eq!(
            Identifier::lei(*b"815600306702171A6844"),
            Some(Identifier::LEI(*b"815600306702171A6844"))
        );
        assert_eq!(
            Identifier::lei(*b"549300GFX6WN7JDUSN34"),
            Some(Identifier::LEI(*b"549300GFX6WN7JDUSN34"))
        );
        assert_eq!(Identifier::lei(*b"549300GFXDSN7JDUSN34"), None);
    }
}
