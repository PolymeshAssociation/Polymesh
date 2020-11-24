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
use pallet_identity::benchmarking::make_account;

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;
pub type Signers<AccountId> = Vec<(AccountId, RawOrigin<AccountId>, IdentityId)>;

fn generate_signers<T: Trait>(signers: &mut Signers<T::AccountId>, n: usize) {
    for x in 0..n {
        signers.push(make_account::<T>("key", x as u32));
    }
}

fn generate_secondary_keys<T: Trait>(keys: &mut Vec<Signatory<T::AccountId>>, n: usize) {
    for x in 0..n {
        keys.push(Signatory::from(make_account::<T>("key", x as u32).2));
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
    signers: Signers<T::AccountId>,
    use_did: bool,
) -> Result<T::AccountId, DispatchError> {
    let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
    let keys = signers.iter().cloned().map(|(key, _, did)| {
        if use_did {
            Signatory::from(did)
        }else {
            Signatory::Account(key)
        }
    }).collect();
    <MultiSig<T>>::create_multisig(origin.clone(), keys, 1)?;
    for (key, origin, did) in signers {
        if use_did {
            let signer = Signatory::from(did);
            let auth_id = get_last_auth_id::<T>(&signer);
            Context::set_current_identity::<Identity<T>>(Some(did));
            <MultiSig<T>>::accept_multisig_signer_as_identity(origin.into(), auth_id)?;
        } else {
            let signer = Signatory::Account(key);
            let auth_id = get_last_auth_id::<T>(&signer);
            <MultiSig<T>>::accept_multisig_signer_as_key(origin.into(), auth_id)?;
        }
    }
    Ok(multisig)
}

benchmarks! {
    _ {}

    create_multisig {
        // Number of signers
        let i in 1 .. 256;

        let (caller, origin, _) = make_account::<T>("caller", 0);
        let mut keys = vec![Signatory::Account(caller)];
        generate_secondary_keys::<T>(&mut keys, i as usize);
    }: _(origin, keys, i as u64)

    create_or_approve_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![(alice.clone(), origin.clone(), alice_did)];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers, true)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_or_approve_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![(alice.clone(), origin.clone(), alice_did)];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers, false)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_identity {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![(alice.clone(), origin.clone(), alice_did)];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers, true)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    create_proposal_as_key {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let mut signers = vec![alice.clone(), origin.clone(), alice_did];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers, false)?;
    }: _(origin, multisig, Box::new(frame_system::Call::<T>::remark(vec![]).into()), None, true)

    approve_as_identity {
        let (alice, origin, alice_did) = make_account::<T>("alice", 0);
        let (bob, bob_origin, bob_did) = make_account::<T>("bob", 1);
        let signers = vec![
            (alice.clone(), origin.clone(), alice_did),
            (bob.clone(), bob_origin.clone(), bob_did)
        ];
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), signers, true)?;
        <MultiSig<T>>::create_proposal_as_identity(
            origin.clone().into(),
            multisig.clone(),
            Box::new(frame_system::Call::<T>::remark(vec![]).into()),
            None,
            true
        )?;
    }: _(bob_origin, multisig, 0)

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

    add_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. 256;

        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, multisig, generate_signers::<T>(i as usize))

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. 256;

        let signers = generate_signers::<T>(i as usize);
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let alice_sig = Signatory::from(alice_did);
        let multisig = generate_multisig_with_signers::<T>(alice.clone(), origin.clone().into(), alice_did, alice_sig, signers.clone())?;
    }: _(origin, multisig, signers)

    change_sigs_required {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, 1)

    change_all_signers_and_sigs_required {
        // Number of signers
        let i in 1 .. 256;

        let signers = generate_signers::<T>(i as usize);
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, signers, 1)

    make_multisig_signer {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let multisig_new = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let origin = RawOrigin::Signed(multisig);
    }: _(origin, multisig_new)

    make_multisig_primary {
        let (alice, origin, alice_did) = make_account::<T>("alice", SEED);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
        let multisig_new = <MultiSig<T>>::get_next_multisig_address(alice);
        <MultiSig<T>>::create_multisig(origin.clone().into(), vec![Signatory::from(alice_did)], 1)?;
    }: _(origin, multisig_new, None)
}
