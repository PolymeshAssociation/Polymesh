use codec::{Decode, Encode};
use core::convert::{TryFrom, TryInto};
use scale_info::TypeInfo;
use sp_std::prelude::Vec;

/// Implementation of common asset identifiers.
/// https://www.cusip.com/identifiers.html.
#[derive(Encode, Decode, TypeInfo, Clone, Debug, PartialEq, Eq)]
pub enum AssetIdentifier {
    /// Universally recognized identifier for financial instruments.
    /// Example: Amazon.com Inc - Common Stock
    /// ISSUER ISSUE CHECK CUSIP
    /// 023135 10    6     023135106
    CUSIP([u8; 9]),
    /// The CUSIP International Numbering System.
    /// Example: Abingdon Capital PLC - Shares
    /// COUNTRY CODE ISSUER ISSUE CHECK CINS
    /// G            0052B  10    5     G0052B105
    CINS([u8; 9]),
    /// The International Securities Identification Number.
    /// Example:
    /// COUNTRY CODE LOCAL IDENTIFIER CHECK ISIN
    /// CA           008911703        4     CA0089117034
    ISIN([u8; 12]),
    /// The Legal Entity Identifier.
    /// Example: Philadelphia Cheesesteak Company
    /// LOU PREFIX ENTITY INDENTIFIER VERIFICATION ID LEI
    /// 5493       00SAMIRN1R27UP     42              549300SAMIRN1R27UP42
    LEI([u8; 20]),
    /// Financial Instrument Global Identifier https://www.omg.org/figi/index.htm.
    /// Example: Alphabet Inc - Common Stock
    /// BBG013V1S0T3
    FIGI([u8; 12]),
}

impl AssetIdentifier {
    /// Validate `bytes` is a valid CUSIP identifier, returns an instance of `Identifier` if successful.
    pub fn cusip(bytes: [u8; 9]) -> Option<AssetIdentifier> {
        validate_cusip(&bytes).then_some(AssetIdentifier::CUSIP(bytes))
    }

    /// Validate `bytes` is a valid CINS identifier, returns an instance of `Identifier` if successful.
    pub fn cins(bytes: [u8; 9]) -> Option<AssetIdentifier> {
        validate_cusip(&bytes).then_some(AssetIdentifier::CINS(bytes))
    }

    /// Validate `bytes` is a valid ISIN identifier, returns an instance of `Identifier` if successful.
    pub fn isin(bytes: [u8; 12]) -> Option<AssetIdentifier> {
        validate_isin(&bytes).then_some(AssetIdentifier::ISIN(bytes))
    }

    /// Validate `bytes` is a valid LEI identifier, returns an instance of `Identifier` if successful.
    pub fn lei(bytes: [u8; 20]) -> Option<AssetIdentifier> {
        validate_lei(&bytes).then_some(AssetIdentifier::LEI(bytes))
    }
    /// Validate `bytes` is a valid FIGI identifier, returns an instance of `Identifier` if successful.
    pub fn figi(bytes: [u8; 12]) -> Option<AssetIdentifier> {
        validate_figi(&bytes).then_some(AssetIdentifier::FIGI(bytes))
    }

    /// Returns `true` if the identifier is valid.
    ///
    /// Mainly used for validating manual constructions of the enum (user input).
    pub fn is_valid(&self) -> bool {
        match self {
            AssetIdentifier::CUSIP(bs) | AssetIdentifier::CINS(bs) => validate_cusip(bs),
            AssetIdentifier::ISIN(bs) => validate_isin(bs),
            AssetIdentifier::LEI(bs) => validate_lei(bs),
            AssetIdentifier::FIGI(bs) => validate_figi(bs),
        }
    }
}

/// Returns `b` CUSIP digit from its ascii code.
/// Returns an error if `b` is an invalid character.
fn byte_value(b: u8) -> Result<u8, &'static str> {
    match b {
        b'*' => Ok(36),
        b'@' => Ok(37),
        b'#' => Ok(38),
        b'0'..=b'9' => Ok(b - b'0'),
        b'A'..=b'Z' => Ok(b - b'A' + 1 + 9),
        b'a'..=b'z' => Ok(b - 0x20 - b'A' + 1 + 9),
        _ => Err("Invalid Character"),
    }
}

/// Returns the check digit for `bytes` by performing the Luhn algorithm.
/// Returns an error if `bytes` contains an invalid character.
fn cusip_check_digit(bytes: &[u8]) -> Result<u8, &'static str> {
    let mut sum: u32 = 0;
    for (index, byte) in bytes.iter().enumerate() {
        let mut v = byte_value(*byte)?;
        // if the index is not even, multiply value by two
        if index % 2 != 0 {
            v *= 2;
        }
        sum += ((v / 10) + (v % 10)) as u32;
    }
    Ok(((10 - (sum % 10)) % 10) as u8)
}

/// Returns `true` if `bytes` follows the  Committee on Uniform Security Identification Procedures format.
fn validate_cusip(bytes: &[u8; 9]) -> bool {
    // The last byte must be a checksum digit (0-9)
    if !bytes[8].is_ascii_digit() {
        return false;
    }

    match cusip_check_digit(&bytes[..8]) {
        Ok(check_digit) => check_digit == bytes[8] - b'0',
        Err(_) => false,
    }
}

/// Returns `true` if `bytes` is a valid International Securities Identification Number.
fn validate_isin(bytes: &[u8; 12]) -> bool {
    // Must have 2-character ISO country code (A-Z) and a checksum digit (0-9)
    if !bytes[0].is_ascii_alphabetic()
        || !bytes[1].is_ascii_alphabetic()
        || !bytes[11].is_ascii_digit()
    {
        return false;
    }

    // After the country code and before the checksum digit, a 9-character security code (A-Z, 0-9)
    for i in 2..11 {
        if !bytes[i].is_ascii_alphanumeric() {
            return false;
        }
    }

    // Get the individual digits for each byte
    let mut digits = Vec::new();
    for byte in bytes {
        match byte_value(*byte) {
            Ok(v) => {
                if v >= 10 {
                    digits.push(v / 10);
                    digits.push(v % 10);
                } else {
                    digits.push(v);
                }
            }
            Err(_) => return false,
        }
    }

    // Luhn test
    let mut s1: u32 = 0;
    let mut s2: u32 = 0;
    for (index, digit) in digits.into_iter().rev().enumerate() {
        if index % 2 == 0 {
            s1 += digit as u32;
            continue;
        }
        // Sum for the odd indexes
        s2 += 2 * digit as u32;
        if digit >= 5 {
            s2 -= 9;
        }
    }

    (s1 + s2) % 10 == 0
}

/// Returns `true` if `bytes` is a valid Legal Entity Identifier number.
fn validate_lei(bytes: &[u8; 20]) -> bool {
    // The fist 18 bytes must be the LOU ID + Entity Identifier (A-Z, 0-9)
    for i in 0..18 {
        if !bytes[i].is_ascii_alphanumeric() {
            return false;
        }
    }

    // The last two bytes must be the verification id (0-9)
    if !bytes[18].is_ascii_digit() || !bytes[19].is_ascii_digit() {
        return false;
    }

    let first_check_digit = bytes[18] - b'0';
    let second_check_digit = bytes[19] - b'0';
    let input_check = first_check_digit * 10 + second_check_digit;

    let bytes: [u8; 18] = bytes[..18].try_into().unwrap_or_default();
    Ok(input_check) == lei_checksum(bytes)
}

fn lei_checksum(bytes: [u8; 18]) -> Result<u8, &'static str> {
    let mut i: u32 = 0;
    let mut sum: u128 = 0;
    for byte in bytes.into_iter().rev() {
        let v = byte_value(byte)? as u128;

        sum += v * (10u128.wrapping_pow(i));
        i += if v > 9 { 2 } else { 1 };
    }
    Ok((98 - ((sum * 100) % 97)) as u8)
}

fn validate_figi(bytes: &[u8; 12]) -> bool {
    if bytes[2] != b'G' {
        return false;
    }

    for i in 3..11 {
        if !bytes[i].is_ascii_alphanumeric() {
            return false;
        }
    }

    if !bytes[11].is_ascii_digit() {
        return false;
    }

    match <[u8; 2]>::try_from(&bytes[..2]).as_ref() {
        // Disallowed prefixes.
        Err(_) | Ok(b"BS" | b"BM" | b"GG" | b"GB" | b"GH" | b"KY" | b"VG") => false,
        // Validate checksum.
        Ok(_) => cusip_check_digit(&bytes[..11]) == Ok(bytes[11] - b'0'),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cusip() {
        assert_eq!(
            AssetIdentifier::cusip(*b"037833100"),
            Some(AssetIdentifier::CUSIP(*b"037833100"))
        );
        assert_eq!(
            AssetIdentifier::cusip(*b"17275R102"),
            Some(AssetIdentifier::CUSIP(*b"17275R102"))
        );
        assert_eq!(
            AssetIdentifier::cusip(*b"38259P508"),
            Some(AssetIdentifier::CUSIP(*b"38259P508"))
        );
        assert_eq!(
            AssetIdentifier::cusip(*b"594918104"),
            Some(AssetIdentifier::CUSIP(*b"594918104"))
        );
        assert_eq!(AssetIdentifier::cusip(*b"68389X106"), None);
        assert_eq!(
            AssetIdentifier::cusip(*b"68389X105"),
            Some(AssetIdentifier::CUSIP(*b"68389X105"))
        );
        assert_eq!(validate_cusip(&[51, 35, 0, 0, 162, 0, 0, 0, 0]), false);
    }

    #[test]
    fn cins() {
        assert_eq!(
            AssetIdentifier::cins(*b"S08000AA9"),
            Some(AssetIdentifier::CINS(*b"S08000AA9"))
        );
        assert_eq!(AssetIdentifier::cins(*b"S08000AA4"), None);
    }

    #[test]
    fn isin() {
        assert_eq!(
            AssetIdentifier::isin(*b"US0378331005"),
            Some(AssetIdentifier::ISIN(*b"US0378331005"))
        );
        assert_eq!(
            AssetIdentifier::isin(*b"US0004026250"),
            Some(AssetIdentifier::ISIN(*b"US0004026250"))
        );
        assert_eq!(
            AssetIdentifier::isin(*b"AU0000XVGZA3"),
            Some(AssetIdentifier::ISIN(*b"AU0000XVGZA3"))
        );
        assert_eq!(
            AssetIdentifier::isin(*b"AU0000VXGZA3"),
            Some(AssetIdentifier::ISIN(*b"AU0000VXGZA3"))
        );
        assert_eq!(
            AssetIdentifier::isin(*b"FR0000988040"),
            Some(AssetIdentifier::ISIN(*b"FR0000988040"))
        );
        assert_eq!(AssetIdentifier::isin(*b"US0373831005"), None);
    }

    #[test]
    fn lei() {
        assert_eq!(
            AssetIdentifier::lei(*b"YZ83GD8L7GG84979J516"),
            Some(AssetIdentifier::LEI(*b"YZ83GD8L7GG84979J516"))
        );
        assert_eq!(
            AssetIdentifier::lei(*b"815600306702171A6844"),
            Some(AssetIdentifier::LEI(*b"815600306702171A6844"))
        );
        assert_eq!(
            AssetIdentifier::lei(*b"549300GFX6WN7JDUSN34"),
            Some(AssetIdentifier::LEI(*b"549300GFX6WN7JDUSN34"))
        );
        assert_eq!(AssetIdentifier::lei(*b"549300GFXDSN7JDUSN34"), None);
    }

    #[test]
    fn figi() {
        assert_eq!(
            AssetIdentifier::figi(*b"BBG000BLNQ16"),
            Some(AssetIdentifier::FIGI(*b"BBG000BLNQ16"))
        );
        assert_eq!(
            AssetIdentifier::figi(*b"NRG92C84SB39"),
            Some(AssetIdentifier::FIGI(*b"NRG92C84SB39"))
        );
        assert_eq!(
            AssetIdentifier::figi(*b"BBG0013YWBF3"),
            Some(AssetIdentifier::FIGI(*b"BBG0013YWBF3"))
        );
        assert_eq!(
            AssetIdentifier::figi(*b"BBG00H9NR574"),
            Some(AssetIdentifier::FIGI(*b"BBG00H9NR574"))
        );
        assert_eq!(
            AssetIdentifier::figi(*b"BBG00094DJF9"),
            Some(AssetIdentifier::FIGI(*b"BBG00094DJF9"))
        );
        assert_eq!(
            AssetIdentifier::figi(*b"BBG016V71XT0"),
            Some(AssetIdentifier::FIGI(*b"BBG016V71XT0"))
        );
        // Bad check digit.
        assert_eq!(AssetIdentifier::figi(*b"BBG00024DJF9"), None);
        // Disallowed prefix.
        assert_eq!(AssetIdentifier::figi(*b"BSG00024DJF9"), None);
        // 3rd char not G.
        assert_eq!(AssetIdentifier::figi(*b"BBB00024DJF9"), None);
    }
}
