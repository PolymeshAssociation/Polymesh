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
use sp_std::prelude::Vec;

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

/// The type of subsets of a set of elements of type `A`.
#[derive(Encode, Decode, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum Subset<A> {
    /// The set of all elements.
    All,
    /// A subset of given elements.
    Elems(Vec<A>),
}

impl<A> Default for Subset<A> {
    fn default() -> Self {
        Self::All
    }
}

impl<A> LatticeOrd for Subset<A>
where
    A: Clone + PartialEq,
{
    fn lattice_cmp(&self, other: &Self) -> LatticeOrdering {
        match (self, other) {
            (Subset::All, Subset::All) => LatticeOrdering::Equal,
            (_, Subset::All) => LatticeOrdering::Less,
            (Subset::All, _) => LatticeOrdering::Greater,
            (Subset::Elems(a), Subset::Elems(b)) => {
                let mut a_minus_b = a.clone();
                // Subtract `b` from a copy of `a`.
                a_minus_b.retain(|elem| !b.contains(elem));
                if a_minus_b.is_empty() {
                    LatticeOrdering::Less
                } else {
                    let mut b_minus_a = b.clone();
                    // Subtract `a` from a copy of `b`.
                    b_minus_a.retain(|elem| !a.contains(elem));
                    if b_minus_a.is_empty() {
                        LatticeOrdering::Greater
                    } else {
                        LatticeOrdering::Incomparable
                    }
                }
            }
        }
    }
}

impl<A> Subset<A>
where
    A: Clone + PartialEq,
{
    /// Constructs the empty subset.
    pub fn empty() -> Self {
        Subset::Elems(Vec::new())
    }

    /// Constructs a subset with one element.
    pub fn elem(a: A) -> Self {
        Subset::Elems(Vec::from([a]))
    }

    /// Computes whether the first subset is greater than or equal to the second subset.
    pub fn ge(&self, other: &Self) -> bool {
        let o = self.lattice_cmp(other);
        o == LatticeOrdering::Greater || o == LatticeOrdering::Equal
    }

    /// Returns the number of elements in the subset if known. Otherwise returns `None`.
    pub fn elems_len(&self) -> Option<usize> {
        if let Self::Elems(elems) = self {
            Some(elems.len())
        } else {
            None
        }
    }
}
