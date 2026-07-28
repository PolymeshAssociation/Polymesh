use alloc::vec::Vec;

use frame_support::dispatch::RawOrigin;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Ext;
use pallet_revive::precompiles::{AddressMapper, Error};

use pallet_asset::WeightInfo;
use polymesh_primitives::asset::{AssetHolderKind, AssetId};

use crate::interface::{IPolymeshInterface, IPolymeshInterfaceEvents, PolymeshInterface};

impl<T> PolymeshInterface<T>
where
    T: pallet_revive::Config
        + pallet_asset::Config
        + pallet_asset::checkpoint::Config
        + pallet_settlement::Config,
{
    /// Mints a `value` amount of tokens to the caller's account.
    pub(crate) fn issue(
        asset_id: AssetId,
        call: &IPolymeshInterface::issueCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::issue())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let amount = Self::to_balance(call.value)?;

        if let Err(e) = pallet_asset::Pallet::<T>::issue(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
            amount,
            AssetHolderKind::Account,
        ) {
            return Err(crate::revert_dispatch_error(e.error));
        }

        Self::deposit_event(
            env,
            IPolymeshInterfaceEvents::Transfer(IPolymeshInterface::Transfer {
                from: [0u8; 20].into(),
                to: caller.0.into(),
                value: call.value,
            }),
        )?;

        Ok(IPolymeshInterface::issueCall::abi_encode_returns(&true))
    }

    /// Redeems a `value` amount of tokens from the caller's account.
    pub(crate) fn redeem(
        asset_id: AssetId,
        call: &IPolymeshInterface::redeemCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_asset::Config>::WeightInfo::redeem())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let amount = Self::to_balance(call.value)?;

        if let Err(e) = pallet_asset::Pallet::<T>::redeem(
            RawOrigin::Signed(caller_account).into(),
            asset_id,
            amount,
            AssetHolderKind::Account,
        ) {
            return Err(crate::revert_dispatch_error(e));
        }

        Self::deposit_event(
            env,
            IPolymeshInterfaceEvents::Transfer(IPolymeshInterface::Transfer {
                from: caller.0.into(),
                to: [0u8; 20].into(),
                value: call.value,
            }),
        )?;

        Ok(IPolymeshInterface::redeemCall::abi_encode_returns(&true))
    }
}
