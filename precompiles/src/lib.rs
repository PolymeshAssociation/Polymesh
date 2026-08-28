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

// Import the fungible asset interface. Generates:
//   - `IFungibleAsset::IFungibleAssetCalls` enum
//   - `IFungibleAsset::IFungibleAssetEvents` enum
alloy_core::sol! {
    #[sol(all_derives)]
    "src/interfaces/FungibleAssetStub.sol"
}

// Import the non-fungible asset interface. Generates:
//   - `INonFungibleAsset::INonFungibleAssetCalls` enum
//   - `INonFungibleAsset::INonFungibleAssetEvents` enum
alloy_core::sol! {
    #[sol(all_derives)]
    "src/interfaces/NonFungibleAssetStub.sol"
}

pub use IFungibleAsset::{IFungibleAssetCalls, IFungibleAssetEvents};
pub use INonFungibleAsset::{INonFungibleAssetCalls, INonFungibleAssetEvents};

/// Runtime bytecode for explorer verification of the fungible asset precompile.
pub const FUNGIBLE_ASSET_CODE: &[u8] = include_bytes!("interfaces/FungibleAssetStub.bin");

/// Runtime bytecode for explorer verification of the non-fungible asset precompile.
pub const NON_FUNGIBLE_ASSET_CODE: &[u8] = include_bytes!("interfaces/NonFungibleAssetStub.bin");

// Import the general-purpose Polymesh runtime interface. Generates:
//   - `IPolymeshRuntime::IPolymeshRuntimeCalls` enum
//   - `IPolymeshRuntime::IPolymeshRuntimeEvents` enum
alloy_core::sol! {
    #[sol(all_derives)]
    "src/interfaces/PolymeshRuntime.sol"
}

pub use IPolymeshRuntime::{IPolymeshRuntimeCalls, IPolymeshRuntimeEvents};
