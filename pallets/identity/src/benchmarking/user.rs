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

use crate::Config;
use codec::{Decode, Encode};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use polymesh_primitives::traits::{group::GroupTrait, IdentityFnTrait};
use polymesh_primitives::{crypto::native_schnorrkel, IdentityId};
use schnorrkel::keys::SecretKey;
use schnorrkel::{ExpansionMode, MiniSecretKey};
use sp_core::sr25519::Signature;
use sp_io::hashing::blake2_256;
use sp_runtime::traits::StaticLookup;
use sp_std::prelude::*;

/// Helper class to create accounts and its DID to simplify benchmarks and UT.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct User<T: Config> {
    pub account: T::AccountId,
    pub secret: Option<SecretKey>,
    pub origin: RawOrigin<T::AccountId>,
    pub did: Option<IdentityId>,
}

impl<T: Config> User<T> {
    pub fn did(&self) -> IdentityId {
        self.did.expect("User without DID")
    }

    pub fn account(&self) -> T::AccountId {
        self.account.clone()
    }

    pub fn origin(&self) -> RawOrigin<T::AccountId> {
        self.origin.clone()
    }

    pub fn lookup(&self) -> <T::Lookup as StaticLookup>::Source {
        T::Lookup::unlookup(self.account.clone())
    }

    pub fn sign(&self, message: &[u8]) -> Option<Signature> {
        self.secret
            .as_ref()
            .and_then(|sk| native_schnorrkel::sign(&sk.to_bytes(), message))
    }
}

pub struct UserBuilder<T: Config> {
    account: Option<T::AccountId>,
    did: Option<IdentityId>,
    balance: u32,
    seed: u32,
    generate_did: bool,
    as_did_registrar: bool,
}

macro_rules! self_update {
    ($self:ident, $member:ident, $value:expr) => {{
        let mut new = $self;
        new.$member = $value;
        new
    }};
}

impl<T: Config> UserBuilder<T> {
    /// Create an account based on the builder configuration.
    pub fn build(self, name: &str) -> User<T> {
        let (account, secret) = self
            .account
            .clone()
            .map_or_else(|| Self::make_key_pair(name, self.seed), |acc| (acc, None));
        let origin = RawOrigin::Signed(account.clone());
        let _ = T::Balances::make_free_balance_be(&account, self.balance.into());

        // Generate DID or use the one given.
        let did = self
            .did
            .or_else(|| self.generate_did.then(|| Self::make_did(account.clone())));

        // Become a DID registrar.
        self.as_did_registrar.then(|| {
            T::DidRegistrars::add_member(did.clone().unwrap())
                .expect("User cannot be added as DID registrar")
        });

        User {
            account,
            secret,
            origin,
            did,
        }
    }

    /// Create a DID for account `acc` using the specified investor ID.
    fn make_did(acc: T::AccountId) -> IdentityId {
        match T::IdentityFn::testing_register_did(acc.clone()) {
            Ok(did) => did,
            _ => T::IdentityFn::get_identity(&acc).unwrap(),
        }
    }
}

impl<T: Config> UserBuilder<T> {
    pub fn generate_did(self) -> Self {
        assert!(self.did.is_none());
        self_update!(self, generate_did, true)
    }

    pub fn become_did_registrar(self) -> Self {
        assert_eq!(self.generate_did, true || self.did.is_some());
        self_update!(self, as_did_registrar, true)
    }

    pub fn did(self, did: IdentityId) -> Self {
        assert_eq!(self.generate_did, false);
        let mut new = self;
        new.did = Some(did);
        new.generate_did = false;
        new
    }

    pub fn seed(self, s: u32) -> Self {
        self_update!(self, seed, s)
    }

    pub fn account<ACC: Into<T::AccountId>>(self, acc: ACC) -> Self {
        self_update!(self, account, Some(acc.into()))
    }

    pub fn balance<B: Into<u32>>(self, b: B) -> Self {
        self_update!(self, balance, b.into())
    }

    fn make_key_pair(name: &str, u: u32) -> (T::AccountId, Option<SecretKey>) {
        let seed = (name, u).using_encoded(blake2_256);

        let keypair = MiniSecretKey::from_bytes(&seed[..])
            .expect("Schnorrkell cannot create a secret key from that seed")
            .expand_to_keypair(ExpansionMode::Ed25519);

        let public = keypair.public.to_bytes();
        let id = T::AccountId::decode(&mut &public[..]).unwrap();

        (id, Some(keypair.secret.clone()))
    }
}

// Derive macro from `Default` is not supported due to trait T.
impl<T: Config> Default for UserBuilder<T> {
    fn default() -> Self {
        Self {
            account: None,
            did: None,
            balance: 5_000_000u32,
            seed: 0,
            generate_did: false,
            as_did_registrar: false,
        }
    }
}

pub fn user<T: Config>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default()
        .generate_did()
        .seed(u)
        .build(prefix)
}

pub fn user_without_did<T: Config>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default().seed(u).build(prefix)
}

pub fn did_registrar<T: Config>(prefix: &'static str, u: u32) -> User<T> {
    UserBuilder::<T>::default()
        .generate_did()
        .seed(u)
        .become_did_registrar()
        .build(prefix)
}
