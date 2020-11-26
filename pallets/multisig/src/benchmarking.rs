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
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use pallet_identity::benchmarking::{make_account, make_account_without_did};

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;

fn generate_signers<T: Trait>(keys: &mut Vec<Signatory<T::AccountId>>, n: usize) {
    for x in 0..n {
        keys.push(Signatory::Account(
            make_account_without_did::<T>("key", x as u32).0,
        ));
    }
}

fn get_last_auth_id<T: Trait>(signatory: &Signatory<T::AccountId>) -> u64 {
    <identity::Authorizations<T>>::iter_prefix_values(signatory)
        .into_iter()
        .max_by_key(|x| x.auth_id)
        .expect("there are no authorizations")
        .auth_id
}

fn generate_multisig_with_signers<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    signers: Vec<Signatory<T::AccountId>>,
) -> Result<T::AccountId, DispatchError> {
    let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
    <MultiSig<T>>::create_multisig(origin.clone(), signers.clone(), 1)?;
    for signer in signers {
        let auth_id = get_last_auth_id::<T>(&signer);
        if let Signatory::Account(key) = signer {
            <MultiSig<T>>::accept_multisig_signer_as_key(RawOrigin::Signed(key).into(), auth_id)?;
        }
    }
    Ok(multisig)
}

benchmarks! {
    _ {}

    create_multisig {
        // Number of signers
        let i in 1 .. 256;

        let (caller, origin, caller_did) = make_account::<T>("caller", 0);
        let mut signers = vec![Signatory::from(caller_did)];
        generate_signers::<T>(&mut signers, i as usize);
    }: _(origin, signers, i as u64)

    create_or_approve_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![Signatory::from(alice_did)];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_or_approve_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin) = make_account_without_did::<T>("alice", 0);
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![Signatory::from(alice_did)];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin) = make_account_without_did::<T>("alice", 0);
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    approve_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let (bob, bob_origin, bob_did) = make_account::<T>("bob", 1);
        let signers = vec![
            Signatory::from(alice_did),
            Signatory::from(bob_did),
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob_origin, multisig, 0)

    approve_as_key {
        let (alice, origin) = make_account_without_did::<T>("alice", 0);
        let (bob, bob_origin) = make_account_without_did::<T>("bob", 1);
        let signers = vec![
            Signatory::Account(bob),
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob_origin, multisig, 0)

    reject_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let (bob, bob_origin, bob_did) = make_account::<T>("bob", 1);
        let signers = vec![
            Signatory::from(alice_did),
            Signatory::from(bob_did),
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob_origin, multisig, 0)

    reject_as_key {
        let (alice, origin) = make_account_without_did::<T>("alice", 0);
        let (bob, bob_origin) = make_account_without_did::<T>("bob", 1);
        let signers = vec![
            Signatory::Account(bob),
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers)?;
        <MultiSig<T>>::create_proposal_as_key(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob_origin, multisig, 0)

    accept_multisig_signer_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice_did));
        Context::set_current_identity::<Identity<T>>(Some(alice_did));
    }: _(origin, alice_auth_id)

    accept_multisig_signer_as_key {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::Account(alice.clone())], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::Account(alice.clone()));
    }: _(origin, alice_auth_id)

    add_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let (bob, _, bob_did) = make_account::<T>("bob", 1);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, Signatory::from(bob_did))

    remove_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, Signatory::from(alice_did))

    add_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
    }: _(origin, multisig, signers)

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. 256;

        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers.clone())?;
    }: _(origin, multisig, signers)

    change_sigs_required {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, 1)

    change_all_signers_and_sigs_required {
        // Number of signers
        let i in 1 .. 256;

        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, signers, 1)

    make_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let multisig_new = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, multisig_new)

    make_multisig_primary {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let multisig_new = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, multisig_new, None)
}
