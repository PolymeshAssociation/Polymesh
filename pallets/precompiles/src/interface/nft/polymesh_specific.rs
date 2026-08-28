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

//! Polymesh-specific NFT operations: supply, issuance and redemption.

use alloc::vec::Vec;

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::primitives::{Address, U256};
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::{Error, Ext};

use pallet_nft::{CollectionAsset, CollectionKeys, CurrentNFTId, NFTsInCollection};
use polymesh_precompiles::{INonFungibleAsset, INonFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolderKind, AssetId};
use polymesh_primitives::asset_metadata::AssetMetadataValue;
use polymesh_primitives::nft::NFTMetadataAttribute;

use crate::common::{revert, Common};
use crate::interface::nft::NonFungibleAssetInterface;
use crate::Config;

// ==================== Error Messages ====================
const ERR_WRONG_NUMBER_OF_METADATA_VALUES: &str =
    "Wrong number of metadata values for this collection";
const ERR_MINT_ID_NOT_FOUND: &str = "Could not determine the id of the issued NFT";
// ========================================================

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Returns the total number of NFTs in this collection.
    pub(crate) fn total_supply(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let supply = NFTsInCollection::<T>::get(asset_id);

        Ok(INonFungibleAsset::totalSupplyCall::abi_encode_returns(
            &U256::from(supply),
        ))
    }

    /// Issues a new NFT of this collection to the caller's account key.
    ///
    /// `metadataValues` supplies one value per mandatory collection key, in the order the keys
    /// are stored for the collection.
    pub(crate) fn issue(
        asset_id: AssetId,
        call: &INonFungibleAsset::mintCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // Reads the collection id and its mandatory metadata keys.
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(2))?;

        let caller = Common::<T>::caller(env)?;
        let collection_id = CollectionAsset::<T>::get(asset_id);
        let collection_keys = CollectionKeys::<T>::get(collection_id);

        if collection_keys.len() != call.metadataValues.len() {
            return Err(revert(ERR_WRONG_NUMBER_OF_METADATA_VALUES));
        }

        let attributes: Vec<NFTMetadataAttribute> = collection_keys
            .into_iter()
            .zip(call.metadataValues.iter())
            .map(|(key, value)| NFTMetadataAttribute {
                key,
                value: AssetMetadataValue(value.to_vec()),
            })
            .collect();

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_nft::Call::<T>::issue_nft {
                asset_id,
                nft_metadata_attributes: attributes,
                holdings_kind: AssetHolderKind::Account,
            },
        )?;

        // The issued id is the collection's current id after the dispatch.
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;
        let nft_id = CurrentNFTId::<T>::get(collection_id).ok_or_else(|| {
            // Unreachable: a successful issue always advances the id sequence.
            revert(ERR_MINT_ID_NOT_FOUND)
        })?;
        let token_id = U256::from(nft_id.0);

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Transfer(INonFungibleAsset::Transfer {
                from: Address::ZERO,
                to: caller.address.0.into(),
                tokenId: token_id,
            }),
        )?;

        Ok(INonFungibleAsset::mintCall::abi_encode_returns(&token_id))
    }

    /// Redeems (burns) `tokenId` from the caller's account key.
    pub(crate) fn redeem(
        asset_id: AssetId,
        call: &INonFungibleAsset::burnCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // Reads the collection id and its mandatory metadata keys.
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(2))?;

        let caller = Common::<T>::caller(env)?;
        let nft_id = Self::nft_id(call.tokenId)?;

        // `redeem_nft` prices itself on `number_of_keys`, falling back to the worst case of
        // `MaxNumberOfCollectionKeys` (255) when it is `None`. That upfront charge is far beyond
        // a normal EVM gas limit, so pass the collection's real key count instead.
        let collection_id = CollectionAsset::<T>::get(asset_id);
        let number_of_keys = CollectionKeys::<T>::get(collection_id).len() as u8;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_nft::Call::<T>::redeem_nft {
                asset_id,
                nft_id,
                holdings_kind: AssetHolderKind::Account,
                number_of_keys: Some(number_of_keys),
            },
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Transfer(INonFungibleAsset::Transfer {
                from: caller.address.0.into(),
                to: Address::ZERO,
                tokenId: call.tokenId,
            }),
        )?;

        Ok(INonFungibleAsset::burnCall::abi_encode_returns(&true))
    }
}
