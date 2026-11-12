mod fungible_asset;
pub mod nft;
mod polymesh;

pub use fungible_asset::*;
pub use nft::*;
pub use polymesh::*;

/// ERC-165 interface id of `IERC165` itself: the selector of `supportsInterface(bytes4)`.
/// Shared by every precompile that implements `supportsInterface`.
pub const ERC165_INTERFACE_ID: [u8; 4] = [0x01, 0xff, 0xc9, 0xa7];
