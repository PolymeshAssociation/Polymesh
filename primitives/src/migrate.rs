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

//! Defines a trait and implementations for storage migration.

use codec::{Decode, Encode};
use frame_support::migration::{put_storage_value, StorageIterator};
use sp_std::vec::Vec;

/// A data type which is migrating through `migrate` to a new type as defined by `Into`.
pub trait Migrate: Decode {
    /// The new type to migrate into.
    type Into: Encode;

    /// Migrate the current type to `Into` if possible.
    ///
    /// For simplicity, we assume that migrations are fallible in the type system,
    /// although they may in fact not be for certain data types.
    fn migrate(self) -> Option<Self::Into>;
}

impl<T: Migrate> Migrate for Vec<T> {
    type Into = Vec<T::Into>;

    fn migrate(self) -> Option<Self::Into> {
        // Heuristic: let's assume migration is successful for everything.
        let mut vec = Vec::with_capacity(self.len());
        for old in self {
            vec.push(old.migrate()?);
        }
        Some(vec)
    }
}

/// Migrate the values with old type `T` in `module::item` to `T::Into`.
///
/// Migrations resulting in `old.migrate() == None` are silently dropped from storage.
pub fn migrate_map<T: Migrate>(module: &[u8], item: &[u8]) {
    StorageIterator::<T>::new(module, item)
        .drain()
        .filter_map(|(key, old)| Some((key, old.migrate()?)))
        .for_each(|(key, new)| put_storage_value(module, item, &key, new));
}
