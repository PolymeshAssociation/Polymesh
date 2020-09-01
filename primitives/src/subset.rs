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
pub enum Subset<A: Ord> {
    /// The set of all elements.
    All,
    /// A subset of given elements. It is strictly contained in [`Subset::All`].
    Elems(BTreeSet<A>),
}

impl<A: Ord> Default for Subset<A> {
    fn default() -> Self {
        Self::All
    }
}

impl<A> LatticeOrd for Subset<A>
where
    A: Clone + Ord + PartialEq,
{
    fn lattice_cmp(&self, other: &Self) -> LatticeOrdering {
        match (self, other) {
            (Subset::All, Subset::All) => LatticeOrdering::Equal,
            (_, Subset::All) => LatticeOrdering::Less,
            (Subset::All, _) => LatticeOrdering::Greater,
            (Subset::Elems(a), Subset::Elems(b)) => match (a.is_subset(&b), b.is_subset(&a)) {
                (true, true) => LatticeOrdering::Equal,
                (true, false) => LatticeOrdering::Less,
                (false, true) => LatticeOrdering::Greater,
                _ => LatticeOrdering::Incomparable,
            },
        }
    }
}

impl<A> FromIterator<A> for Subset<A>
where
    A: Clone + Ord + PartialEq,
{
    fn from_iter<I: IntoIterator<Item = A>>(iter: I) -> Subset<A> {
        let mut set = BTreeSet::new();
        set.extend(iter);
        Subset::Elems(set)
    }
}

impl<A> Subset<A>
where
    A: Clone + Ord + PartialEq,
{
    /// Constructs the empty subset.
    pub fn empty() -> Self {
        Subset::Elems(BTreeSet::new())
    }

    /// Constructs a subset with one element.
    pub fn elem(a: A) -> Self {
        Subset::Elems(BTreeSet::from_iter(iter::once(a)))
    }

    /// Computes whether the first subset is greater than or equal to the second subset.
    pub fn ge(&self, other: &Self) -> bool {
        matches!(
            self.lattice_cmp(other),
            LatticeOrdering::Greater | LatticeOrdering::Equal
        )
    }

    /// Returns the number of elements in the subset if known. Otherwise returns `None`.
    pub fn elems_len(&self) -> Option<usize> {
        if let Self::Elems(elems) = self {
            Some(elems.len())
        } else {
            None
        }
    }

    /// Set union operation on `self` and `other`.
    pub fn union(&self, other: &Self) -> Self {
        match (self, other) {
            (Subset::All, _) | (_, Subset::All) => Subset::All,
            (Subset::Elems(elems1), Subset::Elems(elems2)) => {
                Subset::Elems(elems1.union(elems2).cloned().collect())
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{LatticeOrd, LatticeOrdering, Subset};
    use std::iter::FromIterator;

    #[test]
    fn lattice_cmp() {
        let t: Subset<bool> = Subset::elem(true);
        let f: Subset<bool> = Subset::elem(false);
        let tf: Subset<bool> = Subset::from_iter(vec![true, false].into_iter());
        let ft: Subset<bool> = Subset::from_iter(vec![false, true].into_iter());
        let all = Subset::All;
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
