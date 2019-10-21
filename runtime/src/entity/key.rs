use rstd::prelude::Vec;

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

impl From<Vec<u8>> for Key {
    fn from(v: Vec<u8>) -> Self {
        Key::from(v.as_slice())
    }
}

impl From<&[u8]> for Key {
    fn from(s: &[u8]) -> Self {
        let mut k = Key::new();
        k.0.copy_from_slice(s);
        k
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
    use super::Key;

    #[test]
    #[should_panic]
    fn panic_build_test() {
        let _rk_panic = Key::from("ABCDABCDx".as_bytes());
    }
}
