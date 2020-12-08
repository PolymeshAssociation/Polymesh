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

use crate::traits::{
    group::GroupTrait,
    identity::{IdentityFnTrait, Trait},
};
use polymesh_primitives::{IdentityId, InvestorUid};

use codec::Encode;
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use sp_core::sr25519::Signature;
use sp_runtime::traits::StaticLookup;
use sp_std::{convert::TryInto, prelude::*};

#[cfg(not(feature = "std"))]
use frame_benchmarking::account;
#[cfg(not(feature = "std"))]
const SEED: u32 = 0;

#[cfg(feature = "std")]
use codec::Decode;
#[cfg(feature = "std")]
use sp_core::{crypto::Pair as TPair, sr25519::Pair};
#[cfg(feature = "std")]
use sp_io::hashing::blake2_256;
#[cfg(feature = "std")]
const SIGNING_CTX: &[u8] = b"substrate";

pub fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

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

pub struct UserBuilder<T: Trait> {
    account: Option<T::AccountId>,
    uid: Option<InvestorUid>,
    did: Option<IdentityId>,
    balance: T::Balance,
    generate_did: bool,
    as_cdd_provider: bool,
}

impl<T: Trait> UserBuilder<T> {
    /// Create an account based on the builder configuration.
    pub fn build(self, name: &'static str, u: u32) -> User<T> {
        let (account, secret) = self
            .account
            .clone()
            .map_or_else(|| Self::make_key_pair(name, u), |acc| (acc, [0u8; 64]));
        let origin = RawOrigin::Signed(account.clone());
        let amount: u32 = self.balance.try_into().unwrap_or_default() as u32;
        let _ = T::Balances::make_free_balance_be(&account, amount.into());

        // Generate DID or use the one given.
        let uid = self.uid.unwrap_or_else(|| uid_from_name_and_idx(name, u));
        let did = self.did.or_else(|| {
            self.generate_did
                .then(|| Self::make_did(account.clone(), uid.clone()))
        });

        // Become a CDD provider.
        self.as_cdd_provider.then(|| {
            T::CddServiceProviders::add_member(did.clone().unwrap())
                .expect("User cannot be added as CDD provider")
        });

        User {
            account,
            secret,
            origin,
            did,
            uid: Some(uid),
        }
    }

    pub fn generate_did(self) -> Self {
        assert!(self.did.is_none());
        let mut new = self;
        new.generate_did = true;
        new
    }

    pub fn become_cdd_provider(self) -> Self {
        assert!(self.generate_did == true || self.did.is_some());
        let mut new = self;
        new.as_cdd_provider = true;
        new
    }

    pub fn uid(self, u: InvestorUid) -> Self {
        let mut new = self;
        new.uid = Some(u);
        new
    }

    pub fn did(self, did: IdentityId) -> Self {
        assert!(self.generate_did == false);
        let mut new = self;
        new.did = Some(did);
        new.generate_did = false;
        new
    }

    pub fn account<ACC: Into<T::AccountId>>(self, acc: ACC) -> Self {
        let mut new = self;
        new.account = Some(acc.into());
        new
    }

    pub fn balance<B: Into<T::Balance>>(self, b: B) -> Self {
        let mut new = self;
        new.balance = b.into();
        new
    }

    #[cfg(not(feature = "std"))]
    fn make_key_pair(name: &'static str, u: u32) -> (T::AccountId, SecretKey) {
        let public: T::AccountId = account(name, u, SEED);
        let secret = [0u8; 64];

        (public, secret)
    }

    #[cfg(feature = "std")]
    fn make_key_pair(name: &'static str, u: u32) -> (T::AccountId, SecretKey) {
        let seed = (name, u).using_encoded(blake2_256);
        let pair = Pair::from_seed(&seed);
        let keypair = pair.as_ref();

        let secret = keypair.secret.to_bytes();
        let public = keypair.public.to_bytes();
        let id = T::AccountId::decode(&mut &public[..]).unwrap();

        (id, secret)
    }

    /// Create a DID for account `acc` using the specified investor ID.
    fn make_did(acc: T::AccountId, uid: InvestorUid) -> IdentityId {
        let _ = T::IdentityFn::register_did(acc.clone(), uid, vec![]);
        T::IdentityFn::get_identity(&acc).unwrap()
    }
}

// Derive macro from `Default` is not supported due to trait T.
impl<T: Trait> Default for UserBuilder<T> {
    fn default() -> Self {
        Self {
            account: None,
            uid: None,
            did: None,
            balance: 5_000_000u128.into(),
            generate_did: false,
            as_cdd_provider: false,
        }
    }
}
