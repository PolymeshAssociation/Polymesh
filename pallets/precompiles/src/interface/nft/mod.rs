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

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::primitives::U256;
use pallet_revive::precompiles::{AddressMatcher, Error, Ext, Precompile};

use polymesh_precompiles::{INonFungibleAssetCalls, NON_FUNGIBLE_ASSET_CODE};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::nft::NFTId;

use crate::common::{revert, revert_err, Common};
use crate::Config;

mod erc721;
mod erc7943;
mod metadata;
mod polymesh_specific;

// ==================== Error Messages ====================
pub(crate) const ERR_NFT_ASSET_NOT_FOUND: &str = "Asset not found";
pub(crate) const ERR_ASSET_NOT_NON_FUNGIBLE: &str = "Asset is not non-fungible";
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

        let asset_id = Self::asset_id_from_address(address, env)?;

        match input {
            // State-changing calls - check read-only
            INonFungibleAssetCalls::transferFrom(_)
            | INonFungibleAssetCalls::safeTransferFrom_0(_)
            | INonFungibleAssetCalls::safeTransferFrom_1(_)
            | INonFungibleAssetCalls::approve(_)
            | INonFungibleAssetCalls::setApprovalForAll(_)
            | INonFungibleAssetCalls::mint(_)
            | INonFungibleAssetCalls::burn(_)
            | INonFungibleAssetCalls::forcedTransfer(_)
                if env.is_read_only() =>
            {
                Err(Common::<T>::state_change_denied())
            }

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
    /// Returns the [`AssetId`] from the address.
    pub(crate) fn asset_id_from_address(
        address: &[u8; 20],
        env: &mut impl Ext<T = T>,
    ) -> Result<AssetId, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let bytes: [u8; 16] = address[0..16].try_into().expect("slice is 16 bytes; qed");
        let asset_id = AssetId::from_raw(bytes);

        match pallet_asset::Assets::<T>::try_get(asset_id) {
            Ok(asset_details) => {
                if !asset_details.asset_type.is_non_fungible() {
                    return Err(revert(ERR_ASSET_NOT_NON_FUNGIBLE));
                }
                Ok(asset_id)
            }
            Err(err) => Err(revert_err(err, ERR_NFT_ASSET_NOT_FOUND)),
        }
    }

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
