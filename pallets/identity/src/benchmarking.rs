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

const SEED: u32 = 0;
pub fn uid_from_name_and_idx(name: &'static str, u: u32) -> InvestorUid {
    InvestorUid::from((name, u).encode().as_slice())
}

pub fn make_account_without_did<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>) {
    let account: T::AccountId = account(name, u, SEED);
    let origin = RawOrigin::Signed(account.clone());
    let _ = balances::Module::<T>::make_free_balance_be(&account, 1_000_000.into());
    (account, origin)
}

pub fn make_account<T: Trait>(
    name: &'static str,
    u: u32,
) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let (account, origin) = make_account_without_did::<T>(name, u);
    let uid = uid_from_name_and_idx(name, u);
    let _ = Module::<T>::register_did(origin.clone().into(), uid, vec![]);
    let did = Module::<T>::get_identity(&account).unwrap();
    (account, origin, did)
}

pub fn make_cdd_account<T: Trait>(u: u32) -> (T::AccountId, RawOrigin<T::AccountId>, IdentityId) {
    let (cdd_account, cdd_origin, cdd_did) = make_account::<T>("cdd", u);
    T::CddServiceProviders::add_member(cdd_did).unwrap();
    (cdd_account, cdd_origin, cdd_did)
}

fn setup_investor_uniqueness_claim<T: Trait>(
    name: &'static str,
) -> (
    T::AccountId,
    RawOrigin<T::AccountId>,
    IdentityId,
    Claim,
    InvestorZKProofData,
) {
    let (account, origin) = make_account_without_did::<T>(name, SEED);

    let did = IdentityId::from([
        152u8, 25, 31, 70, 229, 131, 2, 22, 68, 84, 54, 151, 136, 3, 105, 122, 94, 58, 182, 27, 30,
        137, 81, 212, 254, 154, 230, 123, 171, 97, 74, 95,
    ]);
    Module::<T>::link_did(account.clone(), did);

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

    (account, origin, did, conf_scope_claim, inv_proof)
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

        make_cdd_account::<T>(SEED);
        let (_, origin) = make_account_without_did::<T>("caller", SEED);
        let uid = uid_from_name_and_idx("caller", SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(origin, uid, secondary_keys)

    cdd_register_did {
        // Number of secondary items.
        let i in 0 .. 50;

        let (_, origin, origin_did) = make_cdd_account::<T>(SEED);
        let target: T::AccountId = account("target", SEED, SEED);
        let secondary_keys = generate_secondary_keys::<T>(i as usize);
    }: _(origin, target, secondary_keys)

    mock_cdd_register_did {
        let (_, origin, origin_did) = make_cdd_account::<T>(SEED);

        let target: T::AccountId = account("target", SEED, SEED);
    }: _(origin, target)

    invalidate_cdd_claims {
        // NB: This function loops over all cdd claims issued by the cdd provider.
        // Therefore, it's unbounded in complexity. However, this can only be called by governance.
        // Hence, the weight is for best case scenario

        let (_, _, cdd_did) = make_cdd_account::<T>(SEED);

    }: _(RawOrigin::Root, cdd_did, 0.into(), None)

    remove_secondary_keys {
        // Number of secondary items.
        let i in 0 .. 50;

        let (target_account, target_origin, target_did) = make_account::<T>("target", SEED);

        let mut signatories = Vec::with_capacity(i as usize);
        for x in 0..i {
            let signer = Signatory::Account(account("key", x, SEED));
            signatories.push(signer.clone());
            Module::<T>::unsafe_join_identity(target_did, Permissions::default(), signer)?;
        }
    }: _(target_origin, signatories)

    accept_primary_key {
        let (_, cdd_origin, cdd_did) = make_cdd_account::<T>(SEED);
        let (_, target_origin, target_did) = make_account::<T>("target", SEED);
        let (new_key, new_key_origin) = make_account_without_did::<T>("key", SEED);

        let cdd_auth_id =  Module::<T>::add_auth(
            cdd_did,
            Signatory::Account(new_key.clone()),
            AuthorizationData::AttestPrimaryKeyRotation(target_did),
            None,
        );
        Module::<T>::change_cdd_requirement_for_mk_rotation(
            RawOrigin::Root.into(),
            true
        )?;

        let owner_auth_id =  Module::<T>::add_auth(
            target_did,
            Signatory::Account(new_key),
            AuthorizationData::RotatePrimaryKey(target_did),
            None,
        );
    }: _(new_key_origin, owner_auth_id, Some(cdd_auth_id))

    change_cdd_requirement_for_mk_rotation {}: _(RawOrigin::Root, true)

    join_identity_as_key {
        let (_, _, target_did) = make_account::<T>("target", SEED);
        let (new_key, new_key_origin) = make_account_without_did::<T>("key", SEED);

        let auth_id =  Module::<T>::add_auth(
            target_did,
            Signatory::Account(new_key),
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );
    }: _(new_key_origin, auth_id)

    join_identity_as_identity {
        let (_, _, target_did) = make_account::<T>("target", SEED);
        let (_, new_key_origin, new_identity) = make_account::<T>("key", SEED);

        let auth_id =  Module::<T>::add_auth(
            target_did,
            Signatory::Identity(new_identity),
            AuthorizationData::JoinIdentity(Permissions::default()),
            None,
        );
    }: _(new_key_origin, auth_id)

    leave_identity_as_key {
        let (_, _, target_did) = make_account::<T>("target", SEED);
        let (new_key, new_key_origin) = make_account_without_did::<T>("key", SEED);

        Module::<T>::unsafe_join_identity(target_did, Permissions::default(), Signatory::Account(new_key))?;

    }: _(new_key_origin)

    leave_identity_as_identity {
        let (_, _, target_did) = make_account::<T>("target", SEED);
        let (_, new_key_origin, new_did) = make_account::<T>("key", SEED);

        Module::<T>::unsafe_join_identity(target_did, Permissions::default(), Signatory::Identity(new_did))?;

    }: _(new_key_origin, target_did)

    add_claim {
        let (_, origin, origin_did) = make_account::<T>("caller", SEED);
        let (_, _, target_did) = make_account::<T>("target", SEED);
    }: _(origin, target_did, Claim::Jurisdiction(CountryCode::BB, Scope::Identity(origin_did)), Some(666.into()))

    forwarded_call {
        // NB: The automated weight calculation does not account for weight of the transaction being forwarded.
        // The weight of the forwarded call must be added to the weight calculated by this benchmark.
        let (_, _, target_did) = make_account::<T>("target", SEED);
        let (new_account, new_key_origin, new_did) = make_account::<T>("key", SEED);

        let call: T::Proposal = frame_system::Call::<T>::remark(vec![]).into();
        let boxed_proposal = Box::new(call);

        Module::<T>::unsafe_join_identity(target_did, Permissions::default(), Signatory::Identity(new_did))?;
        Module::<T>::set_context_did(Some(new_did));
    }: _(new_key_origin, target_did, boxed_proposal)

    revoke_claim {
        let (_, origin, did, conf_scope_claim, inv_proof) = setup_investor_uniqueness_claim::<T>("caller");
        Module::<T>::add_investor_uniqueness_claim(origin.clone().into(), did, conf_scope_claim.clone(), inv_proof, Some(666.into()))?;
    }: _(origin, did, conf_scope_claim)

    set_permission_to_signer {
        let (_, did_origin, target_did) = make_account::<T>("target", SEED);
        let (new_key, new_key_origin) = make_account_without_did::<T>("key", SEED);
        let signatory = Signatory::Account(new_key);

        Module::<T>::unsafe_join_identity(target_did, Permissions::empty(), signatory.clone())?;
    }: _(did_origin, signatory, Permissions::default().into())

    freeze_secondary_keys {
        let (_, origin, _) = make_account::<T>("caller", SEED);
    }: _(origin)

    unfreeze_secondary_keys {
        let (_, origin, _) = make_account::<T>("caller", SEED);
        Module::<T>::freeze_secondary_keys(origin.clone().into())?;
    }: _(origin)

    add_authorization {
        let (_, origin, did) = make_account::<T>("caller", SEED);
    }: _(origin, Signatory::Identity(did), AuthorizationData::JoinIdentity(Permissions::default()), Some(666.into()))

    remove_authorization {
        let (_, origin, did) = make_account::<T>("caller", SEED);
        let auth_id =  Module::<T>::add_auth(
            did,
            Signatory::Identity(did),
            AuthorizationData::JoinIdentity(Permissions::default()),
            Some(666.into()),
        );
    }: _(origin, Signatory::Identity(did), auth_id, true)

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
        let (_, origin, did) = make_account::<T>("caller", SEED);

        let authorization = TargetIdAuthorization::<T::Moment> {
            target_id: did.clone(),
            nonce: Module::<T>::offchain_authorization_nonce(did),
            expires_at: 600.into(),
        };
    }: _(origin, Signatory::Identity(did), authorization)

    add_investor_uniqueness_claim {
        let (_, origin, did, conf_scope_claim, inv_proof) = setup_investor_uniqueness_claim::<T>("caller");
    }: _(origin, did, conf_scope_claim, inv_proof, Some(666.into()))
}
