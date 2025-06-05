#![allow(missing_docs)]
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

use crate::IdentityId;
use codec::{Decode, Encode};
use scale_info::TypeInfo;
use sp_runtime::DispatchResult;
use sp_std::{
    cmp::{Eq, Ordering, PartialEq},
    vec::Vec,
};

/// The number of group members.
pub type MemberCount = u32;

#[derive(Encode, Decode, TypeInfo, Default, Clone, PartialEq, Eq, Debug)]
pub struct InactiveMember<Moment> {
    pub id: IdentityId,
    pub deactivated_at: Moment,
    pub expiry: Option<Moment>,
}

impl<M> PartialOrd for InactiveMember<M>
where
    M: Eq,
{
    fn partial_cmp(&self, other: &InactiveMember<M>) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl<M> PartialOrd<IdentityId> for InactiveMember<M>
where
    M: Eq,
{
    fn partial_cmp(&self, other: &IdentityId) -> Option<Ordering> {
        Some(self.id.cmp(other))
    }
}

impl<M> Ord for InactiveMember<M>
where
    M: Eq,
{
    fn cmp(&self, other: &InactiveMember<M>) -> Ordering {
        self.id.cmp(&other.id)
    }
}

impl<M> PartialEq<IdentityId> for InactiveMember<M> {
    fn eq(&self, other: &IdentityId) -> bool {
        self.id.eq(other)
    }
}

impl<M: Default> From<IdentityId> for InactiveMember<M> {
    fn from(id: IdentityId) -> Self {
        InactiveMember {
            id,
            ..Default::default()
        }
    }
}

pub trait GroupTrait<Moment: PartialOrd + Copy> {
    /// Retrieve members
    fn get_members() -> Vec<IdentityId>;

    /// Retrieve valid members: active and revoked members.
    fn get_inactive_members() -> Vec<InactiveMember<Moment>>;

    /// It moves `who` from active to inactive group.
    /// Any generated claim from `at` is considered as invalid. If `at` is `None` it will use `now`
    /// by default.
    /// If `expiry` is some value, that member will be removed automatically from this group at the
    /// specific moment, and any generated claim will be invalidated.
    fn disable_member(
        who: IdentityId,
        expiry: Option<Moment>,
        at: Option<Moment>,
    ) -> DispatchResult;

    /// Adds a member `who` to the group.
    fn add_member(who: IdentityId) -> DispatchResult;

    /// It returns the current "active members" and any "inactive member" which its
    /// expiration time-stamp is greater than `moment`.
    fn get_valid_members_at(moment: Moment) -> Vec<IdentityId> {
        Self::get_active_members()
            .into_iter()
            .chain(
                Self::get_inactive_members()
                    .into_iter()
                    .filter(|m| !Self::is_member_expired(&m, moment))
                    .map(|m| m.id),
            )
            .collect::<Vec<_>>()
    }

    fn is_member_expired(member: &InactiveMember<Moment>, now: Moment) -> bool {
        if let Some(expiry) = member.expiry {
            expiry <= now
        } else {
            false
        }
    }

    #[inline]
    fn get_active_members() -> Vec<IdentityId> {
        Self::get_members()
    }

    /// Current set size
    #[inline]
    fn member_count() -> usize {
        Self::get_members().len()
    }

    #[inline]
    fn is_member(member_id: &IdentityId) -> bool {
        Self::get_members().contains(member_id)
    }
}
