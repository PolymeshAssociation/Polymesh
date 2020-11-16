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
use frame_system::RawOrigin;
use pallet_identity::benchmarking::make_account;

const SEED: u32 = 0;

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;

fn generate_signers<T: Trait>(n: usize) -> Vec<Signatory<T::AccountId>> {
    let mut secondary_keys = Vec::with_capacity(n);
    for x in 0..n {
        secondary_keys.push(Signatory::Account(account("key", x as u32, SEED)));
    }
    secondary_keys
}

fn get_last_auth_id<T: Trait>(signatory: &Signatory<T::AccountId>) -> u64 {
    <identity::Authorizations<T>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
        .auth_id
}

fn generate_multisig<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    alice_did: IdentityId,
    alice_sig: Signatory<T::AccountId>,
) -> Result<T::AccountId, DispatchError> {
    generate_multisig_with_signers::<T>(
        alice,
        origin,
        alice_did,
        alice_sig.clone(),
        vec![alice_sig.clone()],
    )
}

fn generate_multisig_rand_signers<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    alice_did: IdentityId,
    alice_sig: Signatory<T::AccountId>,
    signers: usize,
) -> Result<T::AccountId, DispatchError> {
    let mut signers = generate_signers::<T>(signers);
    *signers.first_mut().unwrap() = alice_sig.clone();
    generate_multisig_with_signers::<T>(alice, origin, alice_did, alice_sig, signers)
}

fn generate_multisig_with_signers<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    alice_did: IdentityId,
    alice_sig: Signatory<T::AccountId>,
    signers: Vec<Signatory<T::AccountId>>,
) -> Result<T::AccountId, DispatchError> {
    let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
    <MultiSig<T>>::create_multisig(origin.clone(), signers, 1)?;
    let alice_auth_id = get_last_auth_id::<T>(&alice_sig);
    Context::set_current_identity::<Identity<T>>(Some(alice_did));
    match alice_sig {
        Signatory::Identity(_) => {
            <MultiSig<T>>::accept_multisig_signer_as_identity(origin.clone(), alice_auth_id)?;
        }
        Signatory::Account(_) => {
            <MultiSig<T>>::accept_multisig_signer_as_key(origin.clone(), alice_auth_id)?;
        }
    }
    Ok(multisig)
}

benchmarks! {
    _ {}

    create_multisig {
        // Number of signers
        let i in 1 .. 256;

        let (caller, origin, _) = make_account::<T>("caller", SEED);
        let signers = generate_signers::<T>(i as usize);
    }: _(origin, signers, i as u64)

    create_or_approve_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::from(alice_did);
        let multisig = generate_multisig_rand_signers::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig, i as usize)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_or_approve_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::Account(alice.clone());
        let multisig = generate_multisig_rand_signers::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig, i as usize)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::from(alice_did);
        let multisig = generate_multisig_rand_signers::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig, i as usize)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::Account(alice.clone());
        let multisig = generate_multisig_rand_signers::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig, i as usize)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    approve_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::from(alice_did);
        let multisig = generate_multisig::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig)?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    approve_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::Account(alice.clone());
        let multisig = generate_multisig::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig)?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    reject_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::from(alice_did);
        let multisig = generate_multisig::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig)?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    reject_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::Account(alice.clone());
        let multisig = generate_multisig::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig)?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    accept_multisig_signer_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
    }: _(origin, alice_auth_id)

    accept_multisig_signer_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice.clone())], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice.clone()));
    }: _(origin, alice_auth_id)

    add_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let (bob, _, bob_did) = make_account::<T>("bob", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, Signatory::from(bob_did))

    remove_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, Signatory::from(alice_did))
}
