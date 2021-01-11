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

mod user;
use crate::traits::identity::Trait;
use polymesh_primitives::Ticker;
use sp_std::{convert::TryFrom, prelude::*};
pub use user::{PublicKey, SecretKey, User};

mod user_builder;
pub use user_builder::{uid_from_name_and_idx, UserBuilder};

pub fn user<T: Trait>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default()
        .generate_did()
        .seed(u)
        .build(prefix)
}

/// Given a number, this function generates a ticker with
/// A-Z, least number of characters in Lexicographic order
pub fn generate_ticker(n: u64) -> Ticker {
    fn calc_base26(n: u64, base_26: &mut Vec<u8>) {
        if n >= 26 {
            // Subtracting 1 is not required and shouldn't be done for a proper base_26 conversion
            // However, without this hack, B will be the first char after a bump in number of chars.
            // i.e. the sequence will go A,B...Z,BA,BB...ZZ,BAA. We want the sequence to start with A.
            // Subtracting 1 here means we are doing 1 indexing rather than 0.
            // i.e. A = 1, B = 2 instead of A = 0, B = 1
            calc_base26((n / 26) - 1, base_26);
        }
        let character = n % 26 + 65;
        base_26.push(character as u8);
    }
    let mut base_26 = Vec::new();
    calc_base26(n, &mut base_26);
    Ticker::try_from(base_26.as_slice()).unwrap()
}
