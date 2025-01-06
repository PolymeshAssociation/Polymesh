// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) Polymesh Association

use codec::{Decode, Encode};
use frame_support::{
    storage::migration::take_storage_value,
    traits::{GetStorageVersion, PalletInfoAccess, StorageVersion},
    weights::Weight,
};

/// Old Polymesh storage version type.
#[derive(Encode, Decode)]
#[derive(Debug, Clone, Copy, Eq, PartialEq, PartialOrd, Ord)]
pub struct OldVersion(pub u8);

/// Helper function to migrate an instance pallet from Frame v1 to v2.
pub fn frame_v2_instance_migrate<P: PalletInfoAccess + GetStorageVersion, I: 'static>(
    old_storage_name: &str,
    target_version: StorageVersion,
) -> Weight {
    let instance_name = core::any::type_name::<I>();
    if let Some((_, old_instance_name)) = instance_name.rsplit_once("::") {
        let old_pallet_name =
            scale_info::prelude::format!("{}{}", old_instance_name, old_storage_name);
        frame_v2_migrate::<P>(&old_pallet_name, target_version)
    } else {
        log::warn!(
            "Failed to extract instance name for pallet: P={}, I={}",
            P::name(),
            instance_name
        );
        Weight::zero()
    }
}

/// Helper function to migrate a pallet from Frame v1 to v2.
///
/// # Arguments
/// * `old_pallet_name` - The old storage prefix of the pallet
/// * `version` - The target storage version
///
/// # Returns
/// * `Weight` - The weight consumed by the migration
pub fn frame_v2_migrate<P: PalletInfoAccess + GetStorageVersion>(
    old_pallet_name: &str,
    target_version: StorageVersion,
) -> Weight {
    let mut weight = Weight::zero();
    let pallet_name = P::name();
    // Check if we need to perform the migration
    let current_version = <P as GetStorageVersion>::on_chain_storage_version();
    if current_version >= target_version {
        // No migration needed
        return weight;
    }
    log::info!(
        "Migrating storage for pallet {} from version {:?} to version {:?}",
        pallet_name,
        current_version,
        target_version
    );
    weight += Weight::from_parts(1_000, 0);
    // Update the storage version
    target_version.put::<P>();

    // Remove old `StorageVersion` item.
    const STORAGE_VERSION_KEY: &[u8] = b"StorageVersion";
    if let Some(OldVersion(old_version)) =
        take_storage_value(old_pallet_name.as_bytes(), STORAGE_VERSION_KEY, &[])
    {
        log::info!(
            "Removing old `StorageVersion` item from pallet {}, old version was {}",
            pallet_name,
            old_version
        );
    }

    // If pallet names differ, migrate the storage
    if old_pallet_name != pallet_name {
        log::info!(
            "Migrating pallet from prefix '{}' to prefix '{}'",
            old_pallet_name,
            pallet_name
        );
        // Move the pallet storage from the old prefix to the new one
        frame_support::storage::migration::move_pallet(
            old_pallet_name.as_bytes(),
            pallet_name.as_bytes(),
        );
        weight += Weight::from_parts(100_000_000, 0);
    }

    weight
}
