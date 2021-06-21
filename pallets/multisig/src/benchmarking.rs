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

use crate::*;
use frame_benchmarking::benchmarks;
use frame_system::RawOrigin;
use polymesh_common_utilities::{
    benchs::{AccountIdOf, User, UserBuilder},
    TestUtilsFn,
};

pub type MultiSig<T> = crate::Module<T>;
pub type Identity<T> = identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Module<T>;

fn generate_signers<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    signers: &mut Vec<Signatory<T::AccountId>>,
    n: usize,
) {
    signers.extend((0..n).map(|x| {
        Signatory::Account(
            <UserBuilder<T>>::default()
                .seed(x as u32)
                .build("key")
                .account,
        )
    }));
}

fn get_last_auth_id<T: Config>(signatory: &Signatory<T::AccountId>) -> u64 {
    <identity::Authorizations<T>>::iter_prefix_values(signatory)
        .into_iter()
        .map(|x| x.auth_id)
        .max()
        .unwrap_or(0)
}

fn generate_multisig_with_extra_signers<T: Config + TestUtilsFn<AccountIdOf<T>>>(
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
    )
    .unwrap();
    Ok(multisig)
}

pub type MultisigSetupResult<T, AccountId> = (
    User<T>,
    AccountId,
    Vec<Signatory<AccountId>>,
    RawOrigin<AccountId>,
    RawOrigin<AccountId>,
);

fn generate_multisig_for_alice_wo_accepting<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    singers_required: u32,
) -> Result<MultisigSetupResult<T, T::AccountId>, DispatchError> {
    let alice = <UserBuilder<T>>::default().generate_did().build("alice");
    let mut signers = vec![Signatory::from(alice.did())];
    let multisig = generate_multisig_with_extra_signers::<T>(
        &alice,
        &mut signers,
        total_signers - 1,
        singers_required,
    )
    .unwrap();
    let signer_origin = match signers.last().cloned().unwrap() {
        Signatory::Account(account) => RawOrigin::Signed(account.clone()),
        _ => alice.origin().clone(),
    };
    Ok((
        alice,
        multisig.clone(),
        signers,
        signer_origin,
        RawOrigin::Signed(multisig),
    ))
}

fn generate_multisig_for_alice<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    singers_required: u32,
) -> Result<MultisigSetupResult<T, T::AccountId>, DispatchError> {
    let (alice, multisig, signers, signer_origin, multisig_origin) =
        generate_multisig_for_alice_wo_accepting::<T>(total_signers, singers_required).unwrap();
    for signer in &signers {
        let auth_id = get_last_auth_id::<T>(signer);
        <MultiSig<T>>::unsafe_accept_multisig_signer(signer.clone(), auth_id).unwrap();
    }
    Ok((
        alice,
        multisig.clone(),
        signers,
        signer_origin,
        multisig_origin,
    ))
}

pub type ProposalSetupResult<T, AccountId, Proposal> = (
    User<T>,
    AccountId,
    Vec<Signatory<AccountId>>,
    RawOrigin<AccountId>,
    u64,
    Box<Proposal>,
    AccountId,
);

fn generate_multisig_and_proposal_for_alice<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    singers_required: u32,
) -> Result<ProposalSetupResult<T, T::AccountId, T::Proposal>, DispatchError> {
    let (alice, multisig, signers, signer_origin, _) =
        generate_multisig_for_alice::<T>(total_signers, singers_required).unwrap();
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

fn generate_multisig_and_create_proposal<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    singers_required: u32,
    create_as_key: bool,
) -> Result<ProposalSetupResult<T, T::AccountId, T::Proposal>, DispatchError> {
    let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) =
        generate_multisig_and_proposal_for_alice::<T>(total_signers, singers_required).unwrap();
    if create_as_key {
        <MultiSig<T>>::create_proposal_as_key(
            signer_origin.clone().into(),
            multisig.clone(),
            proposal.clone(),
            None,
            true,
        )
        .unwrap();
    } else {
        <MultiSig<T>>::create_proposal_as_identity(
            alice.origin().into(),
            multisig.clone(),
            proposal.clone(),
            None,
            true,
        )
        .unwrap();
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

macro_rules! assert_proposal_created {
    ($proposal_id:ident, $multisig:ident) => {
        assert!($proposal_id < <MultiSig<T>>::ms_tx_done($multisig));
    };
}

macro_rules! assert_vote_cast {
    ($proposal_id:ident, $multisig:ident, $signatory:expr) => {
        assert!(<MultiSig<T>>::votes(($multisig, $signatory, $proposal_id)));
    };
}

macro_rules! assert_number_of_signers {
    ($expected_number:expr, $multisig:expr) => {
        assert!(<NumberOfSigners<T>>::get($multisig) == $expected_number);
    };
}

const MAX_SIGNERS: u32 = 256;

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_multisig {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;
        let (alice, multisig, signers, _, _) = generate_multisig_for_alice::<T>(i, 1).unwrap();
    }: _(alice.origin(), signers, i as u64)
    verify {
        assert!(<MultiSigToIdentity<T>>::contains_key(multisig), "create_multisig");
    }

    create_or_approve_proposal_as_identity {
        let (alice, multisig, signers, _, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(1, 1).unwrap();
    }: _(alice.origin(), ephemeral_multisig, proposal, Some(1337u32.into()), true)
    verify {
        assert_proposal_created!(proposal_id, multisig);
    }

    create_or_approve_proposal_as_key {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(2, 1).unwrap();
    }: _(signer_origin, ephemeral_multisig, proposal, Some(1337u32.into()), true)
    verify {
        assert_proposal_created!(proposal_id, multisig);
    }

    create_proposal_as_identity {
        let (alice, multisig, _, _, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(1, 1).unwrap();
    }: _(alice.origin(), ephemeral_multisig, proposal, Some(1337u32.into()), true)
    verify {
        assert_proposal_created!(proposal_id, multisig);
    }

    create_proposal_as_key {
        let (_, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(2, 1).unwrap();
    }: _(signer_origin, ephemeral_multisig, proposal, Some(1337u32.into()), true)
    verify {
        assert_proposal_created!(proposal_id, multisig);
    }

    approve_as_identity {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, true).unwrap();
    }: _(alice.origin(), ephemeral_multisig, proposal_id)
    verify {
        assert_vote_cast!(proposal_id, multisig, Signatory::from(alice.did()));
    }

    approve_as_key {
        let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, false).unwrap();
    }: _(signer_origin, ephemeral_multisig, proposal_id)
    verify {
        assert_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    reject_as_identity {
        let (alice, multisig, _, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, true).unwrap();
    }: _(alice.origin(), ephemeral_multisig, proposal_id)
    verify {
        assert_vote_cast!(proposal_id, multisig, Signatory::from(alice.did()));
    }

    reject_as_key {
        let (alice, multisig, signers, signer_origin, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2, false).unwrap();
    }: _(signer_origin, ephemeral_multisig, proposal_id)
    verify {
        assert_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    accept_multisig_signer_as_identity {
        let (alice, multisig, _, _, _) = generate_multisig_for_alice_wo_accepting::<T>(1, 1).unwrap();
        let alice_auth_id = get_last_auth_id::<T>(&Signatory::from(alice.did()));
        assert_number_of_signers!(0, multisig.clone());
    }: _(alice.origin(), alice_auth_id)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    accept_multisig_signer_as_key {
        let (alice, multisig, signers, signer_origin, _) = generate_multisig_for_alice_wo_accepting::<T>(2, 1).unwrap();
        let signer_auth_id = get_last_auth_id::<T>(&signers.last().unwrap());
        assert_number_of_signers!(0, multisig.clone());
    }: _(signer_origin, signer_auth_id)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    add_multisig_signer {
        let (alice, multisig, _, _, multisig_origin) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let bob = Signatory::Account(<UserBuilder<T>>::default().build("bob").account());
        let ephemeral_bob = bob.clone();
        let original_last_auth = get_last_auth_id::<T>(&bob);
    }: _(multisig_origin, ephemeral_bob)
    verify {
        assert!(original_last_auth < get_last_auth_id::<T>(&bob));
    }

    remove_multisig_signer {
        let (alice, multisig, _, _, multisig_origin) = generate_multisig_for_alice::<T>(2, 1).unwrap();
        assert_number_of_signers!(2, multisig.clone());
        let alice_signer = Signatory::from(alice.did());
    }: _(multisig_origin, alice_signer)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    add_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let mut signers = vec![];
        generate_signers::<T>(&mut signers, i as usize);
        let last_signer = signers.last().cloned().unwrap();
        let original_last_auth = get_last_auth_id::<T>(&last_signer);
    }: _(alice.origin(), multisig, signers)
    verify {
        assert!(original_last_auth < get_last_auth_id::<T>(&last_signer));
    }

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. MAX_SIGNERS;

        let (alice, multisig, signers, _, _) = generate_multisig_for_alice::<T>(1 + i, 1).unwrap();
        let signers_to_remove = signers[1..].to_vec();
        assert_number_of_signers!(1 + i as u64, multisig.clone());
        let ephemeral_multisig = multisig.clone();
    }: _(alice.origin(), ephemeral_multisig, signers_to_remove)
    verify {
        assert_number_of_signers!(1, multisig.clone());
    }

    change_sigs_required {
        let (_, multisig, _, _, multisig_origin) = generate_multisig_for_alice::<T>(2, 2).unwrap();
    }: _(multisig_origin, 1)
    verify {
        assert!(<MultiSigSignsRequired<T>>::get(&multisig) == 1);
    }

    make_multisig_signer {
        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let ephemeral_multisig = multisig.clone();
        let ms_signer = Signatory::Account(multisig);
    }: _(alice.origin(), ephemeral_multisig)
    verify {
        assert!(<Identity<T>>::did_records(alice.did()).secondary_keys.iter().any(|sk| sk.signer == ms_signer));
    }

    make_multisig_primary {
        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let ephemeral_multisig = multisig.clone();
    }: _(alice.origin(), ephemeral_multisig, None)
    verify {
        assert!(<Identity<T>>::did_records(alice.did()).primary_key == multisig);
    }

    execute_scheduled_proposal {
        let (alice, multisig, _, _, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(1, 1, false).unwrap();
        let ephemeral_proposal_id = proposal_id.clone();
    }: _(RawOrigin::Root, ephemeral_multisig, ephemeral_proposal_id, alice.did(), 0)
    verify {
        assert!(<ProposalDetail<T>>::get((&multisig, proposal_id)).status == ProposalStatus::ExecutionSuccessful);
    }
}
