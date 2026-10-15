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

//! Benchmarks for the Polymesh precompiles.

use alloc::format;
use alloc::vec::Vec;
use core::marker::PhantomData;

use codec::Encode;
use frame_benchmarking::v2::*;
use frame_support::traits::Get;

use pallet_asset::AssetMetadataValues;
use pallet_identity::benchmarking::UserBuilder;
use pallet_nft::Owner;
use polymesh_primitives::asset::{AssetHolder, AssetType, NonFungibleType};
use polymesh_primitives::asset_metadata::{AssetMetadataKey, AssetMetadataValue};
use polymesh_primitives::bench::create_and_issue_sample_asset;
use polymesh_primitives::nft::NFTId;

use crate::interface::nft::metadata::TOKEN_ID_PLACEHOLDER;
use crate::{Config, NonFungibleAssetInterface};

/// The precompiles are not a FRAME pallet; this type only anchors the benchmarks.
pub struct Pallet<T: Config>(PhantomData<T>);

#[benchmarks]
mod benchmarks {
    use super::*;

    #[benchmark]
    fn token_uri() {
        let alice = UserBuilder::<T>::default().generate_did().build("Alice");
        let alice_holdings = AssetHolder::try_from(alice.account().encode()).unwrap();
        let asset_id = create_and_issue_sample_asset::<T>(
            alice.account(),
            false,
            Some(AssetType::NonFungible(NonFungibleType::Invoice)),
            b"MyNFT",
            false,
        );
        pallet_nft::Pallet::<T>::create_nft_collection(
            alice.origin.clone().into(),
            Some(asset_id),
            None,
            Vec::new().into(),
        )
        .unwrap();

        // Worst case: the NFT has no tokenUri, so the collection's baseTokenUri is also read. The
        // base uri has the maximum length and is made only of placeholders, each replaced by the
        // longest possible id.
        let nft_id = NFTId(u64::MAX);
        Owner::<T>::insert(asset_id, nft_id, alice_holdings);

        let max_value_len = <T as pallet_asset::Config>::AssetMetadataValueMaxLength::get();
        let base_token_uri: Vec<u8> = TOKEN_ID_PLACEHOLDER
            .bytes()
            .cycle()
            .take(max_value_len as usize)
            .collect();
        AssetMetadataValues::<T>::insert(
            asset_id,
            AssetMetadataKey::Global(T::BaseTokenUriMetadataKey::get()),
            AssetMetadataValue(base_token_uri),
        );

        let uri;
        #[block]
        {
            uri = NonFungibleAssetInterface::<T>::resolve_token_uri(asset_id, nft_id);
        }

        assert!(uri.is_some_and(|uri| uri.contains(&format!("{}", u64::MAX))));
    }
}
