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

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;
use sp_runtime::traits::SaturatedConversion;

use polymesh_precompiles::{IPolymeshRuntime, IPolymeshRuntimeEvents};
use polymesh_primitives::asset::{AssetName, FundingRoundName};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::common::{revert, revert_err, Common, ERR_ASSET_NOT_FOUND};
use crate::interface::PolymeshRuntimeInterface;
use crate::Config;

const ERR_TICKER_TOO_LONG: &str = "Ticker is too long";

impl<T: Config> PolymeshRuntimeInterface<T> {
    /// Creates a new asset, registering it under the caller's identity.
    pub(crate) fn create_asset(
        call: &IPolymeshRuntime::assetCreateAssetCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(3))?;

        let caller = Common::<T>::caller(env)?;

        let asset_name = AssetName(call.assetName.as_bytes().to_vec());
        let asset_type = Common::<T>::to_asset_type(&call.assetType)?;
        let asset_identifiers = call
            .assetIdentifiers
            .iter()
            .map(Common::<T>::to_asset_identifier)
            .collect::<Result<Vec<_>, Error>>()?;
        let funding_round_name = (!call.fundingRoundName.is_empty())
            .then(|| FundingRoundName(call.fundingRoundName.as_bytes().to_vec()));

        // `create_asset` doesn't return the id it generates
        let asset_id =
            pallet_asset::Pallet::<T>::generate_asset_id(caller.account_id.clone(), false);

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::create_asset {
                asset_name,
                divisible: call.divisible,
                asset_type,
                asset_identifiers,
                funding_round_name,
            },
        )?;

        let asset_details = pallet_asset::Pallet::<T>::try_get_asset_details(&asset_id)
            .map_err(|err| revert_err(err, ERR_ASSET_NOT_FOUND))?;

        Common::<T>::deposit_event(
            env,
            IPolymeshRuntimeEvents::AssetCreated(IPolymeshRuntime::AssetCreated {
                did: asset_details.owner_did.to_bytes().into(),
                assetId: asset_id.to_bytes().into(),
                assetName: call.assetName.clone(),
            }),
        )?;

        Ok(IPolymeshRuntime::assetCreateAssetCall::abi_encode_returns(
            &asset_id.to_bytes().into(),
        ))
    }

    /// Registers a ticker symbol to the caller's identity.
    pub(crate) fn register_ticker(
        call: &IPolymeshRuntime::assetRegisterTickerCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let ticker_bytes = call.ticker.as_bytes();
        if ticker_bytes.len() > TICKER_LEN {
            return Err(revert(ERR_TICKER_TOO_LONG));
        }
        let ticker = Ticker::from_slice_truncated(ticker_bytes);

        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::register_unique_ticker { ticker },
        )?;

        let registration = pallet_asset::UniqueTickerRegistration::<T>::get(&ticker)
            .ok_or_else(|| revert("Ticker registration not found after registering it"))?;

        Common::<T>::deposit_event(
            env,
            IPolymeshRuntimeEvents::TickerRegistered(IPolymeshRuntime::TickerRegistered {
                did: registration.owner.to_bytes().into(),
                ticker: call.ticker.clone(),
                expiry: registration
                    .expiry
                    .map(|moment| moment.saturated_into::<u64>())
                    .unwrap_or(0),
            }),
        )?;

        Ok(Vec::new())
    }
}
