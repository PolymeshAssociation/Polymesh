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
use pallet_identity::benchmarking::{make_account, make_account_without_did};

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

pub fn get_last_auth_id<T: Trait>(signatory: &Signatory<T::AccountId>) -> u64 {
    <identity::Authorizations<T>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
        .auth_id
}

benchmarks! {
    _ {}

    create_multisig {
        // Number of secondary keys
        let i in 1 .. 256;

        let (caller, origin, _) = make_account::<T>("caller", SEED);
        let mut signers = generate_signers::<T>(i as usize);
    }: _(origin, signers, i as u64)

    create_or_approve_proposal_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
        <MultiSig<T>>::accept_multisig_signer_as_identity(
            origin.clone().into(),
            alice_auth_id
        )?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_or_approve_proposal_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice));
        <MultiSig<T>>::accept_multisig_signer_as_key(
            origin.clone().into(),
            alice_auth_id
        )?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
        <MultiSig<T>>::accept_multisig_signer_as_identity(
            origin.clone().into(),
            alice_auth_id
        )?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice));
        <MultiSig<T>>::accept_multisig_signer_as_key(
            origin.clone().into(),
            alice_auth_id
        )?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    approve_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
        <MultiSig<T>>::accept_multisig_signer_as_identity(
            origin.clone().into(),
            alice_auth_id
        )?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig,
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    approve_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice));
        <MultiSig<T>>::accept_multisig_signer_as_key(
            origin.clone().into(),
            alice_auth_id
        )?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig,
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    reject_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
        <MultiSig<T>>::accept_multisig_signer_as_identity(
            origin.clone().into(),
            alice_auth_id
        )?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig,
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    reject_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice));
        <MultiSig<T>>::accept_multisig_signer_as_key(
            origin.clone().into(),
            alice_auth_id
        )?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig,
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(origin, multisig, 0)

    accept_multisig_signer_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
    }: _(origin, alice_auth_id)

    accept_multisig_signer_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice));
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
