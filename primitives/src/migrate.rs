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
use frame_support::migration::{put_storage_value, StorageIterator, StorageKeyIterator};
use frame_support::storage::unhashed::kill_prefix;
use frame_support::{ReversibleStorageHasher, StorageHasher, Twox128};
use sp_std::vec::Vec;

/// A data type which is migrating through `migrate` to a new type as defined by `Into`.
pub trait Migrate: Decode {
    /// The new type to migrate into.
    type Into: Encode;

    /// An external context / data source to feed into the migration process.
    /// This could e.g., be per key, or some sort of global data.
    type Context;

    /// Migrate the current type to `Into` if possible,
    /// using the given `context` as an external data source.
    ///
    /// For simplicity, we assume that migrations are fallible in the type system,
    /// although they may in fact not be for certain data types.
    fn migrate(self, context: Self::Context) -> Option<Self::Into>;
}

/// The empty context, used by default unless a context is specified.
#[derive(Copy, Clone, Default)]
pub struct Empty;

impl<T: Migrate> Migrate for Vec<T>
where
    T::Context: Clone,
{
    type Into = Vec<T::Into>;
    type Context = T::Context;

    fn migrate(self, context: Self::Context) -> Option<Self::Into> {
        // Heuristic: let's assume migration is successful for everything.
        let mut vec = Vec::with_capacity(self.len());
        for old in self {
            vec.push(old.migrate(context.clone())?);
        }
        Some(vec)
    }
}

/// Migrate the values with old type `T` in `module::item` to `T::Into`.
///
/// Migrations resulting in `old.migrate() == None` are silently dropped from storage.
pub fn migrate_map<T: Migrate, C: FnMut(&[u8]) -> T::Context>(
    module: &[u8],
    item: &[u8],
    derive_context: C,
) {
    migrate_map_rename::<T, C>(module, module, item, item, derive_context)
}

/// Migrate the values with old type `T` in `module::item` to `T::Into` and renames the module to
/// `new_module`.
///
/// Migrations mapping to `None` are silently dropped from storage.
pub fn migrate_map_rename_module<T: Migrate, C: FnMut(&[u8]) -> T::Context>(
    module: &[u8],
    new_module: &[u8],
    item: &[u8],
    derive_context: C,
) {
    migrate_map_rename::<T, C>(module, new_module, item, item, derive_context)
}

/// Migrate the values with old type `T` in `module::item` to `T::Into` in `module::new_item`.
///
/// Migrations resulting in `old.migrate() == None` are silently dropped from storage.
pub fn migrate_map_rename<T: Migrate, C: FnMut(&[u8]) -> T::Context>(
    module: &[u8],
    new_module: &[u8],
    item: &[u8],
    new_item: &[u8],
    mut derive_context: C,
) {
    StorageIterator::<T>::new(module, item)
        .drain()
        .filter_map(|(key, old)| {
            let new = old.migrate(derive_context(&key))?;
            Some((key, new))
        })
        .for_each(|(key, new)| put_storage_value(new_module, new_item, &key, new));
}

/// Migrate the key & value of a map `KO, VO` to key & value of type `KN, VN` via `map`.
/// The map is located in `module::item` and the new map will migrate to `module::new_item`.
/// This assumes that the hashers are all the same, which is the common case.
pub fn migrate_map_keys_and_value<VO, VN, H, KO, KN, F>(
    module: &[u8],
    item: &[u8],
    new_item: &[u8],
    mut map: F,
) where
    F: FnMut(KO, VO) -> Option<(KN, VN)>,
    VO: Decode + Encode,
    VN: Encode + Decode,
    H: ReversibleStorageHasher,
    KO: Decode,
    KN: Decode + Encode,
{
    StorageKeyIterator::<KO, VO, H>::new(module, item)
        .drain()
        .filter_map(|(key, val)| map(key, val))
        .for_each(|(kn, vn)| {
            let kn = kn.using_encoded(H::hash);
            let kn = kn.as_ref();
            put_storage_value(module, new_item, &kn, vn);
        })
}

/// Decode, if possible, the keys of a double map,
/// with K1 & K2 as keys, hashed with `H`, from `raw`.
pub fn decode_double_key<H: ReversibleStorageHasher, K1: Decode, K2: Decode>(
    raw: &[u8],
) -> Option<(K1, K2)> {
    let mut unhashed_key = H::reverse(&raw);
    let k1 = K1::decode(&mut unhashed_key).ok()?;
    let mut raw_k2 = H::reverse(unhashed_key);
    let k2 = K2::decode(&mut raw_k2).ok()?;
    Some((k1, k2))
}

/// Encode keys of a double map `k1` & `k2` using hasher `H`.
pub fn encode_double_key<H: StorageHasher, K1: Encode, K2: Encode>(k1: K1, k2: K2) -> Vec<u8> {
    let k1 = k1.using_encoded(H::hash);
    let k2 = k2.using_encoded(H::hash);
    let k1 = k1.as_ref();
    let k2 = k2.as_ref();
    let mut key = Vec::with_capacity(k1.len() + k2.len());
    key.extend_from_slice(k1);
    key.extend_from_slice(k2);
    key
}

/// Migrate the keys and values of a double map `K1, K2` + `V1` to
/// keys and values of type `KN1, KN2` + `V2` via `map`.
/// The `map` function may fail, in which entries are dropped silently.
/// The double map is located in `module::item`.
/// This assumes that the hashers are all the same, which is the common case.
pub fn migrate_double_map<V1, V2, H, K1, K2, KN1, KN2, F>(module: &[u8], item: &[u8], mut map: F)
where
    F: FnMut(K1, K2, V1) -> Option<(KN1, KN2, V2)>,
    V1: Decode,
    V2: Encode,
    H: ReversibleStorageHasher,
    K1: Decode,
    K2: Decode,
    KN1: Encode,
    KN2: Encode,
{
    let old_map = StorageIterator::<V1>::new(module, item)
        .drain()
        .collect::<Vec<(Vec<u8>, _)>>();

    for (key, value) in old_map.into_iter().filter_map(|(raw_key, value)| {
        let (k1, k2) = decode_double_key::<H, _, _>(&raw_key)?;
        let (kn1, kn2, value) = map(k1, k2, value)?;
        Some((encode_double_key::<H, _, _>(kn1, kn2), value))
    }) {
        put_storage_value(module, item, &key, value);
    }
}

/// Kill a storage item from a module
pub fn kill_item(module: &[u8], item: &[u8]) {
    let mut prefix = [0u8; 32];
    prefix[0..16].copy_from_slice(&Twox128::hash(module));
    prefix[16..32].copy_from_slice(&Twox128::hash(item));
    kill_prefix(&prefix)
}
