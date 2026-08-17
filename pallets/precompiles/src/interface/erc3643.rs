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
use frame_support::dispatch::RawOrigin;

use pallet_revive::precompiles::alloy::primitives::FixedBytes;
use pallet_revive::precompiles::alloy::sol_types::Revert;
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};

use pallet_asset::{AssetIdTicker, AssetNames, WeightInfo};
use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetId, AssetName};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::interface::FungibleAssetInterface;
use crate::interface::{DECIMALS, ERR_INVALID_SYMBOL};

impl<T> FungibleAssetInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Freezes the asset, preventing token transfers. Only an agent of the token can call this function.
    pub(crate) fn pause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::freeze())?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        match pallet_asset::Pallet::<T>::freeze(RawOrigin::Signed(caller_acc).into(), asset_id) {
            Ok(_) => {
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::Paused(IFungibleAsset::Paused {
                        userAddress: caller.0.into(),
                    }),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
    }

    /// Unfreezes the token contract, allowing token transfers. Only an agent of the token can call this function.
    pub(crate) fn unpause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::unfreeze())?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        match pallet_asset::Pallet::<T>::unfreeze(RawOrigin::Signed(caller_acc).into(), asset_id) {
            Ok(_) => {
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::Unpaused(IFungibleAsset::Unpaused {
                        userAddress: caller.0.into(),
                    }),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
    }

    /// Sets the token name. Only the owner of the token contract can call this function.
    pub(crate) fn set_name(
        asset_id: AssetId,
        call: &IFungibleAsset::setNameCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let asset_name_str = &call.name;
        let new_asset_name = AssetName::from(asset_name_str.as_bytes().to_vec());
        env.charge(
            <T as pallet_asset::Config>::WeightInfo::rename_asset(new_asset_name.len() as u32)
                .saturating_add(T::DbWeight::get().reads(1)),
        )?;

        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        match pallet_asset::Pallet::<T>::rename_asset(
            RawOrigin::Signed(caller_acc).into(),
            asset_id,
            new_asset_name.clone(),
        ) {
            Ok(_) => {
                let ticker = AssetIdTicker::<T>::get(&asset_id).unwrap_or_default();
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::UpdatedTokenInformation(
                        IFungibleAsset::UpdatedTokenInformation {
                            newName: FixedBytes::try_from(new_asset_name.0.as_slice())
                                .unwrap_or_default(),
                            newSymbol: FixedBytes::try_from(ticker.as_ref()).unwrap_or_default(),
                            newDecimals: DECIMALS,
                            newVersion: Default::default(),
                            newOnchainID: Default::default(),
                        },
                    ),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
    }

    /// Sets the token symbol. Only the owner of the token contract can call this function.
    pub(crate) fn set_symbol(
        asset_id: AssetId,
        call: &IFungibleAsset::setSymbolCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(
            <T as pallet_asset::Config>::WeightInfo::link_ticker_to_asset_id()
                .saturating_add(<T as pallet_asset::Config>::WeightInfo::register_unique_ticker())
                .saturating_add(T::DbWeight::get().reads(1)),
        )?;

        let new_symbol = &call.symbol.as_bytes();

        if new_symbol.len() > TICKER_LEN {
            return Err(Error::Revert(Revert {
                reason: ERR_INVALID_SYMBOL.into(),
            }));
        }

        let ticker = Ticker::from_slice_truncated(new_symbol);
        let caller = Self::caller(env)?;
        let caller_acc = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        if let Err(e) = pallet_asset::Pallet::<T>::register_unique_ticker(
            RawOrigin::Signed(caller_acc.clone()).into(),
            ticker,
        ) {
            return Err(Self::extrinsic_error(e));
        }

        match pallet_asset::Pallet::<T>::link_ticker_to_asset_id(
            RawOrigin::Signed(caller_acc).into(),
            ticker,
            asset_id,
        ) {
            Ok(_) => {
                let asset_name = AssetNames::<T>::get(&asset_id).unwrap_or_default();
                Self::deposit_event(
                    env,
                    IFungibleAssetEvents::UpdatedTokenInformation(
                        IFungibleAsset::UpdatedTokenInformation {
                            newName: FixedBytes::try_from(asset_name.0.as_slice())
                                .unwrap_or_default(),
                            newSymbol: FixedBytes::try_from(ticker.as_ref()).unwrap_or_default(),
                            newDecimals: DECIMALS,
                            newVersion: Default::default(),
                            newOnchainID: Default::default(),
                        },
                    ),
                )?;
                Ok(Vec::new())
            }
            Err(e) => Err(Self::extrinsic_error(e)),
        }
    }

    /// Sets the frozen status of a specific address. Only an agent of the token can call this function.
    pub(crate) fn set_address_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::setAddressFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::account_id(call.account);

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_asset::Call::<T>::set_address_frozen {
                asset_id,
                freeze: call.freeze,
                account: acc_to_freeze,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::AddressFrozen(IFungibleAsset::AddressFrozen {
                account: call.account,
                freeze: call.freeze,
                owner: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }
}
