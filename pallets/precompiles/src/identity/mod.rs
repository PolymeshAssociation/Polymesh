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

//! Polymesh Identity Precompile
//!
//! Fixed-address precompile exposing Polymesh identity (DID) operations.

use alloc::vec::Vec;
use core::marker::PhantomData;
use core::num::NonZero;

use codec::{Decode, Encode};
use frame_support::dispatch::RawOrigin;
use frame_support::traits::Get;
use sp_runtime::traits::SaturatedConversion;

use pallet_revive::precompiles::alloy::primitives::{FixedBytes, IntoLogData};
use pallet_revive::precompiles::alloy::sol_types::{Revert, SolCall};
use pallet_revive::precompiles::{alloy, AddressMatcher, Ext, Precompile};
use pallet_revive::precompiles::{AddressMapper, Error, RuntimeCosts, H256};

use pallet_identity::{Pallet as IdentityPallet, WeightInfo};
use polymesh_primitives::asset::AssetId;
use polymesh_primitives::{Claim, CountryCode, CustomClaimTypeId, IdentityId, Scope};

use crate::revert_dispatch_error;

// Import the Solidity interface.
alloy::sol! {
    #[sol(all_derives)]
    "src/identity/IPolymeshIdentity.sol"
}

use IPolymeshIdentity::{IPolymeshIdentityCalls, IPolymeshIdentityEvents};

pub(crate) const ERR_INVALID_CALLER: &str = "Invalid caller";
pub(crate) const ERR_INVALID_CLAIM_TYPE: &str = "Invalid claim type";
pub(crate) const ERR_INVALID_CLAIM_DATA: &str = "Invalid claim data";

/// Polymesh identity precompile at the fixed address
/// `0x0000000000000000000000000000000000090000`.
pub struct PolymeshIdentity<T>(PhantomData<T>);

impl<T> Precompile for PolymeshIdentity<T>
where
    T: pallet_revive::Config + pallet_identity::Config,
{
    type T = T;
    type Interface = IPolymeshIdentityCalls;

    const MATCHER: AddressMatcher = AddressMatcher::Fixed(NonZero::new(9).unwrap());
    const HAS_CONTRACT_INFO: bool = false;

    fn call(
        _address: &[u8; 20],
        input: &Self::Interface,
        env: &mut impl Ext<T = Self::T>,
    ) -> Result<Vec<u8>, Error> {
        frame_support::ensure!(
            !env.is_delegate_call(),
            pallet_revive::Error::<Self::T>::PrecompileDelegateDenied,
        );

        match input {
            // State-changing calls - check read-only
            IPolymeshIdentityCalls::selfRegisterDid(_)
            | IPolymeshIdentityCalls::registerDid(_)
            | IPolymeshIdentityCalls::addClaim(_)
            | IPolymeshIdentityCalls::revokeClaim(_)
                if env.is_read_only() =>
            {
                Err(Error::Error(
                    pallet_revive::Error::<Self::T>::StateChangeDenied.into(),
                ))
            }

            // Views
            IPolymeshIdentityCalls::identity(call) => Self::identity(call, env),
            IPolymeshIdentityCalls::isVerified(call) => Self::is_verified(call, env),
            IPolymeshIdentityCalls::hasValidCdd(call) => Self::has_valid_cdd(call, env),

            // Writes
            IPolymeshIdentityCalls::selfRegisterDid(_) => Self::self_register_did(env),
            IPolymeshIdentityCalls::registerDid(call) => Self::register_did(call, env),
            IPolymeshIdentityCalls::addClaim(call) => Self::add_claim(call, env),
            IPolymeshIdentityCalls::revokeClaim(call) => Self::revoke_claim(call, env),
        }
    }
}

impl<T> PolymeshIdentity<T>
where
    T: pallet_revive::Config + pallet_identity::Config,
{
    fn identity(
        call: &IPolymeshIdentity::identityCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(1))?;

        let account = Self::account_id_from_h160(call.account.into_array().into());
        let did = IdentityPallet::<T>::get_identity(&account).unwrap_or_default();

        Ok(IPolymeshIdentity::identityCall::abi_encode_returns(
            &Self::did_to_bytes32(&did),
        ))
    }

    fn is_verified(
        call: &IPolymeshIdentity::isVerifiedCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(2))?;

        let account = Self::account_id_from_h160(call.account.into_array().into());
        let verified = Self::account_has_active_did(&account);

        Ok(IPolymeshIdentity::isVerifiedCall::abi_encode_returns(
            &verified,
        ))
    }

    fn has_valid_cdd(
        call: &IPolymeshIdentity::hasValidCddCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(T::DbWeight::get().reads(2))?;

        // CDD claims are no longer enforced on Polymesh; an active DID is
        // sufficient for onboarding.
        let account = Self::account_id_from_h160(call.account.into_array().into());
        let verified = Self::account_has_active_did(&account);

        Ok(IPolymeshIdentity::hasValidCddCall::abi_encode_returns(
            &verified,
        ))
    }

    fn self_register_did(env: &mut impl Ext<T = T>) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_identity::Config>::WeightInfo::self_register_did())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);

        IdentityPallet::<T>::self_register_did(RawOrigin::Signed(caller_account.clone()).into())
            .map_err(revert_dispatch_error)?;

        let did = IdentityPallet::<T>::get_identity(&caller_account).unwrap_or_default();
        Self::deposit_event(
            env,
            IPolymeshIdentityEvents::DidRegistered(IPolymeshIdentity::DidRegistered {
                account: caller.0.into(),
                did: Self::did_to_bytes32(&did),
            }),
        )?;

        Ok(IPolymeshIdentity::selfRegisterDidCall::abi_encode_returns(
            &true,
        ))
    }

    fn register_did(
        call: &IPolymeshIdentity::registerDidCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_identity::Config>::WeightInfo::register_did())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let target_account = Self::account_id_from_h160(call.target.into_array().into());

        IdentityPallet::<T>::register_did(
            RawOrigin::Signed(caller_account).into(),
            target_account.clone(),
        )
        .map_err(revert_dispatch_error)?;

        let did = IdentityPallet::<T>::get_identity(&target_account).unwrap_or_default();
        Self::deposit_event(
            env,
            IPolymeshIdentityEvents::DidRegistered(IPolymeshIdentity::DidRegistered {
                account: call.target,
                did: Self::did_to_bytes32(&did),
            }),
        )?;

        Ok(IPolymeshIdentity::registerDidCall::abi_encode_returns(
            &true,
        ))
    }

    fn add_claim(
        call: &IPolymeshIdentity::addClaimCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_identity::Config>::WeightInfo::add_claim())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let target = Self::did_from_bytes32(&call.target)?;
        let claim = Self::build_claim(call.claimType, &call.assetId, call.claimData)?;
        let expiry: Option<T::Moment> = match u64::try_from(call.expiry) {
            Ok(0) => None,
            Ok(ms) => Some(ms.saturated_into()),
            Err(_) => None,
        };

        IdentityPallet::<T>::add_claim(
            RawOrigin::Signed(caller_account).into(),
            target,
            claim,
            expiry,
        )
        .map_err(revert_dispatch_error)?;

        Self::deposit_event(
            env,
            IPolymeshIdentityEvents::ClaimAdded(IPolymeshIdentity::ClaimAdded {
                target: call.target,
                claimType: call.claimType,
                assetId: call.assetId,
            }),
        )?;

        Ok(IPolymeshIdentity::addClaimCall::abi_encode_returns(&true))
    }

    fn revoke_claim(
        call: &IPolymeshIdentity::revokeClaimCall,
        env: &mut impl Ext<T = T>,
    ) -> Result<Vec<u8>, Error> {
        env.charge(<T as pallet_identity::Config>::WeightInfo::revoke_claim())?;

        let caller = Self::caller(env)?;
        let caller_account = <T as pallet_revive::Config>::AddressMapper::to_account_id(&caller);
        let target = Self::did_from_bytes32(&call.target)?;
        let claim = Self::build_claim(call.claimType, &call.assetId, call.claimData)?;

        IdentityPallet::<T>::revoke_claim(
            RawOrigin::Signed(caller_account).into(),
            target,
            claim,
        )
        .map_err(revert_dispatch_error)?;

        Self::deposit_event(
            env,
            IPolymeshIdentityEvents::ClaimRevoked(IPolymeshIdentity::ClaimRevoked {
                target: call.target,
                claimType: call.claimType,
                assetId: call.assetId,
            }),
        )?;

        Ok(IPolymeshIdentity::revokeClaimCall::abi_encode_returns(
            &true,
        ))
    }

    // ==================== Helpers ====================

    /// Build a [`Claim`] from the ABI claim type, asset scope and claim data.
    fn build_claim(
        claim_type: u8,
        asset_id: &FixedBytes<16>,
        claim_data: alloy::primitives::U256,
    ) -> Result<Claim, Error> {
        let asset_bytes: [u8; 16] = asset_id.0;
        let scoped = |claim: fn(Scope) -> Claim| claim(Scope::Asset(AssetId::from(asset_bytes)));

        let claim = match claim_type {
            1 => scoped(Claim::Accredited),
            2 => scoped(Claim::Affiliate),
            3 => scoped(Claim::BuyLockup),
            4 => scoped(Claim::SellLockup),
            5 => scoped(Claim::KnowYourCustomer),
            6 => {
                let code = u8::try_from(claim_data).map_err(|_| {
                    Error::Revert(Revert {
                        reason: ERR_INVALID_CLAIM_DATA.into(),
                    })
                })?;
                let country = CountryCode::decode(&mut &[code][..]).map_err(|_| {
                    Error::Revert(Revert {
                        reason: ERR_INVALID_CLAIM_DATA.into(),
                    })
                })?;
                Claim::Jurisdiction(country, Scope::Asset(AssetId::from(asset_bytes)))
            }
            7 => scoped(Claim::Exempted),
            8 => scoped(Claim::Blocked),
            9 => {
                let id = u32::try_from(claim_data).map_err(|_| {
                    Error::Revert(Revert {
                        reason: ERR_INVALID_CLAIM_DATA.into(),
                    })
                })?;
                let scope = if asset_bytes == [0u8; 16] {
                    None
                } else {
                    Some(Scope::Asset(AssetId::from(asset_bytes)))
                };
                Claim::Custom(CustomClaimTypeId(id), scope)
            }
            _ => {
                return Err(Error::Revert(Revert {
                    reason: ERR_INVALID_CLAIM_TYPE.into(),
                }))
            }
        };
        Ok(claim)
    }

    fn account_has_active_did(account: &T::AccountId) -> bool {
        match IdentityPallet::<T>::get_identity(account) {
            Some(did) => IdentityPallet::<T>::is_did_active(did),
            None => false,
        }
    }

    fn did_to_bytes32(did: &IdentityId) -> FixedBytes<32> {
        let bytes: [u8; 32] = did.encode().try_into().unwrap_or_default();
        FixedBytes::<32>::from(bytes)
    }

    fn did_from_bytes32(bytes: &FixedBytes<32>) -> Result<IdentityId, Error> {
        IdentityId::decode(&mut &bytes.0[..]).map_err(|_| {
            Error::Revert(Revert {
                reason: ERR_INVALID_CLAIM_DATA.into(),
            })
        })
    }

    fn account_id_from_h160(account: pallet_revive::H160) -> T::AccountId {
        <T as pallet_revive::Config>::AddressMapper::to_account_id(&account)
    }

    /// Get the caller as an `H160` address.
    fn caller(env: &mut impl Ext<T = T>) -> Result<pallet_revive::H160, Error> {
        env.caller()
            .account_id()
            .map(<T as pallet_revive::Config>::AddressMapper::to_address)
            .map_err(|_| {
                Error::Revert(Revert {
                    reason: ERR_INVALID_CALLER.into(),
                })
            })
    }

    /// Deposit an event to the runtime.
    fn deposit_event(
        env: &mut impl Ext<T = T>,
        event: IPolymeshIdentityEvents,
    ) -> Result<(), Error> {
        let (topics, data) = event.into_log_data().split();
        let topics = topics.into_iter().map(|v| H256(v.0)).collect::<Vec<_>>();
        env.frame_meter_mut()
            .charge_weight_token(RuntimeCosts::DepositEvent {
                num_topic: topics.len() as u32,
                len: topics.len() as u32,
            })?;
        env.deposit_event(topics, data.to_vec());
        Ok(())
    }
}
