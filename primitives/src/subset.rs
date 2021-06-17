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
use core::ops::Sub;
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
    /// Inclusion of the first subset `A` into the second subset `B`.
    /// That is, `A ⊂ B`.
    Less,
    /// Set equality, `A = B`.
    Equal,
    /// The subsets are pairwise different.
    /// That is, `A ⊈ B ∧ B ⊈ A`.
    Incomparable,
    /// Inclusion of the second subset `B` into the first subset `A`.
    /// That is, `B ⊂ A`.
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

impl<A: Ord> LatticeOrd for SubsetRestriction<A> {
    fn lattice_cmp(&self, other: &Self) -> LatticeOrdering {
        // Normalize `U \ {}` to just `U`.
        let left = match self {
            Self::Except(es) if es.is_empty() => &Self::Whole,
            x => x,
        };
        let right = match other {
            Self::Except(es) if es.is_empty() => &Self::Whole,
            x => x,
        };
        // See note below (1).
        let cmp_same = |a: &BTreeSet<_>, b: &BTreeSet<_>| match (a.is_subset(b), b.is_subset(a)) {
            (true, true) => LatticeOrdering::Equal,
            (true, false) => LatticeOrdering::Less,
            (false, true) => LatticeOrdering::Greater,
            _ => LatticeOrdering::Incomparable,
        };
        // See note below (2).
        let cmp_diff = |a: &BTreeSet<_>, b: &BTreeSet<_>, n| match a.intersection(b).next() {
            Some(_) => LatticeOrdering::Incomparable,
            None => n,
        };
        match (left, right) {
            // Trivially, `U = U` holds.
            (Self::Whole, Self::Whole) => LatticeOrdering::Equal,
            // `A ⊂ U`, except when `A = U`, but as noted in (3) we don't account for that here.
            (_, Self::Whole) => LatticeOrdering::Less,
            // Same as above, but with `A` and `B` flipped.
            (Self::Whole, _) => LatticeOrdering::Greater,
            // There are 4 cases here, as identitifed in `cmp_same` (1):
            // 1. `A ⊆ B` and `B ⊆ A`, so exactly `A = B`.
            // 2. `A ⊆ B` and `B ⊈ A`, so `A ⊂ B`.
            // 3. `A ⊈ B` and `B ⊆ A`, so `B ⊂ A`.
            // 4. `A ⊈ B` and `B ⊈ A`, so they are incomparable.
            (Self::These(a), Self::These(b)) => cmp_same(a, b),
            // Same as above, but with `A` and `B` flipped.
            (Self::Except(a), Self::Except(b)) => cmp_same(b, a),
            // (2) Consider `A = {1, 2}` and `B = U \ {3}`.
            // We have that `A ∩ B = B`, hence `A ⊂ B`.
            //
            // If on the other hand, `B` excludes some element in `A`,
            // which happens when the intersection against the "inner set" `b` of `B` is non-empty,
            // then `A ⊈ B` wherefore the sets are incomparable.
            //
            // (3) For finite universes `U`, `Equal` is possible iff `B = U \ {}`
            // and every element in `U` is explicitly listed in `A`,
            // e.g., `A = { true, false }` for `bool`.
            // While this is theoretically possible, it is not accounted for in the code here.
            // Moreover, this branch isn't reached, as prior normalization gave us `B = Whole`.
            (Self::These(a), Self::Except(b)) => cmp_diff(a, b, LatticeOrdering::Less),
            // Same as above, but with `A` and `B` flipped.
            (Self::Except(a), Self::These(b)) => cmp_diff(a, b, LatticeOrdering::Greater),
        }
    }
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

    /// Checks whether there is no restriction.
    pub fn is_unrestricted(&self) -> bool {
        matches!(self, Self::Whole)
    }
}

impl<A: Clone + Ord> SubsetRestriction<A> {
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
}

#[cfg(test)]
mod tests {
    use super::{LatticeOrd, LatticeOrdering, SubsetRestriction};

    #[test]
    fn lattice_cmp() {
        let t: SubsetRestriction<bool> = SubsetRestriction::elem(true);
        let f: SubsetRestriction<bool> = SubsetRestriction::elem(false);
        let tf: SubsetRestriction<bool> =
            SubsetRestriction::These(vec![true, false].into_iter().collect());
        let ft: SubsetRestriction<bool> =
            SubsetRestriction::These(vec![false, true].into_iter().collect());
        let all = SubsetRestriction::Whole;
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
