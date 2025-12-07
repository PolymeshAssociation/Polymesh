// This file is part of the Polymesh distribution (https://github.com/PolymeshAssociation/Polymesh).
// Copyright (c) 2020 Polymesh Association
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, version 3.
//
// This program is distributed in the hope that it will be useful, but
// WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU
// General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use codec::{Decode, DecodeWithMemTracking, Encode};
use scale_info::TypeInfo;
use serde::{Deserialize, Serialize};
use sp_std::collections::btree_set::BTreeSet;
use sp_std::iter::{self, FromIterator};

/// The type of subsets of an open set of elements of type `A` where the whole set is always
/// considered to be bigger than any finite set of its elements. This is true for infinite
/// sets. When talking about finite sets, we have to add that they are _open_.
#[derive(Decode, DecodeWithMemTracking, Encode)]
#[derive(Clone, Debug, Default, Eq, PartialEq, TypeInfo)]
#[derive(Serialize, Deserialize)]
pub enum SubsetRestriction<A: Ord> {
    /// No restrictions, the whole set.
    #[default]
    Whole,
    /// Exactly these elements, and no others.
    These(BTreeSet<A>),
    /// The whole set except these elements.
    Except(BTreeSet<A>),
}

impl<A: Ord> SubsetRestriction<A> {
    /// Constructs the empty subset.
    pub fn empty() -> Self {
        Self::These(BTreeSet::new())
    }

    /// Constructs a subset with everything but one element.
    pub fn except(a: A) -> Self {
        Self::excepts(iter::once(a))
    }

    /// Constructs a subset with everything but these elements.
    pub fn excepts(it: impl IntoIterator<Item = A>) -> Self {
        Self::Except(BTreeSet::from_iter(it))
    }

    /// Constructs a subset with one element.
    pub fn elem(a: A) -> Self {
        Self::elems(iter::once(a))
    }

    /// Constructs a subset from an iterator over elements.
    pub fn elems(it: impl IntoIterator<Item = A>) -> Self {
        Self::These(BTreeSet::from_iter(it))
    }

    /// Returns the complexity of the subset.
    pub fn complexity(&self) -> usize {
        self.inner().map_or(0, |es| es.len())
    }

    /// Returns the inner describing finite sets if any.
    pub fn inner(&self) -> Option<&BTreeSet<A>> {
        match self {
            Self::Whole => None,
            Self::These(es) | Self::Except(es) => Some(es),
        }
    }

    /// Checks whether there is no restriction.
    pub fn is_unrestricted(&self) -> bool {
        matches!(self, Self::Whole)
    }

    /// Folds every element in the inner sets or return `init` for `Whole`.
    pub fn fold<B>(&self, init: B, f: impl FnMut(B, &A) -> B) -> B {
        match self.inner() {
            Some(set) => set.iter().fold(init, f),
            None => init,
        }
    }
}
