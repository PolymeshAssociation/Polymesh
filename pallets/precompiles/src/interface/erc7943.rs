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
use codec::Encode;

use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};

use pallet_asset::WeightInfo;
use polymesh_precompiles::IFungibleAsset;
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{AssetHolder, WeightMeter};

use crate::interface::FungibleAssetInterface;
use crate::interface::ERR_INVALID_ACCOUNT_ID;

impl<T> FungibleAssetInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Checks if a transfer is possible according to token rules. It includes compliance checks.
    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &IFungibleAsset::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let transfer_report_worst_case_weight =
            <T as pallet_asset::Config>::WeightInfo::asset_transfer_report_worst_case();
        let charged = env.charge(transfer_report_worst_case_weight)?;

        let from = call.from.into_array().into();
        let from = <T as pallet_revive::Config>::AddressMapper::to_account_id(&from);
        let from = AssetHolder::try_from(from.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let to = call.to.into_array().into();
        let to = <T as pallet_revive::Config>::AddressMapper::to_account_id(&to);
        let to = AssetHolder::try_from(to.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let errors = pallet_asset::Pallet::<T>::asset_transfer_report(
            &from,
            &to,
            &asset_id,
            Self::to_balance(call.value)?,
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
}
