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
#![feature(iter_advance_by)]

#[cfg(feature = "runtime-benchmarks")]
pub mod benchmarking;

use core::array::IntoIter;
use frame_support::{
    decl_error, decl_module, decl_storage,
    dispatch::{DispatchError, DispatchResult},
    ensure,
};
use pallet_identity::PermissionedCallOriginData;
pub use polymesh_common_utilities::traits::external_agents::{Config, Event, WeightInfo};
use polymesh_primitives::agent::{AGId, AgentGroup};
use polymesh_primitives::{
    ExtrinsicPermissions, IdentityId, PalletPermissions, SubsetRestriction, Ticker,
};
use sp_std::prelude::*;

type Identity<T> = pallet_identity::Module<T>;
type Permissions<T> = pallet_permissions::Module<T>;

decl_storage! {
    trait Store for Module<T: Config> as ExternalAgent {
        /// The next per-`Ticker` AG ID in the sequence.
        ///
        /// The full ID is defined as a combination of `Ticker` and a number in this sequence,
        /// which starts from 1, rather than 0.
        pub AGIdSequence get(fn agent_group_id_sequence):
            map hasher(blake2_128_concat) Ticker
                => AGId;

        /// Maps an agent (`IdentityId`) to all all `Ticker`s they belong to, if any.
        pub AgentOf get(fn agent_of):
            double_map
                hasher(blake2_128_concat) IdentityId,
                hasher(blake2_128_concat) Ticker
                => ();

        /// Maps agents (`IdentityId`) for a `Ticker` to what AG they belong to, if any.
        pub GroupOfAgent get(fn agents):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) IdentityId
                => Option<AgentGroup>;

        /// Maps a `Ticker` to the number of `Full` agents for it.
        pub NumFullAgents get(fn num_full_agents):
            map hasher(blake2_128_concat) Ticker
                => u32;

        /// For custom AGs of a `Ticker`, maps to what permissions an agent in that AG would have.
        pub GroupPermissions get(fn permissions):
            double_map
                hasher(blake2_128_concat) Ticker,
                hasher(twox_64_concat) AGId
                => Option<ExtrinsicPermissions>;
    }
}

decl_module! {
    pub struct Module<T: Config> for enum Call where origin: T::Origin {
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
        #[weight = <T as Config>::WeightInfo::create_group(perms.complexity() as u32)]
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
        #[weight = <T as Config>::WeightInfo::set_group_permissions(perms.complexity() as u32)]
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
        /// - `RemovingLastFullAgent` if `agent` is the last full one.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = <T as Config>::WeightInfo::remove_agent()]
        pub fn remove_agent(origin, ticker: Ticker, agent: IdentityId) -> DispatchResult {
            Self::base_remove_agent(origin, ticker, agent)
        }

        /// Abdicate agentship for `ticker`.
        ///
        /// # Arguments
        /// - `ticker` of which the caller is an agent.
        ///
        /// # Errors
        /// - `NotAnAgent` if the caller is not an agent of `ticker`.
        /// - `RemovingLastFullAgent` if the caller is the last full agent.
        ///
        /// # Permissions
        /// * Asset
        #[weight = <T as Config>::WeightInfo::abdicate()]
        pub fn abdicate(origin, ticker: Ticker) -> DispatchResult {
            Self::base_abdicate(origin, ticker)
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
        /// - `RemovingLastFullAgent` if `agent` was a `Full` one and is being demoted.
        ///
        /// # Permissions
        /// * Asset
        /// * Agent
        #[weight = match group {
            AgentGroup::Custom(_) => <T as Config>::WeightInfo::change_group_custom(),
            _ => <T as Config>::WeightInfo::change_group_builtin(),
        }]
        pub fn change_group(origin, ticker: Ticker, agent: IdentityId, group: AgentGroup) -> DispatchResult {
            Self::base_change_group(origin, ticker, agent, group)
        }
    }
}

decl_error! {
    pub enum Error for Module<T: Config> {
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
        /// This agent is the last full one, and it's being removed,
        /// making the asset orphaned.
        RemovingLastFullAgent,
        /// The counter for full agents will overflow.
        /// This should never happen in practice, but is theoretically possible.
        NumFullAgentsOverflow,
        /// The caller's secondary key does not have the required asset permission.
        SecondaryKeyNotAuthorizedForAsset,
    }
}

impl<T: Config> polymesh_common_utilities::identity::IdentityToExternalAgents for Module<T> {
    fn accept_become_agent(
        did: IdentityId,
        from: IdentityId,
        ticker: Ticker,
        group: AgentGroup,
    ) -> DispatchResult {
        Self::ensure_agent_permissioned(ticker, from)?;
        Self::ensure_agent_group_valid(ticker, group)?;
        ensure!(
            GroupOfAgent::get(ticker, did).is_none(),
            Error::<T>::AlreadyAnAgent
        );

        Self::unchecked_add_agent(ticker, did, group)?;
        Self::deposit_event(Event::AgentAdded(did.for_event(), ticker, group));
        Ok(())
    }
}

impl<T: Config> Module<T> {
    fn base_create_group(
        origin: T::Origin,
        ticker: Ticker,
        perms: ExtrinsicPermissions,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin, ticker)?.for_event();
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;

        // Fetch the AG id & advance the sequence.
        let id = AGIdSequence::try_mutate(ticker, |AGId(id)| -> Result<_, DispatchError> {
            *id = id.checked_add(1).ok_or(Error::<T>::LocalAGIdOverflow)?;
            Ok(AGId(*id))
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
        let did = Self::ensure_perms(origin, ticker)?.for_event();
        <Identity<T>>::ensure_extrinsic_perms_length_limited(&perms)?;
        Self::ensure_custom_agent_group_exists(ticker, &id)?;

        // Commit & emit.
        GroupPermissions::insert(ticker, id, perms.clone());
        Self::deposit_event(Event::GroupPermissionsUpdated(did, ticker, id, perms));
        Ok(())
    }

    fn base_remove_agent(origin: T::Origin, ticker: Ticker, agent: IdentityId) -> DispatchResult {
        let did = Self::ensure_perms(origin, ticker)?.for_event();
        Self::try_mutate_agents_group(ticker, agent, None)?;
        Self::deposit_event(Event::AgentRemoved(did, ticker, agent));
        Ok(())
    }

    fn base_abdicate(origin: T::Origin, ticker: Ticker) -> DispatchResult {
        let did = Self::ensure_asset_perms(origin, &ticker)?.primary_did;
        Self::try_mutate_agents_group(ticker, did, None)?;
        Self::deposit_event(Event::AgentRemoved(did.for_event(), ticker, did));
        Ok(())
    }

    fn base_change_group(
        origin: T::Origin,
        ticker: Ticker,
        agent: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        let did = Self::ensure_perms(origin, ticker)?.for_event();
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

            match (*slot, group) {
                // Identity transition. No change in count.
                (Some(AgentGroup::Full), Some(AgentGroup::Full)) => {}
                // Demotion/Removal. Count is decrementing.
                (Some(AgentGroup::Full), _) => Self::dec_full_count(ticker)?,
                // Promotion. Count is incrementing.
                (_, Some(AgentGroup::Full)) => Self::inc_full_count(ticker)?,
                // Just a change in groups.
                _ => {}
            }

            // Removal
            if group.is_none() {
                AgentOf::remove(agent, ticker);
            }

            *slot = group;
            Ok(())
        })
    }

    pub fn unchecked_add_agent(
        ticker: Ticker,
        did: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        if let AgentGroup::Full = group {
            Self::inc_full_count(ticker)?;
        }
        GroupOfAgent::insert(ticker, did, group);
        AgentOf::insert(did, ticker, ());
        Ok(())
    }

    /// Add `agent` for `ticker` unless it already is.
    pub fn add_agent_if_not(
        ticker: Ticker,
        agent: IdentityId,
        group: AgentGroup,
    ) -> DispatchResult {
        if let None = Self::agents(ticker, agent) {
            Self::unchecked_add_agent(ticker, agent, group)?;
        }
        Ok(())
    }

    /// Decrement the full agent count, or error on < 1.
    fn dec_full_count(ticker: Ticker) -> DispatchResult {
        NumFullAgents::try_mutate(ticker, |n| {
            *n = n
                .checked_sub(1)
                .filter(|&x| x > 0)
                .ok_or(Error::<T>::RemovingLastFullAgent)?;
            Ok(())
        })
    }

    /// Increment the full agent count, or error on overflow.
    fn inc_full_count(ticker: Ticker) -> DispatchResult {
        NumFullAgents::try_mutate(ticker, |n| {
            *n = n.checked_add(1).ok_or(Error::<T>::NumFullAgentsOverflow)?;
            Ok(())
        })
    }

    /// Ensures that `origin` is a permissioned agent for `ticker`.
    pub fn ensure_perms(origin: T::Origin, ticker: Ticker) -> Result<IdentityId, DispatchError> {
        Self::ensure_agent_asset_perms(origin, ticker).map(|d| d.primary_did)
    }

    /// Ensures that `origin` is a permissioned agent for `ticker`.
    pub fn ensure_agent_asset_perms(
        origin: T::Origin,
        ticker: Ticker,
    ) -> Result<PermissionedCallOriginData<T::AccountId>, DispatchError> {
        let data = Self::ensure_asset_perms(origin, &ticker)?;
        Self::ensure_agent_permissioned(ticker, data.primary_did)?;
        Ok(data)
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
        let pallet = |p: &str| PalletPermissions::entire_pallet(p.into());
        let in_pallet = |p: &str, dns| PalletPermissions::new(p.into(), dns);
        fn elems<T: Ord, const N: usize>(elems: [T; N]) -> SubsetRestriction<T> {
            SubsetRestriction::elems(IntoIter::new(elems))
        }
        match GroupOfAgent::get(ticker, agent) {
            None => ExtrinsicPermissions::empty(),
            Some(AgentGroup::Full) => ExtrinsicPermissions::default(),
            Some(AgentGroup::Custom(ag_id)) => {
                GroupPermissions::get(ticker, ag_id).unwrap_or_else(ExtrinsicPermissions::empty)
            }
            // Anything but extrinsics in this pallet & `accept_authorization`.
            Some(AgentGroup::ExceptMeta) => SubsetRestriction::excepts(IntoIter::new([
                // `Identity::accept_authorization` needs to be excluded.
                in_pallet(
                    "Identity",
                    SubsetRestriction::elem("accept_authorization".into()),
                ),
                // `ExternalAgents` needs to be excluded.
                pallet("ExternalAgents"),
            ])),
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
