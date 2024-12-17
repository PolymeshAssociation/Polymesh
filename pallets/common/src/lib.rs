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

#![cfg_attr(not(feature = "std"), no_std)]

pub mod traits;
pub use traits::{
    asset, balances, compliance_manager, governance_group, group, identity, multisig, nft,
    portfolio, transaction_payment, CommonConfig,
};
pub mod context;
pub use context::Context;

#[cfg(feature = "runtime-benchmarks")]
pub mod benchs;

use core::ops::Add;
use frame_support::codec::{Decode, Encode};
use frame_support::traits::Get;
use scale_info::TypeInfo;
#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

/// Use `GetExtra` as the trait bounds for pallet `Config` parameters
/// that will be used for bounded collections.
pub trait GetExtra<T>: Get<T> + Clone + core::fmt::Debug + Default + PartialEq + Eq {}

/// ConstSize type wrapper.
///
/// This allows the use of Bounded collections in extrinsic parameters.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct ConstSize<const T: u32>;

impl<const T: u32> Get<u32> for ConstSize<T> {
    fn get() -> u32 {
        T
    }
}

impl<const T: u32> GetExtra<u32> for ConstSize<T> {}

/// Either a block number, or nothing.
#[derive(Copy, Clone, PartialEq, Eq, Encode, Decode, TypeInfo, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum MaybeBlock<BlockNumber> {
    Some(BlockNumber),
    None,
}

impl<T> Default for MaybeBlock<T> {
    fn default() -> Self {
        Self::None
    }
}

impl<T: Add<Output = T>> Add<T> for MaybeBlock<T> {
    type Output = Self;
    fn add(self, rhs: T) -> Self::Output {
        match self {
            MaybeBlock::Some(lhs) => MaybeBlock::Some(lhs + rhs),
            MaybeBlock::None => MaybeBlock::None,
        }
    }
}
