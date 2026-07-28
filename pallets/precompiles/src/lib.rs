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

pub mod identity;
pub mod interface;

pub use identity::PolymeshIdentity;
pub use interface::PolymeshInterface;

use alloc::format;
use pallet_revive::precompiles::alloy::sol_types::Revert;
use pallet_revive::precompiles::Error;
use sp_runtime::DispatchError;

/// Convert a [`DispatchError`] into a precompile revert with a readable reason.
pub(crate) fn revert_dispatch_error(err: DispatchError) -> Error {
    let reason = match err {
        DispatchError::Module(module_error) => format!(
            "Extrinsic returned an error: {}",
            module_error.message.unwrap_or("unknown module error")
        ),
        other => format!("Extrinsic returned an error: {:?}", other),
    };
    Error::Revert(Revert { reason })
}
