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
use pallet_identity::benchmarking::UserBuilder;

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;

fn generate_signers<T: Trait>(keys: &mut Vec<Signatory<T::AccountId>>, n: usize) {
    for x in 0..n {
        keys.push(Signatory::Account(
            <UserBuilder<T>>::default().build("key", x as u32).account,
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

fn generate_multisig<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    signers: Vec<Signatory<T::AccountId>>,
) -> Result<T::AccountId, DispatchError> {
    let num_of_signers = signers.len() as u64;
    generate_multisig_with_signers::<T>(alice, origin, signers, num_of_signers)
}

fn generate_multisig_with_signers<T: Trait>(
    alice: T::AccountId,
    origin: T::Origin,
    signers: Vec<Signatory<T::AccountId>>,
    num_of_signers: u64,
) -> Result<T::AccountId, DispatchError> {
    let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
    <MultiSig<T>>::create_multisig(origin.clone(), signers.clone(), num_of_signers)?;
    for signer in signers {
        let auth_id = get_last_auth_id::<T>(&signer);
        <MultiSig<T>>::unsafe_accept_multisig_signer(signer, auth_id)?;
    }
    Ok(multisig)
}

const MAX_SIGNERS: u32 = 256;

benchmarks! {
    _ {}

    create_multisig {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let caller = <UserBuilder<T>>::default().build_with_did("caller", 0);
        let mut signers = vec![Signatory::from(caller.did())];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = <MultiSig<T>>::get_next_multisig_address(caller.account());
    }: _(caller.origin, signers, i as u64)
    verify {
        ensure!(<MultiSigToIdentity<T>>::contains_key(multisig), "create_multisig");
    }

    create_or_approve_proposal_as_identity {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let mut signers = vec![Signatory::from(alice.did())];
        generate_signers::<T>(&mut signers, 1);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
    }: _(alice.origin(), multisig.clone(), Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)
    verify {
        ensure!(proposal_id < <MultiSig<T>>::ms_tx_done(multisig.clone()), "create_or_approve_proposal_as_identity");
    }

    create_or_approve_proposal_as_key {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let mut signers = vec![Signatory::Account(bob.account())];
        generate_signers::<T>(&mut signers, 1);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
    }: _(bob.origin, multisig.clone(), Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)
    verify {
        ensure!(proposal_id < <MultiSig<T>>::ms_tx_done(multisig.clone()), "create_or_approve_proposal_as_key");
    }

    create_proposal_as_identity {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let mut signers = vec![Signatory::from(alice.did())];
        generate_signers::<T>(&mut signers, 1);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
    }: _(alice.origin, multisig.clone(), Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)
    verify {
        ensure!(proposal_id < <MultiSig<T>>::ms_tx_done(multisig.clone()), "create_proposal_as_identity");
    }

    create_proposal_as_key {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let mut signers = vec![Signatory::Account(bob.account())];
        generate_signers::<T>(&mut signers, 1);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
    }: _(bob.origin(), multisig.clone(), Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)
    verify {
        ensure!(proposal_id < <MultiSig<T>>::ms_tx_done(multisig.clone()), "create_proposal_as_key");
    }

    approve_as_identity {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build_with_did("bob", 0);
        let signers = vec![
            Signatory::from(alice.did()),
            Signatory::from(bob.did()),
        ];
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
        <MultiSig<T>>::create_proposal_as_identity(
            alice.origin(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob.origin(), multisig.clone(), 0)
    verify {
        ensure!(
            <MultiSig<T>>::votes((multisig.clone(), Signatory::from(bob.did()), proposal_id)),
            "approve_as_identity"
        );
    }

    approve_as_key {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let charlie = <UserBuilder<T>>::default().build("charlie", 0);
        let signers = vec![
            Signatory::Account(bob.account()),
            Signatory::Account(charlie.account()),
        ];
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
        <MultiSig<T>>::create_proposal_as_key(
            bob.origin(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(charlie.origin(), multisig.clone(), 0)
    verify {
        ensure!(
            <MultiSig<T>>::votes((multisig.clone(), Signatory::Account(charlie.account()), proposal_id)),
            "approve_as_key"
        );
    }

    reject_as_identity {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build_with_did("bob", 0);
        let signers = vec![
            Signatory::from(alice.did()),
            Signatory::from(bob.did()),
        ];
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
        <MultiSig<T>>::create_proposal_as_identity(
            alice.origin(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob.origin(), multisig.clone(), 0)
    verify {
        ensure!(
            <MultiSig<T>>::votes((multisig.clone(), Signatory::from(bob.did()), proposal_id)),
            "reject_as_identity"
        );
    }


    reject_as_key {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let charlie = <UserBuilder<T>>::default().build("charlie", 0);
        let signers = vec![
            Signatory::Account(bob.account()),
            Signatory::Account(charlie.account()),
        ];
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
        <MultiSig<T>>::create_proposal_as_key(
            bob.origin(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(charlie.origin(), multisig.clone(), 0)
    verify {
        ensure!(
            <MultiSig<T>>::votes((multisig.clone(), Signatory::Account(charlie.account()), proposal_id)),
            "reject_as_key"
        );
    }

    accept_multisig_signer_as_identity {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::from(alice.did())], 1)?;
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice.did()));
        Context::set_current_identity::<Identity<T>>(Some(alice.did()));
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(alice.origin(), alice_auth_id)
    verify {
        ensure!(
            num_of_signers < <NumberOfSigners<T>>::get(multisig.clone()),
            "accept_multisig_signer_as_identity"
        );
    }

    accept_multisig_signer_as_key {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::Account(bob.account())], 1)?;
        let auth_id = get_last_auth_id::<T>(&Signatory::Account(bob.account()));
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(bob.origin(), auth_id)
    verify {
        ensure!(
            num_of_signers < <NumberOfSigners<T>>::get(multisig.clone()),
            "accept_multisig_signer_as_key"
        );
    }

    add_multisig_signer {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build("bob", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::from(alice.did())], 1)?;
        let origin = RawOrigin::Signed(multisig.clone());
        let auth_id = get_last_auth_id::<T>(&Signatory::Account(bob.account()));
    }: _(origin, Signatory::Account(bob.account()))
    verify {
        ensure!(
            auth_id < get_last_auth_id::<T>(&Signatory::Account(bob.account())),
            "add_multisig_signer"
        );
    }

    remove_multisig_signer {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build_with_did("bob", 0);
        let signers = vec![
            Signatory::from(alice.did()),
            Signatory::from(bob.did()),
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.account(), alice.origin(), signers, 1)?;
        let origin = RawOrigin::Signed(multisig.clone());
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(origin, Signatory::from(bob.did()))
    verify {
        ensure!(
            num_of_signers > <NumberOfSigners<T>>::get(multisig.clone()),
            "remove_multisig_signer"
        );
    }


    add_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::from(alice.did())], 1)?;
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(alice.origin(), multisig.clone(), signers)
    verify {
        ensure!(
            num_of_signers < <NumberOfSigners<T>>::get(multisig.clone()),
            "add_multisig_signers_via_creator"
        );
    }

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let bob = <UserBuilder<T>>::default().build_with_did("bob", 0);
        let mut signers = vec![
            Signatory::from(alice.did()),
        ];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.account(), alice.origin(), signers.clone(), 1)?;
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(alice.origin(), multisig.clone(), signers[1..].to_vec())
    verify {
        ensure!(
            num_of_signers > <NumberOfSigners<T>>::get(multisig.clone()),
            "remove_multisig_signers_via_creator"
        );
    }


    change_sigs_required {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let mut signers = vec![Signatory::from(alice.did())];
        generate_signers::<T>(&mut signers, 1);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let origin = RawOrigin::Signed(multisig.clone());
        let sigs_required = <MultiSigSignsRequired<T>>::get(&multisig);
    }: _(origin, 1)
    verify {
        ensure!(
            sigs_required != <MultiSigSignsRequired<T>>::get(&multisig),
            "change_sigs_required"
        );
    }

    change_all_signers_and_sigs_required {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, MAX_SIGNERS as usize);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), vec![Signatory::from(alice.did())])?;
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let origin = RawOrigin::Signed(multisig.clone());
        let sigs_required = <MultiSigSignsRequired<T>>::get(&multisig);
    }: _(origin, signers, 1)
    verify {
        ensure!(
            sigs_required != <MultiSigSignsRequired<T>>::get(&multisig),
            "change_sigs_required"
        );
    }

    make_multisig_signer {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::from(alice.did())], 1)?;
    }: _(alice.origin(), multisig)

    make_multisig_primary {
        let alice = <UserBuilder<T>>::default().build_with_did("alice", 0);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account().clone());
        <MultiSig<T>>::create_multisig(alice.origin(), vec![Signatory::from(alice.did())], 1)?;
    }: _(alice.origin(), multisig, None)
}
