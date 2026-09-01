// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! Polymesh ERC721 Precompile
//!
//! Routes ABI-encoded function calls to domain modules for ERC721 operations.
//!
//! Each precompile address maps to exactly one NFT collection: bytes `[0..16)` of the address are
//! the collection's [`AssetId`], and the ERC-721 `tokenId` is the on-chain [`NFTId`].

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;

use pallet_revive::precompiles::alloy::primitives::U256;
use pallet_revive::precompiles::{AddressMatcher, Error, Ext, Precompile};

use polymesh_precompiles::{INonFungibleAssetCalls, NON_FUNGIBLE_ASSET_CODE};
use polymesh_primitives::nft::NFTId;

use crate::common::{revert_err, AssetKind, Common};
use crate::Config;

mod erc721;
mod erc7943;
mod metadata;
mod polymesh_specific;

// ==================== Error Messages ====================
pub(crate) const ERR_NFT_INST_NOT_EXECUTED: &str = "Instruction was not executed; Most likely the instruction is missing an affirmation from the receiver/mediator";
pub(crate) const ERR_TOKEN_ID_OUT_OF_RANGE: &str = "Token id out of range";
pub(crate) const ERR_NFT_NOT_FOUND: &str = "NFT does not exist";
pub(crate) const ERR_OWNER_NOT_AN_ACCOUNT: &str =
    "NFT is held in a portfolio, which has no address";
pub(crate) const ERR_UNSAFE_CONTRACT_RECEIVER: &str =
    "safeTransferFrom only supports externally-owned accounts; the receiver has code. Use transferFrom";
// ========================================================

// ERC-165 interface identifiers, each the XOR of the selectors of the interface's functions.
/// `IERC165`: `supportsInterface(bytes4)`.
pub const ERC165_INTERFACE_ID: [u8; 4] = [0x01, 0xff, 0xc9, 0xa7];
/// `IERC721`.
pub const ERC721_INTERFACE_ID: [u8; 4] = [0x80, 0xac, 0x58, 0xcd];
/// `IERC721Metadata`: `name()`, `symbol()`, `tokenURI(uint256)`.
pub const ERC721_METADATA_INTERFACE_ID: [u8; 4] = [0x5b, 0x5e, 0x13, 0x9f];

/// The ERC721 precompile calls exposed by the Polymesh runtime.
pub struct NonFungibleAssetInterface<T>(PhantomData<T>);

impl<T: Config> Precompile for NonFungibleAssetInterface<T> {
    type T = T;
    type Interface = INonFungibleAssetCalls;

    const MATCHER: AddressMatcher = AddressMatcher::VarPrefix {
        id: NonZero::new(9).unwrap(),
        data_bytes: 16,
    };
    const HAS_CONTRACT_INFO: bool = false;
    const CODE: &[u8] = NON_FUNGIBLE_ASSET_CODE;

    fn call(
        address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, Error> {
        Common::<T>::ensure_direct_call(env)?;

        let asset_id = Common::<T>::asset_id_from_address(env, address, AssetKind::NonFungible)?;

        // Calls allowed in a read-only (`STATICCALL`/`eth_call`) context, i.e. exactly those
        // declared `view` in `NonFungibleAssetStub.sol`. This is a whitelist so that a call added
        // to `INonFungibleAsset` and left unclassified is *rejected* here rather than silently
        // allowed to change state; the wildcard arm means the compiler cannot warn about the
        // omission.
        if env.is_read_only() {
            match input {
                INonFungibleAssetCalls::balanceOf(_)
                | INonFungibleAssetCalls::ownerOf(_)
                | INonFungibleAssetCalls::getApproved(_)
                | INonFungibleAssetCalls::isApprovedForAll(_)
                | INonFungibleAssetCalls::name(_)
                | INonFungibleAssetCalls::symbol(_)
                | INonFungibleAssetCalls::tokenURI(_)
                | INonFungibleAssetCalls::supportsInterface(_)
                | INonFungibleAssetCalls::totalSupply(_)
                | INonFungibleAssetCalls::canTransfer(_)
                | INonFungibleAssetCalls::canSend(_)
                | INonFungibleAssetCalls::canReceive(_) => {}
                _ => return Err(Common::<T>::state_change_denied()),
            }
        }

        match input {
            // ERC721 functions
            INonFungibleAssetCalls::balanceOf(call) => Self::balance_of(asset_id, call, env),
            INonFungibleAssetCalls::ownerOf(call) => Self::owner_of(asset_id, call, env),
            INonFungibleAssetCalls::transferFrom(call) => Self::transfer_from(asset_id, call, env),
            INonFungibleAssetCalls::safeTransferFrom_0(call) => {
                Self::safe_transfer_from(asset_id, call, env)
            }
            INonFungibleAssetCalls::safeTransferFrom_1(call) => {
                Self::safe_transfer_from_with_data(asset_id, call, env)
            }
            INonFungibleAssetCalls::approve(call) => Self::approve(asset_id, call, env),
            INonFungibleAssetCalls::setApprovalForAll(call) => {
                Self::set_approval_for_all(asset_id, call, env)
            }
            INonFungibleAssetCalls::getApproved(call) => Self::get_approved(asset_id, call, env),
            INonFungibleAssetCalls::isApprovedForAll(call) => {
                Self::is_approved_for_all(asset_id, call, env)
            }

            // ERC721Metadata functions
            INonFungibleAssetCalls::name(_) => Self::name(asset_id, env),
            INonFungibleAssetCalls::symbol(_) => Self::symbol(asset_id, env),
            INonFungibleAssetCalls::tokenURI(call) => Self::token_uri(asset_id, call, env),

            // ERC165 functions
            INonFungibleAssetCalls::supportsInterface(call) => Self::supports_interface(call, env),

            // Polymesh-specific functions
            INonFungibleAssetCalls::totalSupply(_) => Self::total_supply(asset_id, env),
            INonFungibleAssetCalls::mint(call) => Self::issue(asset_id, call, env),
            INonFungibleAssetCalls::burn(call) => Self::redeem(asset_id, call, env),

            // ERC7943 functions
            INonFungibleAssetCalls::canTransfer(call) => Self::can_transfer(asset_id, call, env),
            INonFungibleAssetCalls::forcedTransfer(call) => {
                Self::forced_transfer(asset_id, call, env)
            }
            INonFungibleAssetCalls::canSend(call) => Self::can_send(asset_id, call, env),
            INonFungibleAssetCalls::canReceive(call) => Self::can_receive(asset_id, call, env),
        }
    }
}

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Converts an ERC-721 `tokenId` into an [`NFTId`].
    ///
    /// `NFTId` is a `u64`, so ids beyond `u64::MAX` cannot name an existing NFT.
    pub(crate) fn nft_id(token_id: U256) -> Result<NFTId, Error> {
        let id: u64 = token_id
            .try_into()
            .map_err(|err| revert_err(err, ERR_TOKEN_ID_OUT_OF_RANGE))?;
        Ok(NFTId(id))
    }
}
