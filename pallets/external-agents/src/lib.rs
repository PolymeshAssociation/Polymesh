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

//! # External Agents Module
//!
//! The External Agents module is TODO
//!
//! ## Overview
//!
//! The External Agents module provides functions for:
//!
//! - TODO
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - TODO
//!
//! ### Public Functions
//!
//! - TODO

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use frame_support::{
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
    weights::Weight,
};
use pallet_identity::PermissionedCallOriginData;
use polymesh_primitives::agent::{AGId, AgentGroup};
use polymesh_primitives::{EventDid, ExtrinsicPermissions, IdentityId, Ticker};

type Identity<T> = pallet_identity::Module<T>;
type Permissions<T> = pallet_permissions::Module<T>;

pub trait WeightInfo {
    fn create_group() -> Weight;
    fn set_group_permissions() -> Weight;
    fn remove_agent() -> Weight;
    fn change_group() -> Weight;
}

pub trait Trait: frame_system::Trait + polymesh_common_utilities::balances::Trait {
    /// The overarching event type.
    type Event: From<Event> + Into<<Self as frame_system::Trait>::Event>;

    type WeightInfo: WeightInfo;
}

decl_storage! {
    trait Store for Module<T: Trait> as ExternalAgent {
        /// The next per-`Ticker` AG ID in the sequence.
        ///
        /// The full ID is defined as a combination of `Ticker` and a number in this sequence,
        /// which starts from 1, rather than 0.
        pub AGIdSequence get(fn agent_group_id_sequence):
            map hasher(blake2_128_concat) Ticker
                => AGId;

        /// Maps agents (`IdentityId`) for a `Ticker` to what AG they belong to, if any.
        pub GroupOfAgent get(fn agents):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) IdentityId
                => Option<AgentGroup>;

        /// For custom AGs of a `Ticker`, maps to what permissions an agent in that AG would have.
        pub GroupPermissions get(fn permissions):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) AGId
                => ExtrinsicPermissions;
    }
}

decl_module! {
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Creates a custom agent group (AG) for the given `ticker`.
        ///
        /// The AG will have the permissions as given by `perms`.
        /// This new AG is then assigned `id = AGIdSequence::get() + 1` as its `AGId`,
        /// which you can use as `AgentGroup::Custom(id)` when adding agents for `ticker`.
        ///
        /// # Arguments
        /// - `ticker` to add the custom group for.
        /// - `perms` that the new AG will have.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `TooLong` if `perms` had some string or list length that was too long.
        /// - `LocalAGIdOverflow` if `AGIdSequence::get() + 1` would exceed `u32::MAX`.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Trait>::WeightInfo::create_group()]
        pub fn create_group(origin, ticker: Ticker, perms: ExtrinsicPermissions) -> DispatchResult {
            Self::base_create_group(origin, ticker, perms)
        }

        /// Updates the permissions of the custom AG identified by `id`, for the given `ticker`.
        ///
        /// # Arguments
        /// - `ticker` the custom AG belongs to.
        /// - `id` for the custom AG within `ticker`.
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
        #[weight = <T as Trait>::WeightInfo::set_group_permissions()]
        pub fn set_group_permissions(origin, ticker: Ticker, id: AGId, perms: ExtrinsicPermissions) -> DispatchResult {
            Self::base_set_group_permissions(origin, ticker, id, perms)
        }

        /// Remove the given `agent` from `ticker`.
        ///
        /// # Arguments
        /// - `ticker` that has the `agent` to remove.
        /// - `agent` of `ticker` to remove.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `NotAnAgent` if `agent` is not an agent of `ticker`.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Trait>::WeightInfo::remove_agent()]
        pub fn remove_agent(origin, ticker: Ticker, agent: IdentityId) -> DispatchResult {
            Self::base_remove_agent(origin, ticker, agent)
        }

        /// Change the agent group that `agent` belongs to in `ticker`.
        ///
        /// # Arguments
        /// - `ticker` that has the `agent`.
        /// - `agent` of `ticker` to change the group for.
        /// - `group` that `agent` will belong to in `ticker`.
        ///
        /// # Errors
        /// - `UnauthorizedAgent` if `origin` was not authorized as an agent to call this.
        /// - `NoSuchAG` if `id` does not identify a custom AG.
        /// - `NotAnAgent` if `agent` is not an agent of `ticker`.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Trait>::WeightInfo::remove_agent()]
        pub fn change_group(origin, ticker: Ticker, agent: IdentityId, group: AgentGroup) -> DispatchResult {
            Self::base_change_group(origin, ticker, agent, group)
        }
    }
}

decl_event! {
    pub enum Event {
        /// An Agent Group was created.
        ///
        /// (Caller DID, AG's ticker, AG's ID, AG's permissions)
        GroupCreated(EventDid, Ticker, AGId, ExtrinsicPermissions),

        /// An Agent Group's permissions was updated.
        ///
        /// (Caller DID, AG's ticker, AG's ID, AG's new permissions)
        GroupPermissionsUpdated(EventDid, Ticker, AGId, ExtrinsicPermissions),

        /// An agent was added.
        ///
        /// (Caller/Agent DID, Agent's ticker, Agent's group)
        AgentAdded(EventDid, Ticker, AgentGroup),

        /// An agent was removed.
        ///
        /// (Caller DID, Agent's ticker, Agent's DID)
        AgentRemoved(EventDid, Ticker, IdentityId),

        /// An agent's group was changed.
        ///
        /// (Caller DID, Agent's ticker, Agent's DID, The new group of the agent)
        GroupChanged(EventDid, Ticker, IdentityId, AgentGroup),
    }
}

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// There have been too many AGs for this ticker and the ID would overflow.
        /// This won't occur in practice.
        LocalAGIdOverflow,
        /// An AG with the given `AGId` did not exist for the `Ticker`.
        NoSuchAG,
        /// The agent is not authorized to call the current extrinsic.
        UnauthorizedAgent,
        /// The provided `agent` is already an agent for the `Ticker`.
        AlreadyAnAgent,
        /// The provided `agent` is not an agent for the `Ticker`.
        NotAnAgent,
        /// The caller's secondary key does not have the required asset permission.
        SecondaryKeyNotAuthorizedForAsset,
    }
}

impl<T: Trait> polymesh_common_utilities::identity::IdentityToExternalAgents for Module<T> {
    fn accept_become_agent(
        did: IdentityId,
        from: IdentityId,
        ticker: Ticker,
        group: AgentGroup,
    ) -> DispatchResult {
        Self::ensure_agent_permissioned(ticker, from)?;
        Self::ensure_agent_group_valid(ticker, group)?;

        GroupOfAgent::try_mutate(ticker, did, |slot| -> DispatchResult {
            ensure!(slot.is_none(), Error::<T>::AlreadyAnAgent);
            *slot = Some(group);
            Ok(())
        })?;

        Self::deposit_event(Event::AgentAdded(did.for_event(), ticker, group));
        Ok(())
    }
}

impl<T: Trait> Module<T> {
    fn base_create_group(
        origin: T::Origin,
        ticker: Ticker,
        perms: ExtrinsicPermissions,
    ) -> DispatchResult {
        let did = Self::ensure_agent_asset_perms(origin, ticker)?.for_event();
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;

        // Fetch the AG id & advance the sequence.
        let id = AGIdSequence::try_mutate(ticker, |AGId(id)| {
            id.checked_add(1)
                .map(AGId)
                .ok_or(Error::<T>::LocalAGIdOverflow)
        })?;

        // Commit & emit.
        GroupPermissions::insert(ticker, id, perms.clone());
        Self::deposit_event(Event::GroupCreated(did, ticker, id, perms));
        Ok(())
    }

    fn base_set_group_permissions(
        origin: T::Origin,
        ticker: Ticker,
        id: AGId,
        perms: ExtrinsicPermissions,
    ) -> DispatchResult {
        let did = Self::ensure_agent_asset_perms(origin, ticker)?.for_event();
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;
        Self::ensure_custom_agent_group_exists(ticker, &id)?;

        // Commit & emit.
        GroupPermissions::insert(ticker, id, perms.clone());
        Self::deposit_event(Event::GroupPermissionsUpdated(did, ticker, id, perms));
        Ok(())
    }

    fn base_remove_agent(origin: T::Origin, ticker: Ticker, agent: IdentityId) -> DispatchResult {
        let did = Self::ensure_agent_asset_perms(origin, ticker)?.for_event();
        Self::try_mutate_agents_group(ticker, agent, None)?;
        Self::deposit_event(Event::AgentRemoved(did, ticker, agent));
        Ok(())
    }

    fn base_change_group(
        origin: T::Origin,
        ticker: Ticker,
        agent: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        let did = Self::ensure_agent_asset_perms(origin, ticker)?.for_event();
        Self::ensure_agent_group_valid(ticker, group)?;
        Self::try_mutate_agents_group(ticker, agent, Some(group))?;
        Self::deposit_event(Event::GroupChanged(did, ticker, agent, group));
        Ok(())
    }

    /// Ensure that `group` is a valid agent group for `ticker`.
    fn ensure_agent_group_valid(ticker: Ticker, group: AgentGroup) -> DispatchResult {
        if let AgentGroup::Custom(id) = group {
            Self::ensure_custom_agent_group_exists(ticker, &id)?;
        }
        Ok(())
    }

    /// Ensure that `id` identifies a custom AG of `ticker`.
    fn ensure_custom_agent_group_exists(ticker: Ticker, id: &AGId) -> DispatchResult {
        ensure!(
            (AGId(1)..=AGIdSequence::get(ticker)).contains(id),
            Error::<T>::NoSuchAG
        );
        Ok(())
    }

    /// Ensure that `agent` is an agent of `ticker` and set the group to `group`.
    fn try_mutate_agents_group(
        ticker: Ticker,
        agent: IdentityId,
        group: Option<AgentGroup>,
    ) -> DispatchResult {
        GroupOfAgent::try_mutate(ticker, agent, |slot| {
            ensure!(slot.is_some(), Error::<T>::NotAnAgent);
            *slot = group;
            Ok(())
        })
    }

    /// Ensures that `origin` is a permissioned agent for `ticker`.
    fn ensure_agent_asset_perms(
        origin: T::Origin,
        ticker: Ticker,
    ) -> Result<IdentityId, DispatchError> {
        let agent = Self::ensure_asset_perms(origin, &ticker)?.primary_did;
        Self::ensure_agent_permissioned(ticker, agent)?;
        Ok(agent)
    }

    /// Ensure that `origin` is permissioned for this call
    /// and the secondary key has relevant asset permissions.
    pub fn ensure_asset_perms(
        origin: T::Origin,
        ticker: &Ticker,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let data = <Identity<T>>::ensure_origin_call_permissions(origin)?;
        let skey = data.secondary_key.as_ref();

        // If `secondary_key` is None, the caller is the primary key and has all permissions.
        if let Some(sk) = skey {
            ensure!(
                sk.has_asset_permission(*ticker),
                Error::<T>::SecondaryKeyNotAuthorizedForAsset
            );
        }

        Ok(data)
    }

    /// Ensures that `agent` is permissioned for `ticker`.
    pub fn ensure_agent_permissioned(ticker: Ticker, agent: IdentityId) -> DispatchResult {
        ensure!(
            Self::agent_permissions(ticker, agent).sufficient_for(
                &<Permissions<T>>::current_pallet_name(),
                &<Permissions<T>>::current_dispatchable_name()
            ),
            Error::<T>::UnauthorizedAgent
        );
        Ok(())
    }

    /// Returns `agent`'s permission set in `ticker`.
    fn agent_permissions(ticker: Ticker, agent: IdentityId) -> ExtrinsicPermissions {
        match GroupOfAgent::get(ticker, agent) {
            None => ExtrinsicPermissions::empty(),
            Some(AgentGroup::Full) => ExtrinsicPermissions::default(),
            Some(AgentGroup::Custom(ag_id)) => GroupPermissions::get(ticker, ag_id),
            // TODO(Centril): Map these to proper permission sets.
            Some(AgentGroup::Meta) => ExtrinsicPermissions::default(),
            Some(AgentGroup::PolymeshV1CAA) => ExtrinsicPermissions::default(),
            Some(AgentGroup::PolymeshV1PIA) => ExtrinsicPermissions::default(),
        }
    }
}
