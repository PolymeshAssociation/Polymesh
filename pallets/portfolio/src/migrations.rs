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

//! Storage migrations for `pallet-portfolio`.

use frame_support::migrations::VersionedMigration;
use frame_support::traits::{Get, UncheckedOnRuntimeUpgrade};
use frame_support::weights::Weight;
use sp_std::collections::btree_map::BTreeMap;

use polymesh_primitives::asset::AssetId;
use polymesh_primitives::nft::NFTCount;
use polymesh_primitives::PortfolioId;

use crate::{Config, Pallet, PortfolioNFT, PortfolioNFTCount};

/// Backfills [`PortfolioNFTCount`] from the existing [`PortfolioNFT`] entries.
pub struct BackfillPortfolioNFTCount<T>(core::marker::PhantomData<T>);

impl<T: Config> UncheckedOnRuntimeUpgrade for BackfillPortfolioNFTCount<T> {
    fn on_runtime_upgrade() -> Weight {
        let mut counts: BTreeMap<(PortfolioId, AssetId), NFTCount> = BTreeMap::new();
        let mut entries_read: u64 = 0;

        for ((portfolio_id, asset_id, _nft_id), held) in PortfolioNFT::<T>::iter() {
            entries_read = entries_read.saturating_add(1);
            if held {
                let count = counts.entry((portfolio_id, asset_id)).or_default();
                *count = count.saturating_add(1);
            }
        }

        let entries_written = counts.len() as u64;
        for ((portfolio_id, asset_id), count) in counts {
            // The zero-count case cannot occur here: entries are only created on increment.
            PortfolioNFTCount::<T>::insert(portfolio_id, asset_id, count);
        }

        log::info!(
            target: "runtime::portfolio",
            "BackfillPortfolioNFTCount: read {} PortfolioNFT entries, wrote {} PortfolioNFTCount entries",
            entries_read,
            entries_written,
        );

        T::DbWeight::get().reads_writes(entries_read.saturating_add(1), entries_written)
    }

    #[cfg(feature = "try-runtime")]
    fn pre_upgrade() -> Result<sp_std::vec::Vec<u8>, sp_runtime::TryRuntimeError> {
        use codec::Encode;
        frame_support::ensure!(
            PortfolioNFTCount::<T>::iter().next().is_none(),
            "PortfolioNFTCount is not empty before the migration"
        );
        let holdings = PortfolioNFT::<T>::iter().filter(|(_, held)| *held).count() as u64;
        Ok(holdings.encode())
    }

    #[cfg(feature = "try-runtime")]
    fn post_upgrade(state: sp_std::vec::Vec<u8>) -> Result<(), sp_runtime::TryRuntimeError> {
        use codec::Decode;
        let expected: u64 =
            Decode::decode(&mut &state[..]).map_err(|_| "failed to decode pre_upgrade state")?;
        let total: u64 = PortfolioNFTCount::<T>::iter()
            .map(|(_, _, count)| count)
            .sum();
        frame_support::ensure!(
            total == expected,
            "PortfolioNFTCount total does not match the number of PortfolioNFT entries"
        );
        frame_support::ensure!(
            PortfolioNFTCount::<T>::iter().all(|(_, _, count)| count > 0),
            "PortfolioNFTCount contains a zero entry"
        );
        Ok(())
    }
}

/// Migrates `pallet-portfolio` storage from version 4 to version 5.
///
/// Adds [`PortfolioNFTCount`], backfilled from [`PortfolioNFT`].
pub type MigrateToV5<T> = VersionedMigration<
    4,
    5,
    BackfillPortfolioNFTCount<T>,
    Pallet<T>,
    <T as frame_system::Config>::DbWeight,
>;
