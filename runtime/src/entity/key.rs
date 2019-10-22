use rstd::{convert::TryFrom, prelude::Vec};

/// Size of key, when it is u64
#[cfg(test)]
const KEY_SIZE: usize = 8;
#[cfg(not(test))]
const KEY_SIZE: usize = 32;

#[derive(codec::Encode, codec::Decode, Default, Eq, Clone, Debug)]
pub struct Key([u8; KEY_SIZE]);

impl Key {
    pub fn new() -> Self {
        Key([0u8; KEY_SIZE])
    }
}

impl TryFrom<Vec<u8>> for Key {
    type Error = &'static str;

    fn try_from(v: Vec<u8>) -> Result<Self, Self::Error> {
        Key::try_from(v.as_slice())
    }
}

impl TryFrom<&Vec<u8>> for Key {
    type Error = &'static str;

    fn try_from(v: &Vec<u8>) -> Result<Self, Self::Error> {
        Key::try_from(v.as_slice())
    }
}

impl TryFrom<&str> for Key {
    type Error = &'static str;

    fn try_from(s: &str) -> Result<Self, Self::Error> {
        Key::try_from(s.as_bytes())
    }
}

impl TryFrom<&[u8]> for Key {
    type Error = &'static str;

    fn try_from(s: &[u8]) -> Result<Self, Self::Error> {
        if s.len() == KEY_SIZE {
            let mut k = Key::new();
            k.0.copy_from_slice(s);
            Ok(k)
        } else {
            Err("Invalid size for a key")
        }
    }
}

impl From<[u8; KEY_SIZE]> for Key {
    fn from(s: [u8; KEY_SIZE]) -> Self {
        Key(s)
    }
}

impl PartialEq for Key {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }
}

impl PartialEq<&[u8]> for Key {
    fn eq(&self, other: &&[u8]) -> bool {
        self.0 == *other
    }
}

impl PartialEq<Vec<u8>> for Key {
    fn eq(&self, other: &Vec<u8>) -> bool {
        self.0 == other.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::{Key, KEY_SIZE};
    use std::convert::TryFrom;

    #[test]
    fn build_test() {
        let k: [u8; KEY_SIZE] = [1u8; KEY_SIZE];

        assert!(Key::try_from(k).is_ok());
        assert!(Key::try_from("ABCDABCD".as_bytes()).is_ok());
        assert!(Key::try_from("ABCDABCD".as_bytes().to_vec()).is_ok());

        assert!(Key::try_from("ABCDABCDx".as_bytes()).is_err());
    }
}
