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

use frame_support::dispatch::RawOrigin;
use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::primitives::{FixedBytes, U256};
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};
use pallet_revive::H160;

use pallet_asset::{FrozenBalance, WeightInfo};
use pallet_identity::Pallet as IdentityPallet;
use polymesh_primitives::asset::{AssetHolder, AssetId};
use polymesh_primitives::{AccountId as AccountId32, Balance, WeightMeter};

use crate::interface::{IPolymeshInterface, IPolymeshInterfaceEvents, PolymeshInterface};
use crate::interface::ERR_INVALID_ACCOUNT_ID;

impl<T> PolymeshInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    pub(crate) fn can_send(
        asset_id: AssetId,
        call: &IPolymeshInterface::canSendCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let account = Self::account_id_from_h160(call.account.into_array().into())?;
        let value = Self::to_balance(call.value)?;
        let result = Self::account_can_transfer(asset_id, &account, value);

        Ok(IPolymeshInterface::canSendCall::abi_encode_returns(&result))
    }

    pub(crate) fn can_receive(
        asset_id: AssetId,
        call: &IPolymeshInterface::canReceiveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let account = Self::account_id_from_h160(call.account.into_array().into())?;
        let value = Self::to_balance(call.value)?;
        let result = Self::account_can_receive(asset_id, &account, value);

        Ok(IPolymeshInterface::canReceiveCall::abi_encode_returns(&result))
    }

    pub(crate) fn can_transfer(
        asset_id: AssetId,
        call: &IPolymeshInterface::canTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let from = Self::account_id_from_h160(call.from.into_array().into())?;
        let to = Self::account_id_from_h160(call.to.into_array().into())?;
        let value = Self::to_balance(call.value)?;

        let sender = AssetHolder::try_from(from.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;
        let receiver = AssetHolder::try_from(to.encode()).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;

        let mut weight_meter = WeightMeter::max_limit_no_minimum();
        let errors = pallet_asset::Pallet::<T>::asset_transfer_report(
            &sender,
            &receiver,
            &asset_id,
            value,
            false,
            &mut weight_meter,
        );

        Ok(IPolymeshInterface::canTransferCall::abi_encode_returns(
            &errors.is_empty(),
        ))
    }

    pub(crate) fn get_frozen_tokens(
        asset_id: AssetId,
        call: &IPolymeshInterface::getFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let account = Self::account_id32_from_h160(call.account.into_array().into())?;
        let frozen_balance = FrozenBalance::<T>::get(&account, &asset_id);
        let value = Self::to_u256(frozen_balance)?;

        Ok(IPolymeshInterface::getFrozenTokensCall::abi_encode_returns(
            &value,
        ))
    }

    pub(crate) fn set_frozen_tokens(
        asset_id: AssetId,
        call: &IPolymeshInterface::setFrozenTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::set_frozen_tokens())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let target = Self::account_id32_from_h160(call.account.into_array().into())?;
        let amount = Self::to_balance(call.amount)?;

        if let Err(e) = pallet_asset::Pallet::<T>::set_frozen_tokens(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
            target,
            amount,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }

        Self::deposit_event(
            env,
            IPolymeshInterfaceEvents::Frozen(IPolymeshInterface::Frozen {
                account: call.account,
                amount: call.amount,
            }),
        )?;

        Ok(IPolymeshInterface::setFrozenTokensCall::abi_encode_returns(
            &true,
        ))
    }

    pub(crate) fn forced_transfer(
        asset_id: AssetId,
        call: &IPolymeshInterface::forcedTransferCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::forced_transfer())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let from = Self::account_id32_from_h160(call.from.into_array().into())?;
        let to = Self::account_id32_from_h160(call.to.into_array().into())?;
        let value = Self::to_balance(call.value)?;

        if let Err(e) = pallet_asset::Pallet::<T>::forced_transfer(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
            value,
            from,
            to,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }

        Self::deposit_event(
            env,
            IPolymeshInterfaceEvents::ForcedTransfer(IPolymeshInterface::ForcedTransfer {
                from: call.from,
                to: call.to,
                value: call.value,
            }),
        )?;

        Ok(IPolymeshInterface::forcedTransferCall::abi_encode_returns(&true))
    }

    pub(crate) fn freeze_partial_tokens(
        asset_id: AssetId,
        call: &IPolymeshInterface::freezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let account = Self::account_id32_from_h160(call.account.into_array().into())?;
        let delta = Self::to_balance(call.value)?;
        let current = FrozenBalance::<T>::get(&account, &asset_id);
        let target = current.saturating_add(delta);
        Self::set_frozen_tokens_impl(asset_id, account, target, env)
            .and_then(|_| Ok(IPolymeshInterface::freezePartialTokensCall::abi_encode_returns(&true)))
    }

    pub(crate) fn unfreeze_partial_tokens(
        asset_id: AssetId,
        call: &IPolymeshInterface::unfreezePartialTokensCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let account = Self::account_id32_from_h160(call.account.into_array().into())?;
        let delta = Self::to_balance(call.value)?;
        let current = FrozenBalance::<T>::get(&account, &asset_id);
        let target = current.saturating_sub(delta);
        Self::set_frozen_tokens_impl(asset_id, account, target, env)
            .and_then(|_| Ok(IPolymeshInterface::unfreezePartialTokensCall::abi_encode_returns(&true)))
    }

    pub(crate) fn pause(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::freeze())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        if let Err(e) = pallet_asset::Pallet::<T>::freeze(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }

        Ok(IPolymeshInterface::pauseCall::abi_encode_returns(&true))
    }

    pub(crate) fn unpause(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::unfreeze())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        if let Err(e) = pallet_asset::Pallet::<T>::unfreeze(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }

        Ok(IPolymeshInterface::unpauseCall::abi_encode_returns(&true))
    }

    pub(crate) fn paused(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;
        Ok(IPolymeshInterface::pausedCall::abi_encode_returns(
            &pallet_asset::Frozen::<T>::get(asset_id),
        ))
    }

    pub(crate) fn supports_interface(
        _asset_id: AssetId,
        call: &IPolymeshInterface::supportsInterfaceCall,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let interface_id = call.interfaceId;
        let supported = interface_id
            == FixedBytes::<4>::from([0x01_u8, 0xFF_u8, 0xC9_u8, 0xA7_u8])
            || interface_id
                == FixedBytes::<4>::from([0x3E_u8, 0xDB_u8, 0xB4_u8, 0xC4_u8]);
        Ok(IPolymeshInterface::supportsInterfaceCall::abi_encode_returns(
            &supported,
        ))
    }

    pub(crate) fn version(
        _asset_id: AssetId,
        _env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Ok(IPolymeshInterface::versionCall::abi_encode_returns(&U256::from(1u64)))
    }

    pub(crate) fn mint(
        asset_id: AssetId,
        call: &IPolymeshInterface::mintCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Self::issue(asset_id, &IPolymeshInterface::issueCall { value: call.value }, env)
    }

    pub(crate) fn burn(
        asset_id: AssetId,
        call: &IPolymeshInterface::burnCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Self::redeem(asset_id, &IPolymeshInterface::redeemCall { value: call.value }, env)
    }

    fn account_can_transfer(asset_id: AssetId, account: &T::AccountId, value: Balance) -> bool {
        let holder = match AssetHolder::try_from(account.encode()) {
            Ok(holder) => holder,
            Err(_) => return false,
        };

        let did = match IdentityPallet::<T>::asset_holder_did(&holder) {
            Ok(did) => did,
            Err(_) => return false,
        };

        if !IdentityPallet::<T>::is_did_active(did) {
            return false;
        }

        pallet_asset::Pallet::<T>::ensure_sufficient_balance(&holder, &asset_id, value).is_ok()
    }

    fn account_can_receive(asset_id: AssetId, account: &T::AccountId, value: Balance) -> bool {
        let holder = match AssetHolder::try_from(account.encode()) {
            Ok(holder) => holder,
            Err(_) => return false,
        };

        let did = match IdentityPallet::<T>::asset_holder_did(&holder) {
            Ok(did) => did,
            Err(_) => return false,
        };

        if !IdentityPallet::<T>::is_did_active(did) {
            return false;
        }

        let current_balance = pallet_asset::Pallet::<T>::get_holders_balance(&holder, &asset_id);
        current_balance
            .checked_add(value)
            .is_some()
            && pallet_asset::Frozen::<T>::get(asset_id) == false
    }

    fn set_frozen_tokens_impl(
        asset_id: AssetId,
        account: AccountId32,
        amount: Balance,
        env: &mut impl Ext<T = T>,
    ) -> Result<(), Error> {
        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        if let Err(e) = pallet_asset::Pallet::<T>::set_frozen_tokens(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
            account,
            amount,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }
        Ok(())
    }

    fn account_id_from_h160(account: H160) -> Result<T::AccountId, Error> {
        Ok(<T as pallet_revive::Config>::AddressMapper::to_account_id(&account))
    }

    fn account_id32_from_h160(account: H160) -> Result<AccountId32, Error> {
        let account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&account);
        let account_id: [u8; 32] = account.encode().try_into().map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_ACCOUNT_ID.into(),
            })
        })?;
        Ok(AccountId32::from(account_id))
    }
}
