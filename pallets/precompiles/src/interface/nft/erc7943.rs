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

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::{Error, Ext};

use pallet_asset::WeightInfo;
use pallet_nft::WeightInfo as NFTWeightInfo;
use polymesh_precompiles::{INonFungibleAsset, INonFungibleAssetEvents};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::nft::NFTs;
use polymesh_primitives::WeightMeter;
use sp_runtime::Weight;

use crate::common::{revert, Common, ERR_WEIGHT_LIMIT_EXCEEDED};
use crate::interface::nft::NonFungibleAssetInterface;
use crate::Config;

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Checks whether `tokenId` can currently move from `from` to `to`, compliance included.
    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &INonFungibleAsset::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let best_case_weight =
            <T as pallet_nft::Config>::WeightInfo::nft_transfer_report_best_case();
        let worst_case_weight =
            <T as pallet_nft::Config>::WeightInfo::nft_transfer_report_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let from = Common::<T>::asset_holder(env, call.from)?;
        let to = Common::<T>::asset_holder(env, call.to)?;
        let nfts = NFTs::new_unverified(asset_id, vec![nft_id]);

        // The best case covers the fixed work, the meter only gets the variable rest
        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            worst_case_weight.saturating_sub(best_case_weight),
        );
        let errors = pallet_nft::Pallet::<T>::nft_transfer_report(
            &from,
            &to,
            &nfts,
            false,
            &mut weight_meter,
        );

        if weight_meter.limit_exceeded() {
            return Err(revert(ERR_WEIGHT_LIMIT_EXCEEDED));
        }

        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());
        env.adjust_gas(charged, real_consumed_weight.min(worst_case_weight));

        Ok(INonFungibleAsset::canTransferCall::abi_encode_returns(
            &errors.is_empty(),
        ))
    }

    /// Takes `tokenId` from `from` and transfers it to `to`.
    ///
    /// Bypasses compliance and frozen checks; the caller must be an agent of the collection.
    /// If the token was frozen for `from`, the runtime unfreezes it and a `Frozen` event is
    /// emitted before `Transfer`, as ERC-7943 requires.
    ///
    /// Only an asset holder account is supported as the destination for now.
    pub(crate) fn forced_transfer(
        asset_id: AssetId,
        call: &INonFungibleAsset::forcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let nft_id = Self::nft_id(call.tokenId)?;
        let source = Common::<T>::asset_holder(env, call.from)?;
        let destination = Common::<T>::asset_holder(env, call.to)?;
        let nfts = NFTs::new_unverified(asset_id, vec![nft_id]);

        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;
        let was_frozen = pallet_nft::Pallet::<T>::is_nft_frozen(&asset_id, &nft_id, &source);

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_nft::Call::<T>::controller_transfer_to {
                nfts,
                source,
                destination,
            },
        )?;

        if was_frozen {
            Common::<T>::deposit_event(
                env,
                INonFungibleAssetEvents::Frozen(INonFungibleAsset::Frozen {
                    account: call.from,
                    tokenId: call.tokenId,
                    frozenStatus: false,
                }),
            )?;
        }

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::ForcedTransfer(INonFungibleAsset::ForcedTransfer {
                from: call.from,
                to: call.to,
                tokenId: call.tokenId,
            }),
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Transfer(INonFungibleAsset::Transfer {
                from: call.from,
                to: call.to,
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
        let best_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let sender = Common::<T>::asset_holder(env, call.account)?;

        // The best case covers the fixed work, the meter only gets the variable rest
        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            worst_case_weight.saturating_sub(best_case_weight),
        );
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &sender,
            &asset_id,
            true,
            &mut weight_meter,
        );

        if weight_meter.limit_exceeded() {
            return Err(revert(ERR_WEIGHT_LIMIT_EXCEEDED));
        }

        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());
        env.adjust_gas(charged, real_consumed_weight.min(worst_case_weight));

        Ok(INonFungibleAsset::canSendCall::abi_encode_returns(&allowed))
    }

    /// Returns `true` if the account is allowed to receive tokens of this collection.
    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &INonFungibleAsset::canReceiveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let best_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let receiver = Common::<T>::asset_holder(env, call.account)?;

        // The best case covers the fixed work, the meter only gets the variable rest
        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            worst_case_weight.saturating_sub(best_case_weight),
        );
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &receiver,
            &asset_id,
            false,
            &mut weight_meter,
        );

        if weight_meter.limit_exceeded() {
            return Err(revert(ERR_WEIGHT_LIMIT_EXCEEDED));
        }

        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());
        env.adjust_gas(charged, real_consumed_weight.min(worst_case_weight));

        Ok(INonFungibleAsset::canReceiveCall::abi_encode_returns(
            &allowed,
        ))
    }

    /// Freezes or unfreezes `tokenId` for `account`. The caller must be an agent of the
    /// collection.
    pub(crate) fn set_frozen_tokens(
        asset_id: AssetId,
        call: &INonFungibleAsset::setFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let nft_id = Self::nft_id(call.tokenId)?;
        let holder = Common::<T>::asset_holder(env, call.account)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_nft::Call::<T>::set_nfts_frozen {
                holder,
                nfts: NFTs::new_unverified(asset_id, vec![nft_id]),
                frozen: call.frozenStatus,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Frozen(INonFungibleAsset::Frozen {
                account: call.account,
                tokenId: call.tokenId,
                frozenStatus: call.frozenStatus,
            }),
        )?;

        Ok(INonFungibleAsset::setFrozenTokensCall::abi_encode_returns(
            &true,
        ))
    }

    /// Returns `true` if `tokenId` is frozen for `account`, whether or not it holds the token.
    pub(crate) fn get_frozen_tokens(
        asset_id: AssetId,
        call: &INonFungibleAsset::getFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let holder = Common::<T>::asset_holder(env, call.account)?;
        let frozen = pallet_nft::Pallet::<T>::is_nft_frozen(&asset_id, &nft_id, &holder);

        Ok(INonFungibleAsset::getFrozenTokensCall::abi_encode_returns(
            &frozen,
        ))
    }
}
