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
use polymesh_primitives::{IdentityId, InvestorUid};

use frame_system::RawOrigin;
use sp_core::sr25519::Signature;
use sp_runtime::traits::StaticLookup;

#[cfg(feature = "std")]
const SIGNING_CTX: &[u8] = b"substrate";

pub type SecretKey = [u8; 64]; // Only in sr25519
pub type PublicKey = [u8; 32]; // Only in sr25519

/// Helper class to create accounts and its DID to simplify benchmarks and UT.
pub struct User<T: Trait> {
    pub account: T::AccountId,
    pub secret: SecretKey,
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

    #[cfg(feature = "std")]
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sk = schnorrkel::keys::SecretKey::from_bytes(&self.secret[..])
            .expect("Invalid sr25519 secret key");
        let pair = schnorrkel::Keypair::from(sk);
        let context = schnorrkel::signing_context(SIGNING_CTX);
        let raw_signature = pair.sign(context.bytes(message)).to_bytes();

        Signature::from_raw(raw_signature)
    }

    #[cfg(not(feature = "std"))]
    pub fn sign(&self, _message: &[u8]) -> Signature {
        panic!("Cannot sign without 'std' support");
    }
}
