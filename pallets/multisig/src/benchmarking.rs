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
use polymesh_common_utilities::benchs::{User, UserBuilder};

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Module<T>;

fn generate_signers<T: Trait>(signers: &mut Vec<Signatory<T::AccountId>>, n: usize) {
    signers.extend((0..n).map(|x| {
        Signatory::Account(
            <UserBuilder<T>>::default()
                .seed(x as u32)
                .build("key")
                .account,
        )
    }));
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
    origin: RawOrigin<T::AccountId>,
    signers: Vec<Signatory<T::AccountId>>,
) -> Result<T::AccountId, DispatchError> {
    let num_of_signers = signers.len() as u64;
    generate_multisig_with_signers::<T>(alice, origin, signers, num_of_signers)
}

fn generate_multisig_with_signers<T: Trait>(
    alice: T::AccountId,
    origin: RawOrigin<T::AccountId>,
    signers: Vec<Signatory<T::AccountId>>,
    num_of_signers: u64,
) -> Result<T::AccountId, DispatchError> {
    let multisig = <MultiSig<T>>::get_next_multisig_address(alice.clone());
    <MultiSig<T>>::create_multisig(origin.into(), signers.clone(), num_of_signers)?;
    for signer in signers {
        let auth_id = get_last_auth_id::<T>(&signer);
        <MultiSig<T>>::unsafe_accept_multisig_signer(signer, auth_id)?;
    }
    Ok(multisig)
}

fn generate_multisig_with_extra_signers<T: Trait>(
    caller: &User<T>,
    mut signers: &mut Vec<Signatory<T::AccountId>>,
    num_of_extra_signers: u32,
    num_of_signers_required: u32,
) -> Result<T::AccountId, DispatchError> {
    generate_signers::<T>(&mut signers, num_of_extra_signers as usize);
    let multisig = <MultiSig<T>>::get_next_multisig_address(caller.account());
    <MultiSig<T>>::create_multisig(
        caller.origin.clone().into(),
        signers.clone(),
        num_of_signers_required.into(),
    )?;
    for signer in signers {
        let auth_id = get_last_auth_id::<T>(signer);
        <MultiSig<T>>::unsafe_accept_multisig_signer(signer.clone(), auth_id)?;
    }
    Ok(multisig)
}

fn generate_multisig_for_alice<T: Trait>(
    total_signers: u32,
    singers_required: u32,
) -> Result<
    (
        User<T>,
        T::AccountId,
        Vec<Signatory<T::AccountId>>,
        RawOrigin<T::AccountId>,
    ),
    DispatchError,
> {
    let alice = <UserBuilder<T>>::default().generate_did().build("alice");
    let mut signers = vec![Signatory::from(alice.did())];
    let multisig = generate_multisig_with_extra_signers::<T>(
        &alice,
        &mut signers,
        total_signers - 1,
        singers_required,
    )?;
    let signer_origin = match signers.last().clone().unwrap() {
        Signatory::Account(account) => RawOrigin::Signed(account.clone()),
        _ => alice.origin().clone(),
    };
    Ok((alice, multisig, signers, signer_origin))
}

pub type SetupResult<T, AccountId, Proposal> = (
    User<T>,
    AccountId,
    Vec<Signatory<AccountId>>,
    RawOrigin<AccountId>,
    u64,
    Box<Proposal>,
    AccountId,
);

fn generate_multisig_and_proposal_for_alice<T: Trait>(
    total_signers: u32,
    singers_required: u32,
) -> Result<SetupResult<T, T::AccountId, T::Proposal>, DispatchError> {
    let (alice, multisig, signers, signer_origin) =
        generate_multisig_for_alice::<T>(total_signers, singers_required)?;
    let proposal_id = <MultiSig<T>>::ms_tx_done(multisig.clone());
    let proposal = Box::new(frame_system::Call::<T>::remark(vec![]).into());
    Ok((
        alice,
        multisig.clone(),
        signers,
        signer_origin,
        proposal_id,
        proposal,
        multisig,
    ))
}

fn generate_multisig_and_create_proposal<T: Trait>(
    total_signers: u32,
    singers_required: u32,
    create_as_key: bool,
) -> Result<SetupResult<T, T::AccountId, T::Proposal>, DispatchError> {
    let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) =
        generate_multisig_and_proposal_for_alice::<T>(total_signers, singers_required)?;
    if create_as_key {
        <MultiSig<T>>::create_proposal_as_key(
            signer_origin.clone().into(),
            multisig.clone(),
            proposal.clone(),
            None,
            true,
        )?;
    } else {
        <MultiSig<T>>::create_proposal_as_identity(
            alice.origin().into(),
            multisig.clone(),
            proposal.clone(),
            None,
            true,
        )?;
    }
    Ok((
        alice,
        multisig,
        signers,
        signer_origin,
        proposal_id,
        proposal,
        ephemeral_multisig,
    ))
}

macro_rules! ensure_proposal_created {
    ($proposal_id:ident, $multisig:ident) => {
        assert!($proposal_id < <MultiSig<T>>::ms_tx_done($multisig));
    };
}

macro_rules! ensure_vote_cast {
    ($proposal_id:ident, $multisig:ident, $signatory:expr) => {
        assert!(<MultiSig<T>>::votes(($multisig, $signatory, $proposal_id)));
    };
}

const MAX_SIGNERS: u32 = 256;

benchmarks! {
    _ {}

    create_multisig {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;
        let (alice, multisig, signers, _) = generate_multisig_for_alice::<T>(i, 1)?;
    }: _(alice.origin(), signers, i as u64)
    verify {
        ensure!(<MultiSigToIdentity<T>>::contains_key(multisig), "create_multisig");
    }

    create_or_approve_proposal_as_identity {
        let (alice, multisig, signers, _, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(1, 1)?;
    }: _(alice.origin(), ephemeral_multisig, proposal, Some(1337.into()), true)
    verify {
        ensure_proposal_created!(proposal_id, multisig);
    }

    create_or_approve_proposal_as_key {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(2, 1)?;
    }: _(signer_origin, ephemeral_multisig, proposal, Some(1337.into()), true)
    verify {
        ensure_proposal_created!(proposal_id, multisig);
    }

    create_proposal_as_identity {
        let (alice, multisig, _, _, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(1, 1)?;
    }: _(alice.origin(), ephemeral_multisig, proposal, Some(1337.into()), true)
    verify {
        ensure_proposal_created!(proposal_id, multisig);
    }

    create_proposal_as_key {
        let (_, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(2, 1)?;
    }: _(signer_origin, ephemeral_multisig, proposal, Some(1337.into()), true)
    verify {
        ensure_proposal_created!(proposal_id, multisig);
    }

    approve_as_identity {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, true)?;
    }: _(alice.origin(), ephemeral_multisig, proposal_id)
    verify {
        ensure_vote_cast!(proposal_id, multisig, Signatory::from(alice.did()));
    }

    approve_as_key {
        let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, false)?;
    }: _(signer_origin, ephemeral_multisig, proposal_id)
    verify {
        ensure_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    reject_as_identity {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, true)?;
    }: _(alice.origin(), ephemeral_multisig, proposal_id)
    verify {
        ensure_vote_cast!(proposal_id, multisig, Signatory::from(alice.did()));
    }

    reject_as_key {
        let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, false)?;
    }: _(signer_origin, ephemeral_multisig, proposal_id)
    verify {
        ensure_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    accept_multisig_signer_as_identity {
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::from(alice.did())], 1)?;
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
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let bob = <UserBuilder<T>>::default().build("bob",);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::Account(bob.account())], 1)?;
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
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let bob = <UserBuilder<T>>::default().build("bob",);
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::from(alice.did())], 1)?;
        let origin = RawOrigin::Signed(multisig.clone());
    }: _(origin, Signatory::Account(bob.account()))
    verify {
        ensure!(
            <identity::Authorizations<T>>::iter_prefix_values(Signatory::Account(bob.account()))
            .into_iter()
            .max_by_key(|x| x.auth_id)
            .is_some(),
            "add_multisig_signer"
        );
    }

    remove_multisig_signer {
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let bob = <UserBuilder<T>>::default().generate_did().build("bob");
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

        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::from(alice.did())], 1)?;
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let count = <identity::Authorizations<T>>::iter().count();
    }: _(alice.origin(), multisig.clone(), signers)
    verify {
        ensure!(
            <identity::Authorizations<T>>::iter().count() > count,
            "add_multisig_signers_via_creator"
        );
    }

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 10 .. MAX_SIGNERS;

        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let multisig = generate_multisig_with_signers::<T>(alice.account(), alice.origin(), signers.clone(), 1)?;
        let num_of_signers = <NumberOfSigners<T>>::get(multisig.clone());
    }: _(alice.origin(), multisig.clone(), signers[1..].to_vec())
    verify {
        assert_ne!(num_of_signers, <NumberOfSigners<T>>::get(multisig.clone()));
        ensure!(
            num_of_signers > <NumberOfSigners<T>>::get(multisig.clone()),
            "remove_multisig_signers_via_creator"
        );
    }


    change_sigs_required {
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
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

        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, MAX_SIGNERS as usize);
        let multisig = generate_multisig::<T>(alice.account(), alice.origin(), signers)?;
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let origin = RawOrigin::Signed(multisig.clone());
        let sigs_required = <MultiSigSignsRequired<T>>::get(&multisig);
    }: _(origin, signers, i as u64)
    verify {
        ensure!(
            sigs_required != <MultiSigSignsRequired<T>>::get(&multisig),
            "change_all_signers_and_sigs_required"
        );
    }

    make_multisig_signer {
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::from(alice.did())], 1)?;
        let old_record = <Identity<T>>::did_records(alice.did());
    }: _(alice.origin(), multisig)
    verify {
        ensure!(
            old_record != <Identity<T>>::did_records(alice.did()),
            "make_multisig_signer"
        );
    }

    make_multisig_primary {
        let alice = <UserBuilder<T>>::default().generate_did().build("alice");
        let multisig = <MultiSig<T>>::get_next_multisig_address(alice.account().clone());
        <MultiSig<T>>::create_multisig(alice.origin().into(), vec![Signatory::from(alice.did())], 1)?;
        let old_record = <Identity<T>>::did_records(alice.did());
    }: _(alice.origin(), multisig, None)
    verify {
        ensure!(
            old_record != <Identity<T>>::did_records(alice.did()),
            "make_multisig_primary"
        );
    }
}
