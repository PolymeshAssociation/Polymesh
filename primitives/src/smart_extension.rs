use codec::{Decode, Encode};
use sp_std::prelude::Vec;

/// Smart Extension types
#[allow(missing_docs)]
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum SmartExtensionType {
    TransferManager,
    Offerings,
    Custom(Vec<u8>),
}

impl Default for SmartExtensionType {
    fn default() -> Self {
        SmartExtensionType::Custom(b"undefined".to_vec())
    }
}

/// A wrapper for a smart extension name.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmartExtensionName(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for SmartExtensionName {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        SmartExtensionName(v)
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
/// U type refers to the AccountId which will act
/// as the address of the smart extension
pub struct SmartExtension<U> {
    /// Type of the extension
    pub extension_type: SmartExtensionType,
    /// Name of extension
    pub extension_name: SmartExtensionName,
    /// AccountId of the smart extension
    pub extension_id: U,
    /// Status of the smart extension
    pub is_archive: bool,
}
