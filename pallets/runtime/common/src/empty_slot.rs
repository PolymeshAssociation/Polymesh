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

//! # Empty Slot Module
//!
//! It is an empty module with no functionality inside.
//! This module is used *only to keep the module index in runtime* when we remove any unused module,
//! or to reserve an specific index for a future use.
#![cfg_attr(not(feature = "std"), no_std)]

use frame_support::{decl_module, decl_storage};

pub trait Trait<I>: frame_system::Trait {}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as EmptySlot {}
}

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance>
        for enum Call
        where origin: T::Origin
    {}
}
