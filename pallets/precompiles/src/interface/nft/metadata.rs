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

//! ERC-721 metadata extension and ERC-165 introspection.

use alloc::string::String;
use alloc::vec::Vec;

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::{Error, Ext};

use pallet_asset::{AssetIdTicker, AssetMetadataValues, AssetNames};
use pallet_nft::{CollectionAsset, MetadataValue};
use polymesh_precompiles::INonFungibleAsset;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::asset_metadata::AssetMetadataKey;
use polymesh_primitives::nft::NFTId;

use crate::interface::nft::{
    NonFungibleAssetInterface, ERC165_INTERFACE_ID, ERC721_INTERFACE_ID,
    ERC721_METADATA_INTERFACE_ID,
};
use crate::Config;

/// The placeholder substituted with the token id when resolving a token URI.
const TOKEN_ID_PLACEHOLDER: &str = "{tokenId}";

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Returns the name of the collection.
    pub(crate) fn name(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let name = AssetNames::<T>::get(asset_id).unwrap_or_default();
        let name = String::from_utf8_lossy(name.0.as_ref()).into_owned();

        Ok(INonFungibleAsset::nameCall::abi_encode_returns(&name))
    }

    /// Returns the symbol (ticker) of the collection.
    pub(crate) fn symbol(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let ticker = AssetIdTicker::<T>::get(asset_id).unwrap_or_default();

        // Removes all trailing null bytes
        let trim_ticker = ticker
            .as_ref()
            .iter()
            .take_while(|&&b| b != 0)
            .copied()
            .collect::<Vec<_>>();
        let ticker = String::from_utf8_lossy(&trim_ticker).into_owned();

        Ok(INonFungibleAsset::symbolCall::abi_encode_returns(&ticker))
    }

    /// Returns the token URI for `tokenId`.
    ///
    /// Resolution order, matching the documented Polymesh semantics of these metadata types:
    ///
    /// 1. The NFT's own `tokenUri` metadata value.
    /// 2. The collection's `baseTokenUri` metadata value.
    ///
    /// In either case a literal `{tokenId}` is replaced with the decimal token id; when the
    /// placeholder is absent the id is appended. An unset URI yields an empty string.
    pub(crate) fn token_uri(
        asset_id: AssetId,
        call: &INonFungibleAsset::tokenURICall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // Worst case: collection lookup, per-NFT value, and the collection-level fallback.
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(3))?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let collection_id = CollectionAsset::<T>::get(asset_id);

        let token_uri = MetadataValue::<T>::get(
            (collection_id, nft_id),
            AssetMetadataKey::Global(T::TokenUriMetadataKey::get()),
        );

        let raw = if token_uri.0.is_empty() {
            AssetMetadataValues::<T>::get(
                asset_id,
                AssetMetadataKey::Global(T::BaseTokenUriMetadataKey::get()),
            )
            .map(|value| value.0)
            .unwrap_or_default()
        } else {
            token_uri.0
        };

        let uri = if raw.is_empty() {
            String::new()
        } else {
            Self::substitute_token_id(&String::from_utf8_lossy(&raw), nft_id)
        };

        Ok(INonFungibleAsset::tokenURICall::abi_encode_returns(&uri))
    }

    /// Returns whether `interfaceId` is one of the interfaces this precompile implements.
    pub(crate) fn supports_interface(
        call: &INonFungibleAsset::supportsInterfaceCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let id = call.interfaceId.0;
        let supported = id == ERC165_INTERFACE_ID
            || id == ERC721_INTERFACE_ID
            || id == ERC721_METADATA_INTERFACE_ID;

        Ok(INonFungibleAsset::supportsInterfaceCall::abi_encode_returns(&supported))
    }

    /// Replaces a literal `{tokenId}` in `template` with `nft_id`, appending it when absent.
    fn substitute_token_id(template: &str, nft_id: NFTId) -> String {
        let id = alloc::format!("{}", nft_id.0);
        if template.contains(TOKEN_ID_PLACEHOLDER) {
            template.replace(TOKEN_ID_PLACEHOLDER, &id)
        } else {
            alloc::format!("{}{}", template, id)
        }
    }
}
