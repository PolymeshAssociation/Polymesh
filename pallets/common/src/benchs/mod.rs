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

mod asset;
pub use asset::{make_asset, make_indivisible_asset, make_ticker, ResultTicker};

mod user;
pub use user::{PublicKey, SecretKey, User};

mod user_builder;
pub use user_builder::{AccountIdOf, UserBuilder};

use crate::traits::{identity::Config, TestUtilsFn};
use frame_system::Config as SysTrait;

pub fn user<T: Config + TestUtilsFn<<T as SysTrait>::AccountId>>(
    prefix: &'static str,
    u: u32,
) -> User<T> {
    UserBuilder::<T>::default()
        .generate_did()
        .seed(u)
        .build(prefix)
}

pub fn user_without_did<T: Config + TestUtilsFn<<T as SysTrait>::AccountId>>(
    prefix: &'static str,
    u: u32,
) -> User<T> {
    UserBuilder::<T>::default().seed(u).build(prefix)
}

pub fn cdd_provider<T: Config + TestUtilsFn<<T as SysTrait>::AccountId>>(
    prefix: &'static str,
    u: u32,
) -> User<T> {
    UserBuilder::<T>::default()
        .generate_did()
        .seed(u)
        .become_cdd_provider()
        .build(prefix)
}
