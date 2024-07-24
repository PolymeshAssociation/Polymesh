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

//! # External Agents Module
//!
//! The External Agents module provides extrinsics for managing the set of
//! agents for an asset, what groups those agents belong to,
//! and what permissions a group affords the agent.
//!
//! This is split into two categories, a) managing groups, b) managing agents.
//! In the former we find `create_group` and `set_group_permissions`
//! and in the latter we find `remove_agent`, `abdicate`, `change_group`,
//! as well as hooks for the Identity module, that,
//! via authorizations enable the addition of agents.
//!
//! Finally, the module provides functions for ensuring that an agent is permissioned.
//! These functions are then used by other pallets where relevant.
//!
//! ## Overview
//!
//! The External Agents module provides functions for:
//!
//! - Adding and altering custom agent groups.
//! - Managing the external agents of an asset.
//! - Ensuring that an agent has sufficient permissions for an extrinsic.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `create_group` creates a custom agent group (CAG) for a asset,
//!   resolving to a certain set of permissions.
//! - `set_group_permissions` changes the permissions a CAG resolves to for a asset.
//! - `remove_agent` removes an agent from an asset.
//! - `abdicate` removes the caller as an agent from an asset.
//! - `change_group` changes the agent group an asset belongs to.

#![cfg_attr(not(feature = "std"), no_std)]

//#[cfg(feature = "runtime-benchmarks")]
//pub mod benchmarking;

use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use pallet_base::{try_next_post, try_next_pre};
use pallet_identity::PermissionedCallOriginData;
pub use polymesh_common_utilities::traits::external_agents::{Config, Event, WeightInfo};
use polymesh_common_utilities::with_transaction;
use polymesh_primitives::agent::{AGId, AgentGroup};
use polymesh_primitives::asset::AssetID;
use polymesh_primitives::{
    extract_auth, AuthorizationData, EventDid, ExtrinsicPermissions, IdentityId, PalletPermissions,
    Signatory, SubsetRestriction,
};
use sp_std::prelude::*;

type Identity<T> = pallet_identity::Module<T>;
type Permissions<T> = pallet_permissions::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as ExternalAgents {
        /// The next per-asset AG ID in the sequence.
        ///
        /// The full ID is defined as a combination of `AssetID` and a number in this sequence,
        /// which starts from 1, rather than 0.
        pub AGIdSequence get(fn agent_group_id_sequence):
            map hasher(blake2_128_concat) AssetID
                => AGId;

        /// Maps an agent (`IdentityId`) to all assets they belong to, if any.
        pub AgentOf get(fn agent_of):
            double_map
                hasher(blake2_128_concat) IdentityId,
                hasher(blake2_128_concat) AssetID
                => ();

        /// Maps agents (`IdentityId`) for an `AssetID` to what AG they belong to, if any.
        pub GroupOfAgent get(fn agents):
            double_map
                hasher(blake2_128_concat) AssetID,
                hasher(twox_64_concat) IdentityId
                => Option<AgentGroup>;

        /// Maps an `AssetID` to the number of `Full` agents for it.
        pub NumFullAgents get(fn num_full_agents):
            map hasher(blake2_128_concat) AssetID
                => u32;

        /// For custom AGs of an `AssetID`, maps to what permissions an agent in that AG would have.
        pub GroupPermissions get(fn permissions):
            double_map
                hasher(blake2_128_concat) AssetID,
                hasher(twox_64_concat) AGId
                => Option<ExtrinsicPermissions>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::RuntimeOrigin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Creates a custom agent group (AG) for the given `asset_id`.
        ///
        /// The AG will have the permissions as given by `perms`.
        /// This new AG is then assigned `id = AGIdSequence::get() + 1` as its `AGId`,
        /// which you can use as `AgentGroup::Custom(id)` when adding agents for `asset_id`.
        ///
        /// # Arguments
        /// - `assetID` the [`AssetID] to add the custom group for.
        /// - `perms` that the new AG will have.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `TooLong` if `perms` had some string or list length that was too long.
        /// - `CounterOverflow` if `AGIdSequence::get() + 1` would exceed `u32::MAX`.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::create_group(perms.complexity() as u32)]
        pub fn create_group(origin, asset_id: AssetID, perms: ExtrinsicPermissions) -> DispatchResult {
            Self::base_create_group(origin, asset_id, perms).map(drop)
        }

        /// Updates the permissions of the custom AG identified by `id`, for the given `asset_id`.
        ///
        /// # Arguments
        /// - `assetID` the [`AssetID] the custom AG belongs to.
        /// - `id` for the custom AG within `asset_id`.
        /// - `perms` to update the custom AG to.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `TooLong` if `perms` had some string or list length that was too long.
        /// - `NoSuchAG` if `id` does not identify a custom AG.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::set_group_permissions(perms.complexity() as u32)]
        pub fn set_group_permissions(origin, asset_id: AssetID, id: AGId, perms: ExtrinsicPermissions) -> DispatchResult {
            Self::base_set_group_permissions(origin, asset_id, id, perms)
        }

        /// Remove the given `agent` from `asset_id`.
        ///
        /// # Arguments
        /// - `assetID` the [`AssetID] that has the `agent` to remove.
        /// - `agent` of `asset_id` to remove.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `NotAnAgent` if `agent` is not an agent of `asset_id`.
        /// - `RemovingLastFullAgent` if `agent` is the last full one.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::remove_agent()]
        pub fn remove_agent(origin, asset_id: AssetID, agent: IdentityId) -> DispatchResult {
            Self::base_remove_agent(origin, asset_id, agent)
        }

        /// Abdicate agentship for `asset_id`.
        ///
        /// # Arguments
        /// - `assetID` the [`AssetID] of which the caller is an agent.
        ///
        /// # Errors
        /// - `NotAnAgent` if the caller is not an agent of `asset_id`.
        /// - `RemovingLastFullAgent` if the caller is the last full agent.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::abdicate()]
        pub fn abdicate(origin, asset_id: AssetID) -> DispatchResult {
            Self::base_abdicate(origin, asset_id)
        }

        /// Change the agent group that `agent` belongs to in `asset_id`.
        ///
        /// # Arguments
        /// - `assetID` the [`AssetID] that has the `agent`.
        /// - `agent` of `asset_id` to change the group for.
        /// - `group` that `agent` will belong to in `asset_id`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `NoSuchAG` if `id` does not identify a custom AG.
        /// - `NotAnAgent` if `agent` is not an agent of `asset_id`.
        /// - `RemovingLastFullAgent` if `agent` was a `Full` one and is being demoted.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = match group {
            AgentGroup::Custom(_) => <T as Config>::WeightInfo::change_group_custom(),
            _ => <T as Config>::WeightInfo::change_group_builtin(),
        }]
        pub fn change_group(origin, asset_id: AssetID, agent: IdentityId, group: AgentGroup) -> DispatchResult {
            Self::base_change_group(origin, asset_id, agent, group)
        }

        /// Accept an authorization by an agent "Alice" who issued `auth_id`
        /// to also become an agent of the asset Alice specified.
        ///
        /// # Arguments
        /// - `auth_id` identifying the authorization to accept.
        ///
        /// # Errors
        /// - `AuthorizationError::Invalid` if `auth_id` does not exist for the given caller.
        /// - `AuthorizationError::Expired` if `auth_id` is for an auth that has expired.
        /// - `AuthorizationError::BadType` if `auth_id` was not for a `BecomeAgent` auth type.
        /// - `UnauthorizedAgent` if "Alice" is not permissioned to provide the auth.
        /// - `NoSuchAG` if the group referred to a custom that does not exist.
        /// - `AlreadyAnAgent` if the caller is already an agent of the asset.
        ///
        /// # Permissions
        /// * Agent
        #[weight = <T as Config>::WeightInfo::accept_become_agent()]
        pub fn accept_become_agent(origin, auth_id: u64) -> DispatchResult {
            Self::base_accept_become_agent(origin, auth_id)
        }

        /// Utility extrinsic to batch `create_group` and  `add_auth`.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::create_group_and_add_auth(perms.complexity() as u32)]
        pub fn create_group_and_add_auth(
            origin,
            asset_id: AssetID,
            perms: ExtrinsicPermissions,
            target: IdentityId,
            expiry: Option<T::Moment>
        ) -> DispatchResult {
            Self::base_create_group_and_add_auth(origin, asset_id, perms, target, expiry)
        }

        /// Utility extrinsic to batch `create_group` and  `change_group` for custom groups only.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::create_and_change_custom_group(perms.complexity() as u32)]
        pub fn create_and_change_custom_group(
            origin,
            asset_id: AssetID,
            perms: ExtrinsicPermissions,
            agent: IdentityId
        ) -> DispatchResult {
            with_transaction(|| Self::base_create_and_change_custom_group(origin, asset_id, perms, agent))
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
        /// An AG with the given `AGId` did not exist for the `AssetID`.
        NoSuchAG,
        /// The agent is not authorized to call the current extrinsic.
        UnauthorizedAgent,
        /// The provided `agent` is already an agent for the `AssetID`.
        AlreadyAnAgent,
        /// The provided `agent` is not an agent for the `AssetID`.
        NotAnAgent,
        /// This agent is the last full one, and it's being removed,
        /// making the asset orphaned.
        RemovingLastFullAgent,
        /// The caller's secondary key does not have the required asset permission.
        SecondaryKeyNotAuthorizedForAsset,
    }
}

impl<T: Config> Module<T> {
    fn base_accept_become_agent(origin: T::RuntimeOrigin, auth_id: u64) -> DispatchResult {
        let to = Identity::<T>::ensure_perms(origin)?;
        Identity::<T>::accept_auth_with(&to.into(), auth_id, |data, from| {
            let (asset_id, group) = extract_auth!(data, BecomeAgent(t, ag));

            Self::ensure_agent_permissioned(&asset_id, from)?;
            Self::ensure_agent_group_valid(&asset_id, group)?;
            ensure!(
                GroupOfAgent::get(&asset_id, to).is_none(),
                Error::<T>::AlreadyAnAgent
            );

            Self::unchecked_add_agent(asset_id, to, group)?;
            Ok(())
        })
    }

    fn base_create_group(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        perms: ExtrinsicPermissions,
    ) -> Result<(IdentityId, AGId), DispatchError> {
        let did = Self::ensure_perms(origin, asset_id)?;
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;
        // Fetch the AG id & advance the sequence.
        let id = AGIdSequence::try_mutate(asset_id, try_next_pre::<T, _>)?;
        // Commit & emit.
        GroupPermissions::insert(asset_id, id, perms.clone());
        Self::deposit_event(Event::GroupCreated(did.for_event(), asset_id, id, perms));
        Ok((did, id))
    }

    fn base_create_group_and_add_auth(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        perms: ExtrinsicPermissions,
        target: IdentityId,
        expiry: Option<T::Moment>,
    ) -> DispatchResult {
        let (did, ag_id) = Self::base_create_group(origin, asset_id, perms)?;
        <Identity<T>>::add_auth(
            did,
            Signatory::Identity(target),
            AuthorizationData::BecomeAgent(asset_id, AgentGroup::Custom(ag_id)),
            expiry,
        )?;
        Ok(())
    }

    fn base_set_group_permissions(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        id: AGId,
        perms: ExtrinsicPermissions,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin, asset_id)?.for_event();
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;
        Self::ensure_custom_agent_group_exists(&asset_id, &id)?;

        // Commit & emit.
        GroupPermissions::insert(asset_id, id, perms.clone());
        Self::deposit_event(Event::GroupPermissionsUpdated(did, asset_id, id, perms));
        Ok(())
    }

    fn base_remove_agent(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        agent: IdentityId,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin, asset_id)?.for_event();
        Self::try_mutate_agents_group(asset_id, agent, None)?;
        Self::deposit_event(Event::AgentRemoved(did, asset_id, agent));
        Ok(())
    }

    fn base_abdicate(origin: T::RuntimeOrigin, asset_id: AssetID) -> DispatchResult {
        let did = Self::ensure_asset_perms(origin, asset_id)?.primary_did;
        Self::try_mutate_agents_group(asset_id, did, None)?;
        Self::deposit_event(Event::AgentRemoved(did.for_event(), asset_id, did));
        Ok(())
    }

    fn base_create_and_change_custom_group(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        perms: ExtrinsicPermissions,
        agent: IdentityId,
    ) -> DispatchResult {
        let (did, ag_id) = Self::base_create_group(origin, asset_id, perms)?;
        Self::unsafe_change_group(did.for_event(), asset_id, agent, AgentGroup::Custom(ag_id))
    }

    fn base_change_group(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
        agent: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin, asset_id)?.for_event();
        Self::unsafe_change_group(did, asset_id, agent, group)
    }

    fn unsafe_change_group(
        did: EventDid,
        asset_id: AssetID,
        agent: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        Self::ensure_agent_group_valid(&asset_id, group)?;
        Self::try_mutate_agents_group(asset_id, agent, Some(group))?;
        Self::deposit_event(Event::GroupChanged(did, asset_id, agent, group));
        Ok(())
    }

    /// Ensure that `group` is a valid agent group for `asset_id`.
    fn ensure_agent_group_valid(asset_id: &AssetID, group: AgentGroup) -> DispatchResult {
        if let AgentGroup::Custom(id) = group {
            Self::ensure_custom_agent_group_exists(asset_id, &id)?;
        }
        Ok(())
    }

    /// Ensure that `id` identifies a custom AG of `asset_id`.
    fn ensure_custom_agent_group_exists(asset_id: &AssetID, id: &AGId) -> DispatchResult {
        ensure!(
            (AGId(1)..=AGIdSequence::get(&asset_id)).contains(id),
            Error::<T>::NoSuchAG
        );
        Ok(())
    }

    /// Ensure that `agent` is an agent of `asset_id` and set the group to `group`.
    fn try_mutate_agents_group(
        asset_id: AssetID,
        agent: IdentityId,
        group: Option<AgentGroup>,
    ) -> DispatchResult {
        GroupOfAgent::try_mutate(asset_id, agent, |slot| {
            ensure!(slot.is_some(), Error::<T>::NotAnAgent);

            match (*slot, group) {
                // Identity transition. No change in count.
                (Some(AgentGroup::Full), Some(AgentGroup::Full)) => {}
                // Demotion/Removal. Count is decrementing.
                (Some(AgentGroup::Full), _) => Self::dec_full_count(asset_id)?,
                // Promotion. Count is incrementing.
                (_, Some(AgentGroup::Full)) => Self::inc_full_count(asset_id)?,
                // Just a change in groups.
                _ => {}
            }

            // Removal
            if group.is_none() {
                AgentOf::remove(agent, asset_id);
            }

            *slot = group;
            Ok(())
        })
    }

    pub fn unchecked_add_agent(
        asset_id: AssetID,
        did: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        if let AgentGroup::Full = group {
            Self::inc_full_count(asset_id)?;
        }
        GroupOfAgent::insert(asset_id, did, group);
        AgentOf::insert(did, asset_id, ());
        Self::deposit_event(Event::AgentAdded(did.for_event(), asset_id, group));
        Ok(())
    }

    /// Decrement the full agent count, or error on < 1.
    fn dec_full_count(asset_id: AssetID) -> DispatchResult {
        NumFullAgents::try_mutate(asset_id, |n| {
            *n = n
                .checked_sub(1)
                .filter(|&x| x > 0)
                .ok_or(Error::<T>::RemovingLastFullAgent)?;
            Ok(())
        })
    }

    /// Increment the full agent count, or error on overflow.
    fn inc_full_count(asset_id: AssetID) -> DispatchResult {
        NumFullAgents::try_mutate(asset_id, try_next_post::<T, _>).map(drop)
    }

    /// Ensures that `origin` is a permissioned agent for `asset_id`.
    pub fn ensure_perms(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
    ) -> Result<IdentityId, DispatchError> {
        Self::ensure_agent_asset_perms(origin, asset_id).map(|d| d.primary_did)
    }

    /// Ensures that `origin` is a permissioned agent for `asset_id`.
    pub fn ensure_agent_asset_perms(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let data = Self::ensure_asset_perms(origin, asset_id)?;
        Self::ensure_agent_permissioned(&asset_id, data.primary_did)?;
        Ok(data)
    }

    /// Ensure that `origin` is permissioned for this call
    /// and the secondary key has relevant asset permissions.
    pub fn ensure_asset_perms(
        origin: T::RuntimeOrigin,
        asset_id: AssetID,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let data = <Identity<T>>::ensure_origin_call_permissions(origin)?;
        let skey = data.secondary_key.as_ref();

        // If `secondary_key` is None, the caller is the primary key and has all permissions.
        if let Some(sk) = skey {
            ensure!(
                sk.has_asset_permission(asset_id),
                Error::<T>::SecondaryKeyNotAuthorizedForAsset
            );
        }

        Ok(data)
    }

    /// Ensures that `agent` is permissioned for `asset_id`.
    pub fn ensure_agent_permissioned(asset_id: &AssetID, agent: IdentityId) -> DispatchResult {
        ensure!(
            Self::agent_permissions(asset_id, agent).sufficient_for(
                &<Permissions<T>>::current_pallet_name(),
                &<Permissions<T>>::current_dispatchable_name()
            ),
            Error::<T>::UnauthorizedAgent
        );
        Ok(())
    }

    /// Returns `agent`'s permission set in `asset_id`.
    fn agent_permissions(asset_id: &AssetID, agent: IdentityId) -> ExtrinsicPermissions {
        let pallet = |p: &str| PalletPermissions::entire_pallet(p.into());
        let in_pallet = |p: &str, dns| PalletPermissions::new(p.into(), dns);
        fn elems<T: Ord, const N: usize>(elems: [T; N]) -> SubsetRestriction<T> {
            SubsetRestriction::elems(elems)
        }
        match GroupOfAgent::get(asset_id, agent) {
            None => ExtrinsicPermissions::empty(),
            Some(AgentGroup::Full) => ExtrinsicPermissions::default(),
            Some(AgentGroup::Custom(ag_id)) => {
                GroupPermissions::get(asset_id, ag_id).unwrap_or_else(ExtrinsicPermissions::empty)
            }
            // Anything but extrinsics in this pallet.
            Some(AgentGroup::ExceptMeta) => SubsetRestriction::except(pallet("ExternalAgents")),
            // Pallets `CorporateAction`, `CorporateBallot`, and `CapitalDistribution`.
            Some(AgentGroup::PolymeshV1CAA) => elems([
                pallet("CorporateAction"),
                pallet("CorporateBallot"),
                pallet("CapitalDistribution"),
            ]),
            Some(AgentGroup::PolymeshV1PIA) => elems([
                // All in `Sto` except `Sto::invest`.
                in_pallet("Sto", SubsetRestriction::except("invest".into())),
                // Asset::{issue, redeem, controller_transfer}.
                in_pallet(
                    "Asset",
                    elems([
                        "issue".into(),
                        "redeem".into(),
                        "controller_transfer".into(),
                    ]),
                ),
            ]),
        }
    }
}
