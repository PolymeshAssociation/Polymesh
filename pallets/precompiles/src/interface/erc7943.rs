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

use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use pallet_asset::WeightInfo;
use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolderKind, AssetId};
use polymesh_primitives::WeightMeter;

use crate::common::Common;
use crate::interface::FungibleAssetInterface;
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Checks if a transfer is possible according to token rules. It includes compliance checks.
    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let transfer_report_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::asset_transfer_report_worst_case();
        let charged = env.charge(transfer_report_worst_case_weight)?;

        let from = Common::<T>::asset_holder(call.from)?;
        let to = Common::<T>::asset_holder(call.to)?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let errors = pallet_asset::Pallet::<T>::asset_transfer_report(
            &from,
            &to,
            &asset_id,
            Common::<T>::to_balance(call.value)?,
            false,
            &mut weight_meter,
        );

        let transfer_report_weight =
            <T as pallet_asset::Config>::WeightInfo::asset_transfer_report_best_case();
        let compliance_and_statistics_weight = weight_meter.consumed();
        let real_consumed_weight =
            transfer_report_weight.saturating_add(compliance_and_statistics_weight);

        if real_consumed_weight.ref_time() < transfer_report_worst_case_weight.ref_time() {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(IFungibleAsset::canTransferCall::abi_encode_returns(
            &errors.is_empty(),
        ))
    }

    /// Takes tokens from one address and transfers them to the caller's account.
    pub(crate) fn forced_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::forcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let source = Common::<T>::asset_holder(call.from)?;
        let value = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::controller_transfer {
                asset_id,
                value,
                source,
                destination_kind: AssetHolderKind::Account,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::ForcedTransfer(IFungibleAsset::ForcedTransfer {
                from: call.from.into(),
                to: caller.address.0.into(),
                amount: call.amount,
            }),
        )?;

        Ok(IFungibleAsset::forcedTransferCall::abi_encode_returns(
            &true,
        ))
    }

    /// Freezes a specific amount of tokens for a given account.
    pub(crate) fn set_frozen_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::setFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let acc_to_freeze = Common::<T>::asset_holder(call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::set_frozen_tokens {
                asset_id,
                asset_holder: acc_to_freeze,
                amount,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Frozen(IFungibleAsset::Frozen {
                account: call.account.into(),
                amount: call.amount,
            }),
        )?;

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
        env.charge(<T as pallet_asset::Config>::WeightInfo::get_holders_frozen_balance())?;

        let from = Common::<T>::asset_holder(call.account)?;

        let frozen_tokens = pallet_asset::Pallet::<T>::get_holders_frozen_balance(&from, &asset_id);
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
        let transfer_is_allowed_for_holder_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(transfer_is_allowed_for_holder_worst_case_weight)?;

        let sender = Common::<T>::asset_holder(call.account)?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &sender,
            &asset_id,
            true,
            &mut weight_meter,
        );

        let transfer_is_allowed_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let compliance_weight = weight_meter.consumed();
        let real_consumed_weight = transfer_is_allowed_weight.saturating_add(compliance_weight);

        if real_consumed_weight.ref_time()
            < transfer_is_allowed_for_holder_worst_case_weight.ref_time()
        {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(IFungibleAsset::canSendCall::abi_encode_returns(&allowed))
    }

    /// Returns `true` if the account is allowed to receive tokens according to token rules.
    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &IFungibleAsset::canReceiveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let transfer_is_allowed_for_holder_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(transfer_is_allowed_for_holder_worst_case_weight)?;

        let receiver = Common::<T>::asset_holder(call.account)?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &receiver,
            &asset_id,
            false,
            &mut weight_meter,
        );

        let transfer_is_allowed_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let compliance_weight = weight_meter.consumed();
        let real_consumed_weight = transfer_is_allowed_weight.saturating_add(compliance_weight);

        if real_consumed_weight.ref_time()
            < transfer_is_allowed_for_holder_worst_case_weight.ref_time()
        {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(IFungibleAsset::canReceiveCall::abi_encode_returns(&allowed))
    }

    /// Freezes a specific amount of tokens for a given account.
    pub(crate) fn set_frozen_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::setFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::set_frozen_tokens())?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        let acc_to_freeze = call.account.into_array().into();
        let acc_to_freeze =
            <T as pallet_revive::Config>::AddressMapper::to_account_id(&acc_to_freeze);
        let acc_to_freeze = AssetHolder::try_from(acc_to_freeze.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let amount = Self::to_balance(call.amount)?;

        if let Err(e) = pallet_asset::Pallet::<T>::set_frozen_tokens(
            RawOrigin::Signed(caller_acc).into(),
            asset_id,
            acc_to_freeze,
            amount,
        ) {
            return Err(Self::extrinsic_error(e));
        }

        Self::deposit_event(
            env,
            IFungibleAssetEvents::Frozen(IFungibleAsset::Frozen {
                account: call.account.into(),
                amount: call.amount,
            }),
        )?;

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
        env.charge(<T as pallet_asset::Config>::WeightInfo::get_holders_frozen_balance())?;

        let from = call.account.into_array().into();
        let from = <T as pallet_revive::Config>::AddressMapper::to_account_id(&from);
        let from = AssetHolder::try_from(from.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let frozen_tokens = pallet_asset::Pallet::<T>::get_holders_frozen_balance(&from, &asset_id);
        let frozen_tokens = Self::to_u256(frozen_tokens)?;

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
        let transfer_is_allowed_for_holder_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(transfer_is_allowed_for_holder_worst_case_weight)?;

        let sender_acc = call.account.into_array().into();
        let sender_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&sender_acc);
        let sender_acc = AssetHolder::try_from(sender_acc.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &sender_acc,
            &asset_id,
            true,
            &mut weight_meter,
        );

        let transfer_is_allowed_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let compliance_weight = weight_meter.consumed();
        let real_consumed_weight = transfer_is_allowed_weight.saturating_add(compliance_weight);

        if real_consumed_weight.ref_time()
            < transfer_is_allowed_for_holder_worst_case_weight.ref_time()
        {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(IFungibleAsset::canSendCall::abi_encode_returns(&allowed))
    }

    /// Returns `true` if the account is allowed to receive tokens according to token rules.
    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &IFungibleAsset::canReceiveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let transfer_is_allowed_for_holder_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_worst_case();
        let charged = env.charge(transfer_is_allowed_for_holder_worst_case_weight)?;

        let receiver_acc = call.account.into_array().into();
        let receiver_acc =
            <T as pallet_revive::Config>::AddressMapper::to_account_id(&receiver_acc);
        let receiver_acc = AssetHolder::try_from(receiver_acc.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let allowed = pallet_asset::Pallet::<T>::transfer_is_allowed_for_holder(
            &receiver_acc,
            &asset_id,
            false,
            &mut weight_meter,
        );

        let transfer_is_allowed_weight =
            <T as pallet_asset::Config>::WeightInfo::transfer_is_allowed_for_holder_best_case();
        let compliance_weight = weight_meter.consumed();
        let real_consumed_weight = transfer_is_allowed_weight.saturating_add(compliance_weight);

        if real_consumed_weight.ref_time()
            < transfer_is_allowed_for_holder_worst_case_weight.ref_time()
        {
            env.adjust_gas(charged, real_consumed_weight);
        }

        Ok(IFungibleAsset::canReceiveCall::abi_encode_returns(&allowed))
    }
}
