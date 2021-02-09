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

//! # Empty Module
//!
//! It is an empty module with no functionality inside.
//! This module is used to mark any module as optional, so you can enable/disable using a
//! compilation feature. We have to do in this way because `construct_runtime` macro does not
//! support `#[cfg(feature = "<something>")]`.
//!
//! # Example
//! Replace your module with the empty module base on a feature:
//!
//! ```ignore
//!
//! #[cfg(not(feature = "enamble_my_mod"))]
//! pub use polymesh_common_utilities::empty_module::*;
//!
//! #[cfg(feature = "enamble_my_mod")]
//! mod my_mod {
//!     pub trait Trait: IdentityTrait {
//!         type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
//!         type WeightInfo: WeightInfo;
//!     }
//!
//!     decl_storage! {
//!         trait Store for Module<T: Trait> as testnet {
//!             // Some storage ...
//!         }
//!     }
//!
//!     decl_module! {
//!         pub struct Module<T: Trait> for enum Call where origin: T::Origin {
//!
//!         type Error = Error<T>;
//!
//!         /// Some extrinsics
//!         #[weight = <T as Trait>::WeightInfo::my_func()]
//!         pub fn my_func( origin) {
//!             // Some code...
//!         }
//! }
//! ```
use frame_support::{decl_event, decl_module, decl_storage};
use frame_system::Trait as SystemTrait;

use sp_std::marker::PhantomData;

pub trait Trait: SystemTrait {
    type Event: From<Event<Self>> + Into<<Self as SystemTrait>::Event>;
}

decl_storage! {
    trait Store for Module<T: Trait> as EmptySlot {}
}

decl_module! {
    pub struct Module<T: Trait>
        for enum Call
        where origin: T::Origin
    {}
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as SystemTrait>::AccountId,
    {
        /// Dummy event uses just for compatibility with optional modules.
        Unused(PhantomData<AccountId>),
    }
);
