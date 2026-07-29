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

//! Polymesh Precompiles Interfaces

#![cfg_attr(not(feature = "std"), no_std)]

use pallet_revive::precompiles::alloy;

// Import the fungible asset interface. Generates:
//   - `IFungibleAsset::IFungibleAssetCalls` enum
//   - `IFungibleAsset::IFungibleAssetEvents` enum
alloy::sol! {
    #[sol(all_derives)]
    "src/interfaces/IFungibleAsset.sol"
}

pub use IFungibleAsset::{IFungibleAssetCalls, IFungibleAssetEvents};
