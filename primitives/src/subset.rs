// This file is part of the Polymesh distribution (https://github.com/PolymathNetwork/Polymesh).
// Copyright (c) 2020 Polymath
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

use core::ops::Sub;
use codec::{Decode, Encode};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{
    collections::btree_set::BTreeSet,
    iter::{self, FromIterator},
};

/// Ordering in a lattice, for example, the lattice of subsets of a set.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum LatticeOrdering {
    /// Inclusion of the first subset into the second subset.
    Less,
    /// Set equality.
    Equal,
    /// The subsets are pairwise different.
    Incomparable,
    /// Inclusion of the second subset into the first subset.
    Greater,
}

/// The lattice order trait.
pub trait LatticeOrd {
    /// The lattice comparison.
    fn lattice_cmp(&self, other: &Self) -> LatticeOrdering;
}

/// The type of subsets of an open set of elements of type `A` where the whole set is always
/// considered to be bigger than any finite set of its elements. This is true for infinite
/// sets. When talking about finite sets, we have to add that they are _open_.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum SubsetRestriction<A: Ord> {
    /// No restrictions, the whole set.
    Whole,
    /// Exactly these elements, and no others.
    These(BTreeSet<A>),
    /// The whole set except these elements.
    Except(BTreeSet<A>),
}

impl<A: Ord> Default for SubsetRestriction<A> {
    fn default() -> Self {
        Self::Whole
    }
}

impl<A> LatticeOrd for SubsetRestriction<A>
where
    A: Clone + Ord + PartialEq,
{
    fn lattice_cmp(&self, other: &Self) -> LatticeOrdering {
        let left = match self {
            Self::Except(es) if es.is_empty() => &Self::Whole,
            x => x,
        };
        let right = match other {
            Self::Except(es) if es.is_empty() => &Self::Whole,
            x => x,
        };
        let cmp_same = |a: &BTreeSet<_>, b: &BTreeSet<_>| match (a.is_subset(b), b.is_subset(a)) {
            (true, true) => LatticeOrdering::Equal,
            (true, false) => LatticeOrdering::Less,
            (false, true) => LatticeOrdering::Greater,
            _ => LatticeOrdering::Incomparable,
        };
        let cmp_diff = |a: &BTreeSet<_>, b: &BTreeSet<_>| match a.intersection(b).next() {
            Some(_) => LatticeOrdering::Incomparable,
            None => LatticeOrdering::Less,
        };
        match (left, right) {
            (Self::Whole, Self::Whole) => LatticeOrdering::Equal,
            (_, Self::Whole) => LatticeOrdering::Less,
            (Self::Whole, _) => LatticeOrdering::Greater,
            (Self::These(a), Self::These(b)) => cmp_same(a, b),
            (Self::Except(a), Self::Except(b)) => cmp_same(b, a),
            (Self::These(a), Self::Except(b)) => cmp_diff(a, b),
            (Self::Except(a), Self::These(b)) => cmp_diff(b, a),
        }
    }
}

impl<A> SubsetRestriction<A>
where
    A: Clone + Ord + PartialEq,
{
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

    /// Computes whether the first subset is greater than or equal to the second subset.
    pub fn ge(&self, other: &Self) -> bool {
        matches!(
            self.lattice_cmp(other),
            LatticeOrdering::Greater | LatticeOrdering::Equal
        )
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

    /// Set union operation on `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (Self::Whole, _) | (_, Self::Whole) => Self::Whole,
            (Self::These(l), Self::These(r)) => Self::These(l.union(&r).cloned().collect()),
            (Self::Except(l), Self::Except(r)) => {
                Self::Except(l.intersection(&r).cloned().collect())
            }
            (Self::These(l), Self::Except(r)) | (Self::Except(r), Self::These(l)) => {
                Self::Except(r.sub(&l))
            }
        }
    }

    /// Checks whether there is no restriction.
    pub fn is_unrestricted(&self) -> bool {
        matches!(self, Self::Whole)
    }
}

#[cfg(test)]
mod tests {
    use super::{LatticeOrd, LatticeOrdering, SubsetRestriction};

    #[test]
    fn lattice_cmp() {
        let t: SubsetRestriction<bool> = SubsetRestriction::elem(true);
        let f: SubsetRestriction<bool> = SubsetRestriction::elem(false);
        let tf: SubsetRestriction<bool> =
            SubsetRestriction(Some(vec![true, false].into_iter().collect()));
        let ft: SubsetRestriction<bool> =
            SubsetRestriction(Some(vec![false, true].into_iter().collect()));
        let all = SubsetRestriction(None);
        assert_eq!(t.lattice_cmp(&t), LatticeOrdering::Equal);
        assert_eq!(t.lattice_cmp(&tf), LatticeOrdering::Less);
        assert_eq!(f.lattice_cmp(&tf), LatticeOrdering::Less);
        assert_eq!(t.lattice_cmp(&all), LatticeOrdering::Less);
        assert_eq!(tf.lattice_cmp(&all), LatticeOrdering::Less);
        assert_eq!(tf.lattice_cmp(&ft), LatticeOrdering::Equal);
        assert_eq!(tf.lattice_cmp(&t), LatticeOrdering::Greater);
        assert_eq!(tf.lattice_cmp(&f), LatticeOrdering::Greater);
        assert_eq!(t.lattice_cmp(&f), LatticeOrdering::Incomparable);
    }
}
