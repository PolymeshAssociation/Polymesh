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

//! Polymesh Precompiles for pallet-revive.

#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

use frame_support::dispatch::{GetDispatchInfo, PostDispatchInfo};
use frame_support::traits::{GetCallMetadata, IsType};
use frame_system::pallet_prelude::OriginFor;
use sp_runtime::traits::Dispatchable;

pub mod common;
pub mod interface;

pub use interface::FungibleAssetInterface;

/// Runtime configuration needed by the Polymesh precompiles.
pub trait Config:
    pallet_revive::Config
    + pallet_permissions::Config
    + pallet_asset::Config
    + pallet_asset::checkpoint::Config
    + pallet_settlement::Config
{
    /// The runtime's aggregated call type.
    ///
    /// Precompiles build and dispatch these so that the runtime's call filter and the
    /// secondary key permission checks see the extrinsic that is really being called.
    type RuntimeCall: Dispatchable<RuntimeOrigin = OriginFor<Self>, PostInfo = PostDispatchInfo>
        + GetDispatchInfo
        + GetCallMetadata
        + IsType<<Self as frame_system::Config>::RuntimeCall>
        + From<pallet_asset::Call<Self>>
        + From<pallet_settlement::Call<Self>>;
}

/// The runtime call type used by the precompiles.
pub type CallOf<T> = <T as Config>::RuntimeCall;
