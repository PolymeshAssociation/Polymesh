use codec::{Decode, Encode};
use sp_std::prelude::Vec;

/// Smart Extension types
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

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
/// U type refers to the AccountId which will act
/// as the address of the smart extension
pub struct SmartExtension<U> {
    /// Type of the extension
    pub extension_type: SmartExtensionType,
    /// Name of extension
    pub extension_name: Vec<u8>,
    /// AccountId of the smart extension
    pub extension_id: U,
    /// Status of the smart extension
    pub is_archive: bool,
}
