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
}
