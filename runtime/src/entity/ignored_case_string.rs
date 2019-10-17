use rstd::prelude::Vec;

#[derive(codec::Encode, codec::Decode, Default, Clone, Debug)]
pub struct IgnoredCaseString(Vec<u8>);

impl IgnoredCaseString {
    fn as_vec(&self) -> &Vec<u8> {
        &self.0
    }
}

impl PartialEq for IgnoredCaseString {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_slice().eq_ignore_ascii_case(other.0.as_slice())
    }
}

impl From<&[u8]> for IgnoredCaseString {
    fn from(s: &[u8]) -> Self {
        IgnoredCaseString(s.to_ascii_uppercase())
    }
}

impl From<&str> for IgnoredCaseString {
    fn from(s: &str) -> Self {
        IgnoredCaseString(s.to_ascii_uppercase().as_bytes().to_vec())
    }
}

impl From<Vec<u8>> for IgnoredCaseString {
    fn from(v: Vec<u8>) -> Self {
        IgnoredCaseString(v)
    }
}
