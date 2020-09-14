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
use frame_support::ReversibleStorageHasher;
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
    migrate_map_rename::<T>(module, item, item)
}

/// Migrate the values with old type `T` in `module::item` to `T::Into` in `module::new_item`.
///
/// Migrations resulting in `old.migrate() == None` are silently dropped from storage.
pub fn migrate_map_rename<T: Migrate>(module: &[u8], item: &[u8], new_item: &[u8]) {
    StorageIterator::<T>::new(module, item)
        .drain()
        .filter_map(|(key, old)| Some((key, old.migrate()?)))
        .for_each(|(key, new)| put_storage_value(module, new_item, &key, new));
}

/// Migrate the keys of a double map `K1, K2` to keys of type `KN1, KN2` via `map`.
/// The double map is located in `module::item` and the value of type `V` is ontouched.
/// This assumes that the hashers are all the same, which is the common case.
pub fn migrate_double_map_keys<V, H, K1, K2, KN1, KN2, F>(module: &[u8], item: &[u8], mut map: F)
where
    F: FnMut(K1, K2) -> (KN1, KN2),
    V: Decode + Encode,
    H: ReversibleStorageHasher,
    K1: Decode,
    K2: Decode,
    KN1: Encode,
    KN2: Encode,
{
    let old_map = StorageIterator::<V>::new(module, item)
        .drain()
        .collect::<Vec<(Vec<u8>, _)>>();

    for (raw_key, value) in old_map.iter() {
        let mut unhashed_key = H::reverse(&raw_key);
        if let Ok(k1) = K1::decode(&mut unhashed_key) {
            let mut raw_k2 = H::reverse(unhashed_key);
            if let Ok(k2) = K2::decode(&mut raw_k2) {
                let (kn1, kn2) = map(k1, k2);
                let kn1 = kn1.using_encoded(H::hash);
                let kn2 = kn2.using_encoded(H::hash);
                let kn1 = kn1.as_ref();
                let kn2 = kn2.as_ref();
                let mut key = Vec::with_capacity(kn1.len() + kn2.len());
                key.extend_from_slice(kn1);
                key.extend_from_slice(kn2);
                put_storage_value(module, item, &key, value);
            }
        }
    }
}
