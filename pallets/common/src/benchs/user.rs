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

use crate::traits::identity::Trait;
use polymesh_primitives::{crypto::native_schnorrkel, IdentityId, InvestorUid};

use frame_system::RawOrigin;
use sp_core::sr25519::Signature;
use sp_runtime::traits::StaticLookup;

pub use schnorrkel::keys::{PublicKey, SecretKey};

/// Helper class to create accounts and its DID to simplify benchmarks and UT.
pub struct User<T: Trait> {
    pub account: T::AccountId,
    pub secret: Option<SecretKey>,
    pub origin: RawOrigin<T::AccountId>,
    pub uid: Option<InvestorUid>,
    pub did: Option<IdentityId>,
}

impl<T: Trait> User<T> {
    pub fn did(self: &Self) -> IdentityId {
        self.did.clone().expect("User without DID")
    }

    pub fn uid(self: &Self) -> InvestorUid {
        self.uid.clone().expect("User without UID")
    }

    pub fn account(self: &Self) -> T::AccountId {
        self.account.clone()
    }

    pub fn origin(self: &Self) -> RawOrigin<T::AccountId> {
        self.origin.clone()
    }

    pub fn lookup(self: &Self) -> <T::Lookup as StaticLookup>::Source {
        T::Lookup::unlookup(self.account.clone())
    }

    pub fn sign(&self, message: &[u8]) -> Option<Signature> {
        self.secret
            .as_ref()
            .and_then(|sk| native_schnorrkel::sign(sk.to_bytes(), message))
    }
}
