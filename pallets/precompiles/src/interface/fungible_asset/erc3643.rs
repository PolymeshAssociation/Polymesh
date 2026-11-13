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

use alloc::string::String;
use alloc::vec::Vec;

use frame_support::traits::Get;
use pallet_asset::{AssetIdTicker, WeightInfo};
use pallet_revive::precompiles::alloy::primitives::{keccak256, Address};
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetId, AssetName};
use polymesh_primitives::ticker::TICKER_LEN;
use polymesh_primitives::Ticker;

use crate::common::{extrinsic_error, Common};
use crate::interface::FungibleAssetInterface;
use crate::interface::{DECIMALS, ERR_INVALID_SYMBOL};
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Freezes the asset, preventing token transfers. Only an agent of the token can call this function.
    pub(crate) fn pause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::freeze { asset_id },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Paused(IFungibleAsset::Paused {
                userAddress: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }

    /// Unfreezes the token contract, allowing token transfers. Only an agent of the token can call this function.
    pub(crate) fn unpause(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::unfreeze { asset_id },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Unpaused(IFungibleAsset::Unpaused {
                userAddress: caller.address.0.into(),
            }),
        )?;
        Ok(Vec::new())
    }

    /// Returns `true` if the asset is frozen, which is what `pause` does.
    pub(crate) fn paused(asset_id: AssetId, env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let paused = pallet_asset::Frozen::<T>::get(asset_id);

        Ok(IFungibleAsset::pausedCall::abi_encode_returns(&paused))
    }

    /// Returns `true` if the account is frozen for this asset, which is what `setAddressFrozen`
    /// sets. A frozen account cannot send; an amount frozen with `freezePartialTokens` or
    /// `setFrozenTokens` is not reported here but by `getFrozenTokens`.
    pub(crate) fn is_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::isFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let holder = Common::<T>::asset_holder(env, call.account)?;
        let frozen =
            pallet_asset::Pallet::<T>::ensure_holder_is_not_frozen(&holder, &asset_id).is_err();

        Ok(IFungibleAsset::isFrozenCall::abi_encode_returns(&frozen))
    }

    /// Sets the token name. Only the owner of the token contract can call this function.
    pub(crate) fn set_name(
        asset_id: AssetId,
        call: &IFungibleAsset::setNameCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let new_asset_name = AssetName::from(&call.name.as_bytes().to_vec());

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::rename_asset {
                asset_id,
                asset_name: new_asset_name.clone(),
            },
        )?;

        Self::deposit_token_information(asset_id, env)?;
        Ok(Vec::new())
    }

    /// Sets the token symbol. Only the owner of the token contract can call this function.
    ///
    /// Registers the new ticker and links it to the asset. A previously linked ticker is unlinked.
    pub(crate) fn set_symbol(
        asset_id: AssetId,
        call: &IFungibleAsset::setSymbolCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let new_symbol = &call.symbol.as_bytes();

        if new_symbol.len() > TICKER_LEN {
            return Err(Error::Revert(Revert {
                reason: ERR_INVALID_SYMBOL.into(),
            }));
        }

        let ticker = Ticker::from_slice_truncated(new_symbol);
        let caller = Common::<T>::caller(env)?;

        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;
        let current_ticker = AssetIdTicker::<T>::get(asset_id);

        if current_ticker == Some(ticker) {
            // Nothing to change, but the caller must still be allowed to call it.
            env.charge(<T as pallet_asset::Config>::WeightInfo::link_ticker_to_asset_id())?;
            // Sets the call metadata but only runs the permission check, not the actual extrinsic.
            Common::<T>::with_runtime_call(
                env,
                caller.account_id.clone(),
                pallet_asset::Call::<T>::link_ticker_to_asset_id { ticker, asset_id },
                || {
                    pallet_external_agents::Pallet::<T>::ensure_perms(
                        caller.runtime_origin(),
                        &asset_id,
                    )
                },
            )?
            .map_err(extrinsic_error)?;

            // ERC-3643 emits the event on every successful call, changed or not
            Self::deposit_token_information(asset_id, env)?;
            return Ok(Vec::new());
        }

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::register_unique_ticker { ticker },
        )?;

        if let Some(current_ticker) = current_ticker {
            Common::<T>::call_runtime(
                env,
                caller.runtime_origin(),
                caller.account_id.clone(),
                pallet_asset::Call::<T>::unlink_ticker_from_asset_id {
                    ticker: current_ticker,
                    asset_id,
                },
            )?;
        }

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::link_ticker_to_asset_id { ticker, asset_id },
        )?;

        Self::deposit_token_information(asset_id, env)?;
        Ok(Vec::new())
    }

    /// Emits ERC-3643 `UpdatedTokenInformation` with the asset's stored name and symbol.
    ///
    /// Both are read back from storage, so the event reports what `name()` and `symbol()` now
    /// return. The name and symbol are indexed strings, which a log carries as their keccak-256
    /// hashes. The asset has no version and no ONCHAINID.
    fn deposit_token_information(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<(), Error> {
        let name = Self::asset_name(asset_id, env)?;
        let symbol = Self::asset_symbol(asset_id, env)?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::UpdatedTokenInformation(
                IFungibleAsset::UpdatedTokenInformation {
                    newName: keccak256(name.as_bytes()),
                    newSymbol: keccak256(symbol.as_bytes()),
                    newDecimals: DECIMALS,
                    newVersion: String::new(),
                    newOnchainID: Address::ZERO,
                },
            ),
        )
    }

    /// Sets the frozen status of a specific address. Only an agent of the token can call this function.
    pub(crate) fn set_address_frozen(
        asset_id: AssetId,
        call: &IFungibleAsset::setAddressFrozenCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::asset_holder(env, call.account)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::set_holder_frozen {
                asset_holder: acc_to_freeze,
                asset_id,
                freeze: call.freeze,
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

    /// Returns `true` if the account is an agent of the token.
    pub(crate) fn is_agent(
        asset_id: AssetId,
        call: &IFungibleAsset::isAgentCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(3))?;

        let account_id = Common::<T>::account_id(env, call.account)?;

        let is_agent = match pallet_identity::Pallet::<T>::get_identity(&account_id) {
            Some(did) => pallet_external_agents::GroupOfAgent::<T>::contains_key(asset_id, did),
            None => false,
        };

        Ok(IFungibleAsset::isAgentCall::abi_encode_returns(&is_agent))
    }

    /// Freezes an additional amount of tokens for a specific address, on top of any tokens
    /// already frozen. Only an agent of the token can call this function.
    pub(crate) fn freeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::freezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_freeze = Common::<T>::asset_holder(env, call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::freeze_partial_tokens {
                asset_id,
                asset_holder: acc_to_freeze.clone(),
                amount,
            },
        )?;

        // Read back the stored total for the ERC-7943 `Frozen` event, rather than recomputing it
        let frozen_after = Self::frozen_balance(env, &acc_to_freeze, &asset_id)?;

        // ERC-3643 emits this on every call with the requested amount, even 0, as T-REX does
        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::TokensFrozen(IFungibleAsset::TokensFrozen {
                account: call.account,
                amount: call.amount,
            }),
        )?;
        Self::deposit_frozen_total(env, call.account, frozen_after)?;
        Ok(Vec::new())
    }

    /// Unfreezes an amount of tokens for a specific address, reducing the amount currently
    /// frozen. Only an agent of the token can call this function.
    pub(crate) fn unfreeze_partial_tokens(
        asset_id: AssetId,
        call: &IFungibleAsset::unfreezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;

        let acc_to_unfreeze = Common::<T>::asset_holder(env, call.account)?;
        let amount = Common::<T>::to_balance(call.amount)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id,
            pallet_asset::Call::<T>::unfreeze_partial_tokens {
                asset_id,
                asset_holder: acc_to_unfreeze.clone(),
                amount,
            },
        )?;

        // Read back the stored total for the ERC-7943 `Frozen` event, rather than recomputing it
        let frozen_after = Self::frozen_balance(env, &acc_to_unfreeze, &asset_id)?;

        // ERC-3643 emits this on every call with the requested amount, even 0, as T-REX does
        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::TokensUnfrozen(IFungibleAsset::TokensUnfrozen {
                account: call.account,
                amount: call.amount,
            }),
        )?;
        Self::deposit_frozen_total(env, call.account, frozen_after)?;
        Ok(Vec::new())
    }
}
