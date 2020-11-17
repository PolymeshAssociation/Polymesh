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

#![cfg(feature = "runtime-benchmarks")]
use crate::*;
use frame_benchmarking::{account, benchmarks};
use frame_support::traits::Currency;
use frame_system::RawOrigin;
use pallet_balances as balances;
use polymesh_common_utilities::traits::identity::TargetIdAuthorization;
use polymesh_primitives::{
    AuthorizationData, Claim, CountryCode, IdentityId, InvestorUid, Permissions, Scope, Signatory,
};

use schnorrkel::Signature;
use sp_std::prelude::*;

#[cfg(feature = "std")]
use schnorrkel::Keypair;
#[cfg(feature = "std")]
use sp_core::{crypto::Pair as TPair, sr25519::Pair};
#[cfg(feature = "std")]
const SIGNING_CTX: &[u8] = b"substrate";

const SEED: u32 = 0;
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
    uid: Option<InvestorUid>,
    pub did: Option<IdentityId>,
}

impl<T: Trait> User<T> {
    /// Create an account based on `name` and `u` with 1_000_000 as free balance.
    /// It also registers the DID for that account.
    pub fn new(name: &'static str, u: u32) -> Self {
        let mut user = Self::without_did(name, u);
        let uid = uid_from_name_and_idx(name, u);
        let _ = Module::<T>::register_did(user.origin.clone().into(), uid, vec![]);
        user.uid = Some(uid);
        user.did = Module::<T>::get_identity(&user.account.clone());

        user
    }

    /// Create a new CDD account.
    pub fn new_cdd(u: u32) -> Self {
        let user = Self::new("cdd", u);
        T::CddServiceProviders::add_member(user.did()).unwrap();
        user
    }

    /// Create an account based on `name` and `u` with 1_000_000 as free balance.
    pub fn without_did(name: &'static str, u: u32) -> Self {
        let (account, secret) = Self::make_key_pair(name, u);
        let origin = RawOrigin::Signed(account.clone());
        let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());

        Self {
            account,
            secret,
            origin,
            uid: None,
            did: None,
        }
    }

    /// Create an account with specific public key.
    /// It is used to make reproducible test-cases on WASM.
    pub fn new_from_public(pk: PublicKey) -> Self {
        let account = T::AccountId::decode(&mut &pk[..]).unwrap();
        let secret = [0u8; 64];
        let origin = RawOrigin::Signed(account.clone());
        let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
        let uid = InvestorUid::from(&pk[..16]);
        let _ = Module::<T>::register_did(origin.clone().into(), uid, vec![]);
        let did = Module::<T>::get_identity(&account);

        Self {
            account,
            secret,
            origin,
            uid: Some(uid),
            did,
        }
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
        let keypair: &Keypair = pair.as_ref();

        let secret = keypair.secret.to_bytes();
        let public = keypair.public.to_bytes();
        let id = T::AccountId::decode(&mut &public[..]).unwrap();

        (id, secret)
    }

    pub fn did(self: &Self) -> IdentityId {
        self.did.clone().expect("User without DID")
    }

    pub fn uid(self: &Self) -> InvestorUid {
        self.uid.clone().expect("User without UID")
    }

    #[cfg(feature = "std")]
    pub fn sign(&self, message: &[u8]) -> Signature {
        let sk = schnorrkel::keys::SecretKey::from_bytes(&self.secret[..])
            .expect("Invalid sr25519 secret key");
        let pair = Keypair::from(sk);
        let context = schnorrkel::signing_context(SIGNING_CTX);
        pair.sign(context.bytes(message)).into()
    }

    #[cfg(not(feature = "std"))]
    pub fn sign(&self, _message: &[u8]) -> Signature {
        panic!("Cannot sign without 'std' support");
    }
}

fn setup_investor_uniqueness_claim<T: Trait>(
    name: &'static str,
) -> (User<T>, Claim, InvestorZKProofData) {
    let mut user = User::<T>::without_did(name, SEED);

    let did = IdentityId::from([
        152u8, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122, 94, 58, 182, 27, 30,
        137, 81, 212, 254, 154, 230, 123, 171, 97, 74, 95,
    ]);
    Module::<T>::link_did(user.account.clone(), did);
    user.did = Some(did.clone());

    let cdd_id = CddId::from([
        102u8, 210, 32, 212, 213, 80, 255, 99, 142, 30, 202, 20, 220, 131, 109, 106, 137, 12, 137,
        191, 123, 156, 212, 20, 215, 87, 23, 42, 84, 181, 128, 73,
    ]);
    let cdd_claim = Claim::CustomerDueDiligence(cdd_id);
    Module::<T>::base_add_claim(did, cdd_claim, did, Some(666.into()));

    let scope = Scope::Custom([228u8, 152, 116, 104, 5, 8, 30, 188, 143, 185, 10, 208].to_vec());
    let scope_did = IdentityId::from([
        2u8, 72, 20, 154, 7, 96, 116, 105, 155, 74, 227, 252, 172, 18, 200, 203, 137, 107, 200,
        210, 194, 71, 250, 41, 108, 172, 100, 107, 223, 114, 182, 101,
    ]);
    let conf_scope_claim = Claim::InvestorUniqueness(scope, scope_did, cdd_id);

    let inv_proof = InvestorZKProofData(
        Signature::from_bytes(&[
            216u8, 224, 57, 254, 200, 45, 150, 202, 12, 108, 226, 233, 148, 213, 237, 7, 35, 150,
            142, 18, 127, 146, 162, 19, 161, 164, 95, 67, 181, 100, 156, 25, 201, 210, 209, 165,
            182, 74, 184, 145, 230, 255, 215, 144, 223, 100, 100, 147, 226, 58, 142, 92, 103, 153,
            153, 204, 123, 120, 133, 113, 218, 51, 208, 132,
        ])
        .unwrap(),
    );

    (user, conf_scope_claim, inv_proof)
}

fn generate_secondary_keys<T: Trait>(
    n: usize,
) -> Vec<secondary_key::api::SecondaryKey<T::AccountId>> {
    let mut secondary_keys = Vec::with_capacity(n);
    for x in 0..n {
        secondary_keys.push(secondary_key::api::SecondaryKey {
            signer: Signatory::Account(account("key", x as u32, SEED)),
            ..Default::default()
        });
    }
    secondary_keys
}

benchmarks! {
    _ {}

    register_did {
        // Number of secondary items.
        let i in 0 .. 50;

        let _cdd = User::<T>::new_cdd(SEED);
        let caller = User::<T>::new("caller", SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(caller.origin, caller.uid(), secondary_keys)

    cdd_register_did {
        // Number of secondary items.
        let i in 0 .. 50;

        let cdd = User::<T>::new_cdd(SEED);
        let target: T::AccountId = account("target", SEED, SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(cdd.origin, target, secondary_keys)

    mock_cdd_register_did {
        let cdd = User::<T>::new_cdd(SEED);
        let target: T::AccountId = account("target", SEED, SEED);
    }: _(cdd.origin, target)

    invalidate_cdd_claims {
        // NB: This function loops over all cdd claims issued by the cdd provider.
        // Therefore, it's unbounded in complexity. However, this can only be called by governance.
        // Hence, the weight is for best case scenario

        let cdd = User::<T>::new_cdd(SEED);

    }: _(RawOrigin::Root, cdd.did(), 0.into(), None)

    remove_secondary_keys {
        // Number of secondary items.
        let i in 0 .. 50;

        let target = User::<T>::new("target", SEED);

        let mut signatories = Vec::with_capacity(i as usize);
        for x in 0..i {
            let signer = Signatory::Account(account("key", x, SEED));
            signatories.push(signer.clone());
            Module::<T>::unsafe_join_identity(target.did(), Permissions::default(), signer)?;
        }
    }: _(target.origin, signatories.clone())

    accept_primary_key {
        let cdd = User::<T>::new_cdd(SEED);
        let target = User::<T>::new("target", SEED);
        let new_key = User::<T>::without_did("key", SEED);
        let signatory = Signatory::Account(new_key.account.clone());

        let cdd_auth_id =  Module::<T>::add_auth(
            cdd.did(), signatory.clone(),
            AuthorizationData::AttestPrimaryKeyRotation(target.did()),
            None,
        );
        Module::<T>::change_cdd_requirement_for_mk_rotation(
            RawOrigin::Root.into(),
            true
        )?;

        let owner_auth_id =  Module::<T>::add_auth(
            target.did(), signatory,
            AuthorizationData::RotatePrimaryKey(target.did()),
            None,
        );
    }: _(new_key.origin, owner_auth_id, Some(cdd_auth_id))

    change_cdd_requirement_for_mk_rotation {}: _(RawOrigin::Root, true)

    join_identity_as_key {
        let target = User::<T>::new("target", SEED);
        let new_key = User::<T>::without_did("key", SEED);

        let auth_id =  Module::<T>::add_auth(
            target.did(),
            Signatory::Account(new_key.account.clone()),
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );
    }: _(new_key.origin, auth_id)

    join_identity_as_identity {
        let target = User::<T>::new("target", SEED);
        let key = User::<T>::new("key", SEED);

        let auth_id =  Module::<T>::add_auth(
            target.did(),
            Signatory::Identity(key.did()),
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );
    }: _(key.origin, auth_id)

    leave_identity_as_key {
        let target = User::<T>::new("target", SEED);
        let key = User::<T>::without_did("key", SEED);
        let signatory = Signatory::Account(key.account.clone());

        Module::<T>::unsafe_join_identity(target.did(), Permissions::default(), signatory)?;

    }: _(key.origin)

    leave_identity_as_identity {
        let target = User::<T>::new("target", SEED);
        let key = User::<T>::new("key", SEED);
        let signatory = Signatory::Identity(key.did());

        Module::<T>::unsafe_join_identity(target.did(), Permissions::default(), signatory)?;

    }: _(key.origin, target.did())

    add_claim {
        let caller = User::<T>::new("caller", SEED);
        let target = User::<T>::new("target", SEED);
        let scope = Scope::Identity(caller.did());
        let claim = Claim::Jurisdiction(CountryCode::BB, scope);
    }: _(caller.origin, target.did(), claim, Some(666.into()))

    forwarded_call {
        // NB: The automated weight calculation does not account for weight of the transaction being forwarded.
        // The weight of the forwarded call must be added to the weight calculated by this benchmark.
        let target = User::<T>::new("target", SEED);
        let key = User::<T>::new("key", SEED);

        let call: T::Proposal = frame_system::Call::<T>::remark(vec![]).into();
        let boxed_proposal = Box::new(call);

        Module::<T>::unsafe_join_identity(target.did(), Permissions::default(), Signatory::Identity(key.did()))?;
        Module::<T>::set_context_did(Some(key.did()));
    }: _(key.origin, target.did(), boxed_proposal)

    revoke_claim {
        let (caller, conf_scope_claim, inv_proof) = setup_investor_uniqueness_claim::<T>("caller");
        Module::<T>::add_investor_uniqueness_claim(caller.origin.clone().into(), caller.did(), conf_scope_claim.clone(), inv_proof, Some(666.into()))?;
    }: _(caller.origin, caller.did(), conf_scope_claim)

    set_permission_to_signer {
        let target = User::<T>::new("target", SEED);
        let key = User::<T>::without_did("key", SEED);
        let signatory = Signatory::Account(key.account);

        Module::<T>::unsafe_join_identity(target.did(), Permissions::empty(), signatory.clone())?;
    }: _(target.origin, signatory, Permissions::default().into())

    freeze_secondary_keys {
        let caller = User::<T>::new("caller", SEED);
    }: _(caller.origin)

    unfreeze_secondary_keys {
        let caller = User::<T>::new("caller", SEED);
        Module::<T>::freeze_secondary_keys(caller.origin.clone().into())?;
    }: _(caller.origin)

    add_authorization {
        let caller = User::<T>::new("caller", SEED);
        let signatory = Signatory::Identity(caller.did());
        let auth_data = AuthorizationData::JoinIdentity(Permissions::default());
    }: _(caller.origin, signatory, auth_data, Some(666.into()))

    remove_authorization {
        let caller = User::<T>::new("caller", SEED);
        let signatory = Signatory::Identity(caller.did());
        let auth_id =  Module::<T>::add_auth(
            caller.did(),
            signatory.clone(),
            AuthorizationData::JoinIdentity(Permissions::default()),
            Some(666.into()),
        );
    }: _(caller.origin, signatory, auth_id)

    // TODO: accept_authorization. The worst case of `accept_authorization` will be whatever authorization type takes most resources.
    // A defensive weight has been hardcoded for now but it should be updated once we've done benchmarks for all auth types.

    // TODO: fix this.
    // Account keyring is not available in no_std so it's not possible to sign data directly.
    // However, substrate injects the required functions as host functions in WASM.
    // We need to setup some helper functions to access those.
    // A defensive weight has been hardcoded for now.
    // add_secondary_keys_with_authorization {
    //     // Number of keys.
    //     let n in 0 .. 8;

    //     let (_, origin, did) = make_account::<T>("caller", SEED);

    //     let expires_at = 600u64;
    //     let authorization = TargetIdAuthorization::<u64> {
    //         target_id: did.clone(),
    //         nonce: Module::<T>::offchain_authorization_nonce(did),
    //         expires_at,
    //     };
    //     let auth_encoded = authorization.encode();

    //     let accounts = [
    //         AccountKeyring::Alice,
    //         AccountKeyring::Bob,
    //         AccountKeyring::Charlie,
    //         AccountKeyring::Dave,
    //         AccountKeyring::Eve,
    //         AccountKeyring::Ferdie,
    //         AccountKeyring::One,
    //         AccountKeyring::Two
    //     ];

    //     let secondary_keys_with_auth = accounts.into_iter().enumerate().take(n as usize).map(|(i, acc)| {
    //         let (_, _, key_did) = make_account::<T>("key", i as u32);
    //         let sig = H512::from(acc.sign(&auth_encoded));
    //         SecondaryKeyWithAuth {
    //             secondary_key: SecondaryKey::from(key_did).into(),
    //             auth_signature: sig,
    //         }
    //     }).collect::<Vec<_>>();

    // }: _(origin, secondary_keys_with_auth, expires_at)

    revoke_offchain_authorization {
        let caller = User::<T>::new("caller", SEED);
        let nonce = Module::<T>::offchain_authorization_nonce(caller.did());

        let authorization = TargetIdAuthorization::<T::Moment> {
            target_id: caller.did(),
            nonce,
            expires_at: 600.into(),
        };
    }: _(caller.origin, Signatory::Identity(caller.did()), authorization)

    add_investor_uniqueness_claim {
        let (caller, conf_scope_claim, inv_proof) = setup_investor_uniqueness_claim::<T>("caller");
    }: _(caller.origin, caller.did(), conf_scope_claim, inv_proof, Some(666.into()))
}
