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

//! ERC-7943 transfer restrictions for NFT collections.

use alloc::vec;
use alloc::vec::Vec;

use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::{Error, Ext};

use pallet_asset::WeightInfo;
use pallet_nft::WeightInfo as NFTWeightInfo;
use polymesh_precompiles::{INonFungibleAsset, INonFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolderKind, AssetId};
use polymesh_primitives::nft::NFTs;
use polymesh_primitives::WeightMeter;

use crate::common::Common;
use crate::interface::nft::NonFungibleAssetInterface;
use crate::Config;

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Checks whether `tokenId` can currently move from `from` to `to`, compliance included.
    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &INonFungibleAsset::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        // `nft_transfer_report` performs the same checks as a single-NFT transfer.
        let worst_case_weight = <T as pallet_nft::Config>::WeightInfo::base_nft_transfer(1);
        let charged = env.charge(worst_case_weight)?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let from = Common::<T>::asset_holder(call.from)?;
        let to = Common::<T>::asset_holder(call.to)?;
        let nfts = NFTs::new_unverified(asset_id, vec![nft_id]);

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let errors = pallet_nft::Pallet::<T>::nft_transfer_report(
            &from,
            &to,
            &nfts,
            false,
            &mut weight_meter,
        );

        let consumed = weight_meter.consumed();
        if consumed.ref_time() < worst_case_weight.ref_time() {
            env.adjust_gas(charged, consumed);
        }

        Ok(INonFungibleAsset::canTransferCall::abi_encode_returns(
            &errors.is_empty(),
        ))
    }

    /// Takes `tokenId` from `from` and transfers it to the caller's account key.
    ///
    /// Bypasses compliance and frozen checks; the caller must be an agent of the collection.
    pub(crate) fn forced_transfer(
        asset_id: AssetId,
        call: &INonFungibleAsset::forcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let nft_id = Self::nft_id(call.tokenId)?;
        let source = Common::<T>::asset_holder(call.from)?;
        let nfts = NFTs::new_unverified(asset_id, vec![nft_id]);

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_nft::Call::<T>::controller_transfer {
                nfts,
                source,
                destination_kind: AssetHolderKind::Account,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::ForcedTransfer(INonFungibleAsset::ForcedTransfer {
                from: call.from,
                to: caller.address.0.into(),
                tokenId: call.tokenId,
            }),
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Transfer(INonFungibleAsset::Transfer {
                from: call.from,
                to: caller.address.0.into(),
                tokenId: call.tokenId,
            }),
        )?;

        Ok(INonFungibleAsset::forcedTransferCall::abi_encode_returns(
            &true,
        ))
    }

    /// Returns `true` if the account is allowed to send tokens of this collection.
    pub(crate) fn can_send(
        asset_id: AssetId,
        call: &INonFungibleAsset::canSendCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let sender = Common::<T>::asset_holder(call.account)?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &sender,
            &asset_id,
            true,
            &mut weight_meter,
        );

        let best_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());

        if real_consumed_weight.ref_time() < worst_case_weight.ref_time() {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(INonFungibleAsset::canSendCall::abi_encode_returns(&allowed))
    }

    /// Returns `true` if the account is allowed to receive tokens of this collection.
    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &INonFungibleAsset::canReceiveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let receiver = Common::<T>::asset_holder(call.account)?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &receiver,
            &asset_id,
            false,
            &mut weight_meter,
        );

        let best_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());

        if real_consumed_weight.ref_time() < worst_case_weight.ref_time() {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(INonFungibleAsset::canReceiveCall::abi_encode_returns(
            &allowed,
        ))
    }
}
