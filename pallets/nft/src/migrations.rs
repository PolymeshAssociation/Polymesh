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

//! Storage migrations for `pallet-nft`.

use frame_support::migrations::VersionedMigration;
use frame_support::traits::{Get, UncheckedOnRuntimeUpgrade};
use frame_support::weights::Weight;
use sp_std::collections::btree_map::BTreeMap;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::nft::{NFTCount, NFTOwnerStatus};
use polymesh_primitives::AccountId as AccountId32;

use crate::{Config, NFTAccountCount, NFTHolder, Pallet};

/// Backfills [`NFTAccountCount`] from the existing [`NFTHolder`] entries.
pub struct BackfillNFTAccountCount<T>(core::marker::PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for BackfillNFTAccountCount<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut counts: BTreeMap<(AccountId32, AssetId), NFTCount> = BTreeMap::new();
        let mut entries_read: u64 = 0;

        for ((acc_id, asset_id, _nft_id), status) in NFTHolder::<T>::iter() {
            entries_read = entries_read.saturating_add(1);
            match status {
                NFTOwnerStatus::Owner | NFTOwnerStatus::OwnerLocked => {
                    let count = counts.entry((acc_id, asset_id)).or_default();
                    *count = count.saturating_add(1);
                }
                NFTOwnerStatus::NotOwned => {}
            }
        }

        let entries_written = counts.len() as u64;
        for ((acc_id, asset_id), count) in counts {
            // The zero-count case cannot occur here: entries are only created on increment.
            NFTAccountCount::<T>::insert(acc_id, asset_id, count);
        }

        log::info!(
            target: "runtime::nft",
            "BackfillNFTAccountCount: read {} NFTHolder entries, wrote {} NFTAccountCount entries",
            entries_read,
            entries_written,
        );

        T::DbWeight::get().reads_writes(entries_read.saturating_add(1), entries_written)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        frame_support::ensure!(
            NFTAccountCount::<T>::iter().next().is_none(),
            "NFTAccountCount is not empty before the migration"
        );
        let holdings = NFTHolder::<T>::iter()
            .filter(|(_, status)| !matches!(status, NFTOwnerStatus::NotOwned))
            .count() as u64;
        Ok(holdings.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        let expected: u64 =
            Decode::decode(&mut &state[..]).map_err(|_| "failed to decode pre_upgrade state")?;
        let total: u64 = NFTAccountCount::<T>::iter()
            .map(|(_, _, count)| count)
            .sum();
        frame_support::ensure!(
            total == expected,
            "NFTAccountCount total does not match the number of NFTHolder entries"
        );
        // Every counted entry must be non-zero.
        frame_support::ensure!(
            NFTAccountCount::<T>::iter().all(|(_, _, count)| count > 0),
            "NFTAccountCount contains a zero entry"
        );
        Ok(())
    }
}

/// Migrates `pallet-nft` storage from version 7 to version 8.
///
/// Adds [`NFTAccountCount`], backfilled from [`NFTHolder`].
pub type MigrateToV8<T> = VersionedMigration<
    7,
    8,
    BackfillNFTAccountCount<T>,
    Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
