// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath

// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.

// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

//! # Testnet module
//!
//! This module contains helpers or mocked functionality.
//!
//! ## Overview
//!
//! Testnet pallet is used to cover functionality that is not yet implemented or required external
//! entities to carry on some processes.
//! The main idea is that it mocks that functionality and allow generating development or testnet
//! networks.
//!
//! ### DID generation and CDD claims.
//!
//! DID generation requires a CDD service provider involved in the process. In this module, you can
//! find some extrinsics allow to generate those DID by yourself.
//!
//! ### Getting information using events.
//!
//! It also contains some extrinsics to generate events containing specific information. Those
//! events are used in web application to extract easily information from the chain.
//!
//! ## Dispatchable Functions.
//!
//! - `register_did` - Register a new did with a CDD claim for the caller.
//! - `cdd_register_did` - Registers a new did for the target and attaches a CDD claim to it.
//! - `get_my_did` - Generates an event containing the DID of the caller.
//! - `get_cdd_of` -> Generates an event containing the CDD claim of the target account.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, weights::Weight,
};
use frame_system::{ensure_signed, RawOrigin};
use pallet_identity::PermissionedCallOriginData;
use polymesh_common_utilities::{
    protocol_fee::ProtocolOp, traits::identity::Config as IdentityConfig, TestUtilsFn,
};
use polymesh_primitives::{secondary_key, CddId, Claim, IdentityId, InvestorUid, SecondaryKey};
use sp_std::{prelude::*, vec};

type Identity<T> = pallet_identity::Module<T>;
type CallPermissions<T> = pallet_permissions::Module<T>;

pub trait WeightInfo {
    fn register_did(i: u32) -> Weight;
    fn mock_cdd_register_did() -> Weight;
    fn get_my_did() -> Weight;
    fn get_cdd_of() -> Weight;
}

pub trait Config: IdentityConfig {
    type Event: From<Event<Self>> + Into<<Self as frame_system::Config>::Event>;
    /// Weight information for extrinsics in the identity pallet.
    type WeightInfo: WeightInfo;
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as frame_system::Config>::AccountId,
    {
        /// A new mocked `InvestorUid` has been created for the given Identity.
        /// (Target DID, New InvestorUid)
        MockInvestorUIDCreated(IdentityId, InvestorUid),
        /// Emits the `IdentityId` and the `AccountId` of the caller.
        /// (Caller DID, Caller account)
        DidStatus(IdentityId, AccountId),
        /// Shows the `DID` associated to the `AccountId`, and a flag indicates if that DID has a
        /// valid CDD claim.
        /// (Target DID, Target Account, a valid CDD claim exists)
        CddStatus(Option<IdentityId>, AccountId, bool),
    }
);

decl_storage! {
    trait Store for Module<T: Config> as testnet {
    }
}
decl_error! {
    pub enum Error for Module<T: Config> {
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {

        type Error = Error<T>;

        fn deposit_event() = default;

        /// Generates a new `IdentityID` for the caller, and issues a self-generated CDD claim.
        ///
        /// The caller account will be the primary key of that identity.
        /// For each account of `secondary_keys`, a new `JoinIdentity` authorization is created, so
        /// each of them will need to accept it before become part of this new `IdentityID`.
        ///
        /// # Errors
        /// - `AlreadyLinked` if the caller account or if any of the given `secondary_keys` has already linked to an `IdentityID`
        /// - `SecondaryKeysContainPrimaryKey` if `secondary_keys` contains the caller account.
        /// - `DidAlreadyExists` if auto-generated DID already exists.
        #[weight = <T as Config>::WeightInfo::register_did(secondary_keys.len() as u32)]
        pub fn register_did(
            origin,
            uid: InvestorUid,
            secondary_keys: Vec<SecondaryKey<T::AccountId>>,
        ) {
            let sender = ensure_signed(origin)?;
            Identity::<T>::_register_did(sender.clone(), secondary_keys, Some(ProtocolOp::IdentityRegisterDid))?;

            // Add CDD claim
            let did = Identity::<T>::get_identity(&sender).ok_or("DID Self-register failed")?;
            let cdd_claim = Claim::CustomerDueDiligence(CddId::new_v1(did, uid));
            Identity::<T>::base_add_claim(did, cdd_claim, did, None);
        }

        /// Registers a new Identity for the `target_account` and issues a CDD claim to it.
        /// The Investor UID is generated deterministically by the hash of the generated DID and
        /// then we fix it to be compliant with UUID v4.
        ///
        /// # See
        /// - [RFC 4122: UUID](https://tools.ietf.org/html/rfc4122)
        ///
        /// # Failure
        /// - `origin` has to be an active CDD provider. Inactive CDD providers cannot add new
        /// claims.
        /// - `target_account` (primary key of the new Identity) can be linked to just one and only
        /// one identity.
        #[weight = <T as Config>::WeightInfo::mock_cdd_register_did()]
        pub fn mock_cdd_register_did(origin, target_account: T::AccountId) {
            let cdd_id = Identity::<T>::ensure_perms(origin)?;
            let target_did = Identity::<T>::base_cdd_register_did(cdd_id, target_account, vec![])?;
            let target_uid = confidential_identity::mocked::make_investor_uid(target_did.as_bytes());

            // Add CDD claim for the target
            let cdd_claim = Claim::CustomerDueDiligence(CddId::new_v1(target_did, target_uid.clone().into()));
            Identity::<T>::base_add_claim(target_did, cdd_claim, cdd_id, None);

            Self::deposit_event(RawEvent::MockInvestorUIDCreated(target_did, target_uid.into()));
        }

        /// Emits an event with caller's identity.
        #[weight = <T as Config>::WeightInfo::get_my_did()]
        pub fn get_my_did(origin) {
            let PermissionedCallOriginData {
                sender,
                primary_did: did,
                ..
            } = Identity::<T>::ensure_origin_call_permissions(origin)?;
            Self::deposit_event(RawEvent::DidStatus(did, sender));
        }

        /// Emits an event with caller's identity and CDD status.
        #[weight = <T as Config>::WeightInfo::get_cdd_of()]
        pub fn get_cdd_of(origin, of: T::AccountId) {
            let sender = ensure_signed(origin)?;
            CallPermissions::<T>::ensure_call_permissions(&sender)?;
            let did_opt = Identity::<T>::get_identity(&of);
            let has_cdd = did_opt.map(Identity::<T>::has_valid_cdd).unwrap_or_default();

            Self::deposit_event(RawEvent::CddStatus(did_opt, of, has_cdd));
        }
    }
}

impl<T: Config> TestUtilsFn<T::AccountId> for Module<T> {
    fn register_did(
        target: T::AccountId,
        investor: InvestorUid,
        secondary_keys: Vec<secondary_key::api::SecondaryKey<T::AccountId>>,
    ) -> DispatchResult {
        let keys = secondary_keys.into_iter().map(SecondaryKey::from).collect();
        Self::register_did(RawOrigin::Signed(target).into(), investor, keys)
    }
}
