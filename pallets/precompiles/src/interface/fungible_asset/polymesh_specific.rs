use alloc::vec::Vec;

use frame_support::traits::Get;
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::Error;
use pallet_revive::precompiles::Ext;

use polymesh_precompiles::{IFungibleAsset, IFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolderKind, AssetId};

use crate::common::Common;
use crate::interface::FungibleAssetInterface;
use crate::Config;

impl<T: Config> FungibleAssetInterface<T> {
    /// Mints a `value` amount of tokens to the caller's account.
    pub(crate) fn issue(
        asset_id: AssetId,
        call: &IFungibleAsset::mintCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let amount = Common::<T>::to_balance(call.value)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::issue {
                asset_id,
                amount,
                asset_holder_kind: AssetHolderKind::Account,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Transfer(IFungibleAsset::Transfer {
                from: [0u8; 20].into(),
                to: caller.address.0.into(),
                value: call.value,
            }),
        )?;

        Ok(IFungibleAsset::mintCall::abi_encode_returns(&true))
    }

    /// Redeems a `value` amount of tokens from the caller's account.
    pub(crate) fn redeem(
        asset_id: AssetId,
        call: &IFungibleAsset::burnCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let value = Common::<T>::to_balance(call.value)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            caller.account_id.clone(),
            pallet_asset::Call::<T>::redeem {
                asset_id,
                value,
                asset_holder_kind: AssetHolderKind::Account,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            IFungibleAssetEvents::Transfer(IFungibleAsset::Transfer {
                from: caller.address.0.into(),
                to: [0u8; 20].into(),
                value: call.value,
            }),
        )?;

        Ok(IFungibleAsset::burnCall::abi_encode_returns(&true))
    }

    /// Returns `true` if the token is divisible.
    ///
    /// `decimals()` is 6 for every asset, so this is how a caller learns that an asset only
    /// moves in whole units before a transfer reverts with `InvalidGranularity`.
    pub(crate) fn is_divisible(
        asset_id: AssetId,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let divisible = pallet_asset::Pallet::<T>::is_divisible(&asset_id);

        Ok(IFungibleAsset::isDivisibleCall::abi_encode_returns(
            &divisible,
        ))
    }
}
