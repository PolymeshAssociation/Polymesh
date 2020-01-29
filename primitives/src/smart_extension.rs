use codec::{Decode, Encode};
use sp_std::prelude::Vec;

/// Smart Extension types
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum SmartExtensionTypes {
    TransferManager,
    Offerings,
    Custom(Vec<u8>),
}

impl Default for SmartExtensionTypes {
    fn default() -> Self {
        SmartExtensionTypes::Custom(b"undefined".to_vec())
    }
}

#[derive(Encode, Decode, Default, Clone, PartialEq, Debug)]
pub struct SmartExtension<U> {
    /// Type of the extension
    pub extension_type: SmartExtensionTypes,
    /// Name of extension
    pub extension_name: Vec<u8>,
    /// AccountId of the smart extension
    pub extension_id: U,
    /// Status of the smart extension
    pub is_archive: bool,
}
