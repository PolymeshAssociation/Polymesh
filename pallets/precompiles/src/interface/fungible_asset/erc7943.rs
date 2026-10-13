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

use alloc::vec::Vec;

use pallet_revive::precompiles::alloy::primitives::Address;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use pallet_asset::WeightInfo;
use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolder, AssetId};
use polymesh_primitives::{Balance, WeightMeter};
use sp_runtime::Weight;

use crate::common::{revert, Common, ERR_WEIGHT_LIMIT_EXCEEDED};
use crate::interface::{
    FungibleAssetInterface, ERC165_INTERFACE_ID, ERC7943_FUNGIBLE_INTERFACE_ID,
};
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Checks if a transfer is possible according to token rules. It includes compliance checks.
    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let best_case_weight =
            <T as pallet_asset::Config>::WeightInfo::asset_transfer_report_best_case();
        let worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::asset_transfer_report_worst_case();
        let charged = env.charge(worst_case_weight)?;

        let from = Common::<T>::asset_holder(env, call.from)?;
        let to = Common::<T>::asset_holder(env, call.to)?;
        let value = Common::<T>::to_balance(call.value)?;

        // The best case covers the fixed work, the meter only gets the variable rest
        let mut weight_meter = WeightMeter::from_limit_unchecked(
            Weight::zero(),
            worst_case_weight.saturating_sub(best_case_weight),
        );
        let errors = pallet_asset::Pallet::<T>::asset_transfer_report(
            &from,
            &to,
            &asset_id,
            value,
            false,
            &mut weight_meter,
        );

        if weight_meter.limit_exceeded() {
            return Err(revert(ERR_WEIGHT_LIMIT_EXCEEDED));
        }

        let real_consumed_weight = best_case_weight.saturating_add(weight_meter.consumed());
        env.adjust_gas(charged, real_consumed_weight.min(worst_case_weight));

        Ok(IFungibleAsset::canTransferCall::abi_encode_returns(
            &errors.is_empty(),
        ))
    }

    /// Takes tokens from one address and transfers them to another.
    ///
    /// Emits `Transfer` then `ForcedTransfer`, the order of the ERC-7943 reference
    /// implementation. If the transfer had to unfreeze some of the source's tokens,
    /// `TokensUnfrozen` (ERC-3643) and `Frozen` (ERC-7943) are emitted first, as ERC-7943
    /// requires the frozen change to be logged before the transfer.
    ///
    /// Only an asset holder account is supported as the destination for now.
    pub(crate) fn forced_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::forcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let source = Common::<T>::asset_holder(env, call.from)?;
        let destination = Common::<T>::asset_holder(env, call.to)?;
        let value = Common::<T>::to_balance(call.amount)?;

        // A controller transfer unfreezes the source's tokens when its free balance is too low
        let frozen_before = Self::frozen_balance(env, &source, &asset_id)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::controller_transfer_to {
                asset_id,
                value,
                source: source.clone(),
                destination,
            },
        )?;

        // Only a source that had frozen tokens can have had some unfrozen
        if frozen_before > 0 {
            let frozen_after = Self::frozen_balance(env, &source, &asset_id)?;
            if frozen_after != frozen_before {
                Self::deposit_frozen_change(env, call.from, frozen_before, frozen_after)?;
            }
        }

        // Same order as the ERC-7943 reference implementation: the base `Transfer` first
        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Transfer(IFungibleAsset::Transfer {
                from: call.from,
                to: call.to,
                value: call.amount,
            }),
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::ForcedTransfer(IFungibleAsset::ForcedTransfer {
                from: call.from,
                to: call.to,
                amount: call.amount,
            }),
        )?;

        Ok(IFungibleAsset::forcedTransferCall::abi_encode_returns(
            &true,
        ))
    }

    /// Freezes a specific amount of tokens for a given account.
    ///
    /// Always emits `Frozen` with the new total, even when it is unchanged, as ERC-7943 requires.
    /// So ERC-3643 consumers see the change too, `TokensFrozen` or `TokensUnfrozen` with the
    /// difference is emitted first when there is one.
    pub(crate) fn set_frozen_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::setFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let acc_to_freeze = Common::<T>::asset_holder(env, call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        let frozen_before = Self::frozen_balance(env, &acc_to_freeze, &asset_id)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::set_frozen_tokens {
                asset_id,
                asset_holder: acc_to_freeze,
                amount,
            },
        )?;

        // ERC-7943 requires `Frozen` on every call, even when the amount is unchanged
        Self::deposit_frozen_change(env, call.account, frozen_before, amount)?;

        Ok(IFungibleAsset::setFrozenTokensCall::abi_encode_returns(
            &true,
        ))
    }

    /// Returns the amount of frozen tokens for a given account.
    pub(crate) fn get_frozen_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::getFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let from = Common::<T>::asset_holder(env, call.account)?;

        let frozen_tokens = Self::frozen_balance(env, &from, &asset_id)?;
        let frozen_tokens = Common::<T>::to_u256(frozen_tokens)?;

        Ok(IFungibleAsset::getFrozenTokensCall::abi_encode_returns(
            &frozen_tokens,
        ))
    }

    /// Returns `true` if the account is allowed to send tokens according to token rules.
    pub(crate) fn can_send(
        asset_id: AssetId,
        call: &IFungibleAsset::canSendCall,
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

        Ok(IFungibleAsset::canSendCall::abi_encode_returns(&allowed))
    }

    /// Returns `true` if the account is allowed to receive tokens according to token rules.
    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &IFungibleAsset::canReceiveCall,
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

        Ok(IFungibleAsset::canReceiveCall::abi_encode_returns(&allowed))
    }

    /// Reads `holder`'s frozen balance of `asset_id`, charging the weight of the read.
    ///
    /// Every frozen-balance read in this precompile goes through here, so none can skip the charge.
    pub(crate) fn frozen_balance(
        env: &mut impl Ext<T = T>,
        holder: &AssetHolder,
        asset_id: &AssetId,
    ) -> Result<Balance, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::get_holders_frozen_balance())?;
        Ok(pallet_asset::Pallet::<T>::get_holders_frozen_balance(
            holder, asset_id,
        ))
    }

    /// Logs a change of `account`'s frozen tokens from `before` to `after`, for both standards.
    ///
    /// Emits `TokensFrozen` or `TokensUnfrozen` (ERC-3643) with the difference, when there is
    /// one, then `Frozen` (ERC-7943) with the new total. `Frozen` is always emitted.
    ///
    /// For calls whose ERC-3643 event reports the requested amount on every call
    /// (`freezePartialTokens`/`unfreezePartialTokens`), emit that event and then use
    /// [`Self::deposit_frozen_total`] instead.
    pub(crate) fn deposit_frozen_change(
        env: &mut impl Ext<T = T>,
        account: Address,
        before: Balance,
        after: Balance,
    ) -> Result<(), Error> {
        if after > before {
            Common::<T>::deposit_event(
                env,
                IFungibleAssetEvents::TokensFrozen(IFungibleAsset::TokensFrozen {
                    account,
                    amount: Common::<T>::to_u256(after - before)?,
                }),
            )?;
        } else if after < before {
            Common::<T>::deposit_event(
                env,
                IFungibleAssetEvents::TokensUnfrozen(IFungibleAsset::TokensUnfrozen {
                    account,
                    amount: Common::<T>::to_u256(before - after)?,
                }),
            )?;
        }

        Self::deposit_frozen_total(env, account, after)
    }

    /// Emits the ERC-7943 `Frozen` event with `account`'s new frozen `total`.
    pub(crate) fn deposit_frozen_total(
        env: &mut impl Ext<T = T>,
        account: Address,
        total: Balance,
    ) -> Result<(), Error> {
        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Frozen(IFungibleAsset::Frozen {
                account,
                amount: Common::<T>::to_u256(total)?,
            }),
        )
    }

    /// Returns whether `interfaceId` is one of the interfaces this precompile implements.
    pub(crate) fn supports_interface(
        call: &IFungibleAsset::supportsInterfaceCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let id = call.interfaceId.0;
        let supported = id == ERC165_INTERFACE_ID || id == ERC7943_FUNGIBLE_INTERFACE_ID;

        Ok(IFungibleAsset::supportsInterfaceCall::abi_encode_returns(
            &supported,
        ))
    }
}
