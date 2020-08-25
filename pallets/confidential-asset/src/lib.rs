// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Confidential Asset Module
//!
//! The Confidential Asset module is one place to create the MERCAT security tokens on the Polymesh blockchain.

use frame_support::{decl_error, decl_event, decl_module, decl_storage};
use pallet_statistics::{self as statistics};
use polymesh_common_utilities::{
    asset::Trait as AssetTrait, balances::Trait as BalancesTrait,
    compliance_manager::Trait as ComplianceManagerTrait, identity::Trait as IdentityTrait,
};

/// The module's configuration trait.
pub trait Trait:
    frame_system::Trait
    + BalancesTrait
    + IdentityTrait
    + pallet_session::Trait
    + statistics::Trait
    + pallet_contracts::Trait
    + pallet_portfolio::Trait
{
}

decl_storage! {
    trait Store for Module<T: Trait> as Asset {
    }
}

// Public interface for this runtime module.
decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
    }
}

/// All functions in the decl_module macro become part of the public interface of the module
/// If they are there, they are accessible via extrinsics calls whether they are public or not
/// However, in the impl module section (this, below) the functions can be public and private
/// Private functions are internal to this module.
/// Public functions can be called from other modules e.g.: lock and unlock (being called from the tcr module)
/// All functions in the impl module section are not part of public interface because they are not part of the Call enum.
impl<T: Trait> Module<T> {}
