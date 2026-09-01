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

//! ERC-721 core: ownership, transfers and approvals.

use alloc::vec;
use alloc::vec::Vec;

use frame_support::traits::Get;
use frame_support::weights::Weight;
use pallet_revive::precompiles::alloy::primitives::{Address, U256};
use pallet_revive::precompiles::alloy::sol_types::SolCall;
use pallet_revive::precompiles::{Error, Ext, RuntimeCosts};
use pallet_revive::H160;

use pallet_nft::{NFTAccountCount, OperatorApproval, Owner, TokenApproval};
use polymesh_precompiles::{INonFungibleAsset, INonFungibleAssetEvents};
use polymesh_primitives::asset::{AssetHolder, AssetId};
use polymesh_primitives::nft::{NFTId, NFTs};
use polymesh_primitives::portfolio::{Fund, FundDescription};
use polymesh_primitives::traits::SettlementFnTrait;
use polymesh_primitives::WeightMeter;

use crate::common::{revert, Common};
use crate::interface::nft::{
    NonFungibleAssetInterface, ERR_NFT_INST_NOT_EXECUTED, ERR_NFT_NOT_FOUND,
    ERR_OWNER_NOT_AN_ACCOUNT, ERR_UNSAFE_CONTRACT_RECEIVER,
};
use crate::Config;

impl<T: Config> NonFungibleAssetInterface<T> {
    /// Returns the number of NFTs of this collection held by `owner`'s account key.
    pub(crate) fn balance_of(
        asset_id: AssetId,
        call: &INonFungibleAsset::balanceOfCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let owner = Common::<T>::account_id32(env, call.owner)?;
        let count = NFTAccountCount::<T>::get(&owner, &asset_id);

        Ok(INonFungibleAsset::balanceOfCall::abi_encode_returns(
            &U256::from(count),
        ))
    }

    /// Returns the address holding `tokenId`.
    pub(crate) fn owner_of(
        asset_id: AssetId,
        call: &INonFungibleAsset::ownerOfCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let owner = Self::account_owner_of(asset_id, &nft_id)?;

        Ok(INonFungibleAsset::ownerOfCall::abi_encode_returns(&owner))
    }

    /// Transfers `tokenId` from `from` to `to`.
    pub(crate) fn transfer_from(
        asset_id: AssetId,
        call: &INonFungibleAsset::transferFromCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Self::base_transfer(asset_id, call.from, call.to, call.tokenId, env)?;
        Ok(Vec::new())
    }

    /// Transfers `tokenId` from `from` to `to`, refusing contract receivers.
    ///
    /// See [`Self::ensure_eoa_receiver`] for why.
    pub(crate) fn safe_transfer_from(
        asset_id: AssetId,
        call: &INonFungibleAsset::safeTransferFrom_0Call,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Self::ensure_eoa_receiver(call.to, env)?;
        Self::base_transfer(asset_id, call.from, call.to, call.tokenId, env)?;
        Ok(Vec::new())
    }

    /// Transfers `tokenId` from `from` to `to`, refusing contract receivers.
    ///
    /// `data` is ignored: it exists only to be forwarded to `onERC721Received`, which is never
    /// invoked because the receiver is always an externally-owned account.
    pub(crate) fn safe_transfer_from_with_data(
        asset_id: AssetId,
        call: &INonFungibleAsset::safeTransferFrom_1Call,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        Self::ensure_eoa_receiver(call.to, env)?;
        Self::base_transfer(asset_id, call.from, call.to, call.tokenId, env)?;
        Ok(Vec::new())
    }

    /// Rejects `to` if it has code.
    ///
    /// ERC-721's `safeTransferFrom` exists to stop NFTs being locked in contracts that cannot
    /// handle them, which it does by requiring an `onERC721Received` acknowledgement. A
    /// precompile cannot re-enter the EVM to make that call, so instead of skipping the check —
    /// which would silently drop the guarantee — we refuse every receiver with code.
    ///
    /// This is stricter than the standard: a compliant receiver is rejected too. It is never
    /// weaker, so an NFT can not become stranded. Contracts that knowingly handle NFTs can still
    /// receive them via `transferFrom`.
    ///
    /// Note precompile addresses also carry code, so they are refused as well.
    fn ensure_eoa_receiver(to: Address, env: &mut impl Ext<T = T>) -> Result<(), Error> {
        env.frame_meter_mut()
            .charge_weight_token(RuntimeCosts::CodeSize)?;

        if env.code_size(&H160::from(to.into_array())) > 0 {
            return Err(revert(ERR_UNSAFE_CONTRACT_RECEIVER));
        }
        Ok(())
    }

    /// Approves `to` to transfer `tokenId`, or clears the approval when `to` is the zero address.
    pub(crate) fn approve(
        asset_id: AssetId,
        call: &INonFungibleAsset::approveCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let nft_id = Self::nft_id(call.tokenId)?;

        // ERC-721 uses the zero address to mean "no approval".
        let spender = if call.to == Address::ZERO {
            None
        } else {
            Some(Common::<T>::account_id(env, call.to)?)
        };

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_nft::Call::<T>::approve {
                asset_id,
                nft_id,
                spender,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::Approval(INonFungibleAsset::Approval {
                owner: caller.address.0.into(),
                approved: call.to,
                tokenId: call.tokenId,
            }),
        )?;

        Ok(Vec::new())
    }

    /// Grants or revokes `operator` for every NFT of this collection held by the caller.
    pub(crate) fn set_approval_for_all(
        asset_id: AssetId,
        call: &INonFungibleAsset::setApprovalForAllCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        let caller = Common::<T>::caller(env)?;
        let operator = Common::<T>::account_id(env, call.operator)?;

        Common::<T>::call_runtime(
            env,
            caller.runtime_origin(),
            pallet_nft::Call::<T>::set_approval_for_all {
                asset_id,
                operator,
                approved: call.approved,
            },
        )?;

        Common::<T>::deposit_event(
            env,
            INonFungibleAssetEvents::ApprovalForAll(INonFungibleAsset::ApprovalForAll {
                owner: caller.address.0.into(),
                operator: call.operator,
                approved: call.approved,
            }),
        )?;

        Ok(Vec::new())
    }

    /// Returns the account approved for `tokenId`, or the zero address if there is none.
    pub(crate) fn get_approved(
        asset_id: AssetId,
        call: &INonFungibleAsset::getApprovedCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let nft_id = Self::nft_id(call.tokenId)?;
        let approved = match TokenApproval::<T>::get(&asset_id, &nft_id) {
            Some(account) => Common::<T>::address_of(&account)?,
            None => Address::ZERO,
        };

        Ok(INonFungibleAsset::getApprovedCall::abi_encode_returns(
            &approved,
        ))
    }

    /// Returns whether `operator` may transfer any NFT of this collection held by `owner`.
    pub(crate) fn is_approved_for_all(
        asset_id: AssetId,
        call: &INonFungibleAsset::isApprovedForAllCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as frame_system::Config>::DbWeight::get().reads(1))?;

        let owner = Common::<T>::account_id32(env, call.owner)?;
        let operator = Common::<T>::account_id32(env, call.operator)?;
        let approved = OperatorApproval::<T>::get((&owner, &operator, &asset_id));

        Ok(INonFungibleAsset::isApprovedForAllCall::abi_encode_returns(
            &approved,
        ))
    }

    /// Returns the address of the account key holding `nft_id`.
    ///
    /// Reverts if the NFT does not exist or is held in a portfolio, which has no EVM address.
    pub(crate) fn account_owner_of(asset_id: AssetId, nft_id: &NFTId) -> Result<Address, Error> {
        match Owner::<T>::get(asset_id, nft_id) {
            Some(AssetHolder::Account(account)) => Common::<T>::address_of(&account),
            Some(AssetHolder::Portfolio(_)) => Err(revert(ERR_OWNER_NOT_AN_ACCOUNT)),
            None => Err(revert(ERR_NFT_NOT_FOUND)),
        }
    }

    /// Moves `token_id` from `from` to `to` through settlement, emitting a `Transfer` event.
    ///
    /// When the caller is not `from` this consumes the caller's NFT approval, exactly as the
    /// fungible precompile's `transferFrom` consumes an allowance.
    pub(crate) fn base_transfer(
        asset_id: AssetId,
        from: Address,
        to: Address,
        token_id: U256,
        env: &mut impl Ext<T = T>,
    ) -> Result<(), Error> {
        let nft_id = Self::nft_id(token_id)?;
        let from_holder = Common::<T>::asset_holder(env, from)?;

        let nfts = NFTs::new_unverified(asset_id, vec![nft_id]);
        let fund = Fund::new(FundDescription::NonFungible(nfts), None);

        let worst_case_weight =
            <T as pallet_asset::Config>::SettlementFn::transfer_funds_weight_limit(
                Some(&from_holder),
                &fund,
            );
        let charged_amount = env.charge(worst_case_weight)?;

        let caller = Common::<T>::caller(env)?;
        let to_holder = Common::<T>::asset_holder(env, to)?;

        let mut weight_meter = WeightMeter::from_limit_unchecked(Weight::zero(), worst_case_weight);

        let result = Common::<T>::with_runtime_call(
            env,
            pallet_settlement::Call::<T>::transfer_funds {
                from: Some(from_holder.clone()),
                to: to_holder.clone(),
                fund: fund.clone(),
            },
            || {
                <T as pallet_asset::Config>::SettlementFn::transfer_funds(
                    caller.runtime_origin(),
                    Some(from_holder),
                    to_holder,
                    fund,
                    &mut weight_meter,
                    #[cfg(feature = "runtime-benchmarks")]
                    false,
                )
            },
        )?;

        match result {
            Err(e) => Err(crate::common::extrinsic_error(e)),
            Ok(inst_id) => {
                env.adjust_gas(charged_amount, weight_meter.consumed());

                // Instruction was created but not executed
                if inst_id.is_some() {
                    return Err(revert(ERR_NFT_INST_NOT_EXECUTED));
                }

                Common::<T>::deposit_event(
                    env,
                    INonFungibleAssetEvents::Transfer(INonFungibleAsset::Transfer {
                        from,
                        to,
                        tokenId: token_id,
                    }),
                )?;

                Ok(())
            }
        }
    }
}
