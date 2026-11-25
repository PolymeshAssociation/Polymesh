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

//! ERC-3643 batch functions.
//!
//! Each one runs the single call's handler for every element, in order, so an element is
//! checked, charged and logged exactly as the single call is. A batch is all-or-nothing: the
//! first failing element fails the call, and `pallet_revive` rolls back everything the call did.

use alloc::vec::Vec;

use frame_support::dispatch::GetDispatchInfo;
use frame_support::weights::Weight;
use pallet_revive::precompiles::{Error, Ext};

use polymesh_precompiles::IFungibleAsset;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::traits::SettlementFnTrait;

use crate::common::{call_metadata_weight, revert, Common};
use crate::interface::{FungibleAssetInterface, MAX_BATCH_SIZE};
use crate::{CallOf, Config};

// ==================== Error Messages ====================
const ERR_BATCH_LENGTH_MISMATCH: &str = "Batch arrays have different lengths";
const ERR_BATCH_TOO_LONG: &str = "Batch is longer than the maximum of 16 elements";
// ========================================================

// The message states the cap, so it must change with it
const _: () = assert!(MAX_BATCH_SIZE == 16);

impl<T: Config> FungibleAssetInterface<T> {
    /// Calls `transfer` for each element.
    pub(crate) fn batch_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::batchTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let len = Self::batch_len(&[call.toList.len(), call.amounts.len()])?;

        let fund = Fund::new(
            FundDescription::Fungible {
                asset_id,
                amount: Default::default(),
            },
            None,
        );
        let per_element =
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(None, &fund)
                .saturating_add(call_metadata_weight::<T>());
        Self::reserve_batch_weight(env, len, per_element)?;

        for (to, value) in call.toList.iter().zip(&call.amounts) {
            let transfer = IFungibleAsset::transferCall {
                to: *to,
                value: *value,
            };
            Self::transfer(asset_id, &transfer, env)?;
        }
        Ok(Vec::new())
    }

    /// Calls `forcedTransfer` for each element.
    pub(crate) fn batch_forced_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::batchForcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let len = Self::batch_len(&[call.fromList.len(), call.toList.len(), call.amounts.len()])?;

        let holder = Common::<T>::account_holder(&Common::<T>::caller(env)?.account_id)?;
        let per_element = Self::dispatch_weight(pallet_asset::Call::<T>::controller_transfer_to {
            asset_id,
            value: Default::default(),
            source: holder.clone(),
            destination: holder,
        });
        Self::reserve_batch_weight(env, len, per_element)?;

        let elements = call.fromList.iter().zip(&call.toList).zip(&call.amounts);
        for ((from, to), amount) in elements {
            let forced_transfer = IFungibleAsset::forcedTransferCall {
                from: *from,
                to: *to,
                amount: *amount,
            };
            Self::forced_transfer(asset_id, &forced_transfer, env)?;
        }
        Ok(Vec::new())
    }

    /// Calls `setAddressFrozen` for each element.
    pub(crate) fn batch_set_address_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::batchSetAddressFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let len = Self::batch_len(&[call.userAddresses.len(), call.freeze.len()])?;

        let asset_holder = Common::<T>::account_holder(&Common::<T>::caller(env)?.account_id)?;
        let per_element = Self::dispatch_weight(pallet_asset::Call::<T>::set_holder_frozen {
            asset_holder,
            asset_id,
            freeze: true,
        });
        Self::reserve_batch_weight(env, len, per_element)?;

        for (account, freeze) in call.userAddresses.iter().zip(&call.freeze) {
            let set_address_frozen = IFungibleAsset::setAddressFrozenCall {
                account: *account,
                freeze: *freeze,
            };
            Self::set_address_frozen(asset_id, &set_address_frozen, env)?;
        }
        Ok(Vec::new())
    }

    /// Calls `freezePartialTokens` for each element.
    pub(crate) fn batch_freeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::batchFreezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let len = Self::batch_len(&[call.userAddresses.len(), call.amounts.len()])?;

        let asset_holder = Common::<T>::account_holder(&Common::<T>::caller(env)?.account_id)?;
        let per_element = Self::dispatch_weight(pallet_asset::Call::<T>::freeze_partial_tokens {
            asset_id,
            asset_holder,
            amount: Default::default(),
        });
        Self::reserve_batch_weight(env, len, per_element)?;

        for (account, amount) in call.userAddresses.iter().zip(&call.amounts) {
            let freeze = IFungibleAsset::freezePartialTokensCall {
                account: *account,
                amount: *amount,
            };
            Self::freeze_partial_tokens(asset_id, &freeze, env)?;
        }
        Ok(Vec::new())
    }

    /// Calls `unfreezePartialTokens` for each element.
    pub(crate) fn batch_unfreeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::batchUnfreezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let len = Self::batch_len(&[call.userAddresses.len(), call.amounts.len()])?;

        let asset_holder = Common::<T>::account_holder(&Common::<T>::caller(env)?.account_id)?;
        let per_element = Self::dispatch_weight(pallet_asset::Call::<T>::unfreeze_partial_tokens {
            asset_id,
            asset_holder,
            amount: Default::default(),
        });
        Self::reserve_batch_weight(env, len, per_element)?;

        for (account, amount) in call.userAddresses.iter().zip(&call.amounts) {
            let unfreeze = IFungibleAsset::unfreezePartialTokensCall {
                account: *account,
                amount: *amount,
            };
            Self::unfreeze_partial_tokens(asset_id, &unfreeze, env)?;
        }
        Ok(Vec::new())
    }

    /// Returns the number of elements of a batch whose arrays have the given lengths.
    ///
    /// Reverts if the lengths differ or exceed [`MAX_BATCH_SIZE`].
    fn batch_len(lengths: &[usize]) -> Result<usize, Error> {
        let len = lengths.first().copied().unwrap_or_default();
        if lengths.iter().any(|other| *other != len) {
            return Err(revert(ERR_BATCH_LENGTH_MISMATCH));
        }
        if len > MAX_BATCH_SIZE {
            return Err(revert(ERR_BATCH_TOO_LONG));
        }
        Ok(len)
    }

    /// The weight `Common::call_runtime` charges before it dispatches `call`.
    fn dispatch_weight(call: impl Into<CallOf<T>>) -> Weight {
        let call: CallOf<T> = call.into();
        call.get_dispatch_info()
            .call_weight
            .saturating_add(call_metadata_weight::<T>())
    }

    /// Checks, before any element runs, that the gas left covers the worst case of the whole
    /// batch, so a batch that cannot finish fails without doing any work.
    ///
    /// `per_element` is the worst-case weight the single call charges up front. The reservation
    /// is released again straight away, because every element then charges its own weight, and
    /// is refunded what it did not use, exactly as the single call is.
    fn reserve_batch_weight(
        env: &mut impl Ext<T = T>,
        len: usize,
        per_element: Weight,
    ) -> Result<(), Error> {
        let reserved = env.charge(per_element.saturating_mul(len as u64))?;
        env.adjust_gas(reserved, Weight::zero());
        Ok(())
    }
}
