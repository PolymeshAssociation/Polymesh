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

use frame_benchmarking::benchmarks;
use frame_support::storage::StorageDoubleMap;
use frame_system::RawOrigin;

use polymesh_common_utilities::benchs::{AccountIdOf, User, UserBuilder};
use polymesh_common_utilities::TestUtilsFn;

use crate::*;

pub type MultiSig<T> = crate::Pallet<T>;
pub type Identity<T> = pallet_identity::Module<T>;
pub type Timestamp<T> = pallet_timestamp::Pallet<T>;

fn generate_signers<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    n: usize,
) -> (BoundedVec<T::AccountId, T::MaxSigners>, Vec<User<T>>) {
    let mut users = Vec::with_capacity(n);
    let signers = (0..n)
        .into_iter()
        .map(|x| {
            let user = UserBuilder::<T>::default().seed(x as u32).build("key");
            let account = user.account.clone();
            users.push(user);
            account
        })
        .collect::<Vec<_>>()
        .try_into()
        .unwrap();
    (signers, users)
}

fn get_last_auth_id<T: Config>(account: &T::AccountId) -> u64 {
    let signatory = Signatory::Account(account.clone());
    <pallet_identity::Authorizations<T>>::iter_prefix_values(signatory)
        .into_iter()
        .map(|x| x.auth_id)
        .max()
        .unwrap_or(0)
}

fn generate_multisig_with_extra_signers<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    caller: &User<T>,
    num_of_extra_signers: u32,
    num_of_signers_required: u32,
) -> Result<
    (
        T::AccountId,
        BoundedVec<T::AccountId, T::MaxSigners>,
        Vec<User<T>>,
    ),
    DispatchError,
> {
    let (signers, users) = generate_signers::<T>(num_of_extra_signers as usize);
    let multisig = MultiSig::<T>::get_next_multisig_address(caller.account()).expect("Next MS");
    MultiSig::<T>::create_multisig(
        caller.origin.clone().into(),
        signers.clone(),
        num_of_signers_required.into(),
    )
    .unwrap();
    Ok((multisig, signers, users))
}

pub type MultisigSetupResult<T, AccountId, MaxSigners> = (
    User<T>,
    AccountId,
    BoundedVec<AccountId, MaxSigners>,
    Vec<User<T>>,
    RawOrigin<AccountId>,
);

fn generate_multisig_for_alice_wo_accepting<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    signers_required: u32,
) -> Result<MultisigSetupResult<T, T::AccountId, T::MaxSigners>, DispatchError> {
    let alice = UserBuilder::<T>::default().generate_did().build("alice");
    let (multisig, signers, users) =
        generate_multisig_with_extra_signers::<T>(&alice, total_signers, signers_required).unwrap();
    Ok((
        alice,
        multisig.clone(),
        signers,
        users,
        RawOrigin::Signed(multisig),
    ))
}

fn generate_multisig_for_alice<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    signers_required: u32,
) -> Result<MultisigSetupResult<T, T::AccountId, T::MaxSigners>, DispatchError> {
    let (alice, multisig, signers, users, multisig_origin) =
        generate_multisig_for_alice_wo_accepting::<T>(total_signers, signers_required).unwrap();
    for signer in &signers {
        let auth_id = get_last_auth_id::<T>(signer);
        MultiSig::<T>::base_accept_multisig_signer(signer.clone(), auth_id).unwrap();
    }
    Ok((alice, multisig.clone(), signers, users, multisig_origin))
}

pub type ProposalSetupResult<T, AccountId, Proposal, MaxSigners> = (
    User<T>,
    AccountId,
    BoundedVec<AccountId, MaxSigners>,
    Vec<User<T>>,
    u64,
    Box<Proposal>,
    AccountId,
);

fn generate_multisig_and_proposal_for_alice<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    signers_required: u32,
) -> Result<ProposalSetupResult<T, T::AccountId, T::Proposal, T::MaxSigners>, DispatchError> {
    let (alice, multisig, signers, users, _) =
        generate_multisig_for_alice::<T>(total_signers, signers_required).unwrap();
    let proposal_id = MultiSig::<T>::ms_tx_done(multisig.clone());
    let proposal = Box::new(frame_system::Call::<T>::remark { remark: vec![] }.into());
    Ok((
        alice,
        multisig.clone(),
        signers,
        users,
        proposal_id,
        proposal,
        multisig,
    ))
}

fn generate_multisig_and_create_proposal<T: Config + TestUtilsFn<AccountIdOf<T>>>(
    total_signers: u32,
    signers_required: u32,
) -> Result<ProposalSetupResult<T, T::AccountId, T::Proposal, T::MaxSigners>, DispatchError> {
    let (alice, multisig, signers, users, proposal_id, proposal, ephemeral_multisig) =
        generate_multisig_and_proposal_for_alice::<T>(total_signers, signers_required).unwrap();
    // Use the first signer to create the proposal.
    MultiSig::<T>::create_proposal(
        users[0].origin().into(),
        multisig.clone(),
        proposal.clone(),
        None,
    )
    .unwrap();
    Ok((
        alice,
        multisig,
        signers,
        users,
        proposal_id,
        proposal,
        ephemeral_multisig,
    ))
}

macro_rules! assert_proposal_created {
    ($proposal_id:ident, $multisig:ident) => {
        assert!($proposal_id < MultiSig::<T>::ms_tx_done($multisig));
    };
}

macro_rules! assert_vote_cast {
    ($proposal_id:ident, $multisig:ident, $signatory:expr) => {
        assert!(MultiSig::<T>::votes(($multisig, $proposal_id), $signatory));
    };
}

macro_rules! assert_number_of_signers {
    ($expected_number:expr, $multisig:expr) => {
        assert!(NumberOfSigners::<T>::get($multisig) == $expected_number);
    };
}

benchmarks! {
    where_clause { where T: TestUtilsFn<AccountIdOf<T>> }

    create_multisig {
        // Number of signers
        let i in 1 .. T::MaxSigners::get() as u32;
        let (alice, multisig, signers, _, _) = generate_multisig_for_alice::<T>(i, 1).unwrap();
    }: _(alice.origin(), signers, i as u64)
    verify {
        assert!(CreatorDid::<T>::contains_key(multisig), "create_multisig");
    }

    create_proposal {
        let (_, multisig, _, users, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_proposal_for_alice::<T>(3, 3).unwrap();
    }: _(users[0].origin(), ephemeral_multisig, proposal, Some(1337u32.into()))
    verify {
        assert_proposal_created!(proposal_id, multisig);
    }

    approve {
        let (alice, multisig, signers, users, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(3, 3).unwrap();
    }: _(users[2].origin(), ephemeral_multisig, proposal_id, Weight::MAX)
    verify {
        assert_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    execute_proposal {
        let (alice, multisig, signers, users, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(3, 3).unwrap();
        let did = alice.did.expect("Alice must have a DID");
    }: {
      assert!(MultiSig::<T>::execute_proposal(&multisig, proposal_id, did, Weight::MAX).is_ok());
    }

    reject {
        let (alice, multisig, signers, users, proposal_id, proposal, ephemeral_multisig) = generate_multisig_and_create_proposal::<T>(2, 2).unwrap();
    }: _(users[1].origin(), ephemeral_multisig, proposal_id)
    verify {
        assert_vote_cast!(proposal_id, multisig, signers.last().unwrap());
    }

    accept_multisig_signer {
        let (alice, multisig, signers, users, _) = generate_multisig_for_alice_wo_accepting::<T>(2, 1).unwrap();
        let user = users.last().unwrap().clone();
        let signer_auth_id = get_last_auth_id::<T>(&signers.last().unwrap());
        assert_number_of_signers!(0, multisig.clone());
    }: _(user.origin(), signer_auth_id)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    add_multisig_signers {
        // Number of signers
        let i in 1 .. T::MaxSigners::get() as u32;

        let (alice, multisig, _, _, multisig_origin) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let (signers, _) = generate_signers::<T>(i as usize);
        let last_signer = signers.last().cloned().unwrap();
        let original_last_auth = get_last_auth_id::<T>(&last_signer);
    }: _(multisig_origin, signers)
    verify {
        assert!(original_last_auth < get_last_auth_id::<T>(&last_signer));
    }

    remove_multisig_signers {
        // Number of signers
        let i in 2 .. T::MaxSigners::get() as u32;

        let (alice, multisig, signers, _, multisig_origin) = generate_multisig_for_alice::<T>(i, 1).unwrap();
        let signers_to_remove = signers[1..].to_vec().try_into().unwrap();
        assert_number_of_signers!(i as u64, multisig.clone());
    }: _(multisig_origin, signers_to_remove)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    add_multisig_signers_via_creator {
        // Number of signers
        let i in 1 .. T::MaxSigners::get() as u32;

        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let (signers, _) = generate_signers::<T>(i as usize);
        let last_signer = signers.last().cloned().unwrap();
        let original_last_auth = get_last_auth_id::<T>(&last_signer);
    }: _(alice.origin(), multisig, signers)
    verify {
        assert!(original_last_auth < get_last_auth_id::<T>(&last_signer));
    }

    remove_multisig_signers_via_creator {
        // Number of signers
        let i in 2 .. T::MaxSigners::get() as u32;

        let (alice, multisig, signers, _, _) = generate_multisig_for_alice::<T>(i, 1).unwrap();
        let signers_to_remove = signers[1..].to_vec().try_into().unwrap();
        assert_number_of_signers!(i as u64, multisig.clone());
        let ephemeral_multisig = multisig.clone();
    }: _(alice.origin(), ephemeral_multisig, signers_to_remove)
    verify {
        assert_number_of_signers!(1, multisig);
    }

    change_sigs_required {
        let (_, multisig, _, _, multisig_origin) = generate_multisig_for_alice::<T>(2, 2).unwrap();
    }: _(multisig_origin, 1)
    verify {
        assert!(MultiSigSignsRequired::<T>::get(&multisig) == 1);
    }

    make_multisig_secondary {
        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
        let whole = Permissions::default();
    }: _(alice.origin(), multisig.clone(), Some(whole))
    verify {
        assert!(Identity::<T>::is_secondary_key(alice.did(), &multisig));
    }

    make_multisig_primary {
        let (alice, multisig, _, _, _) = generate_multisig_for_alice::<T>(1, 1).unwrap();
    }: _(alice.origin(), multisig.clone(), None)
    verify {
        assert!(Identity::<T>::get_primary_key(alice.did()) == Some(multisig));
    }

    change_sigs_required_via_creator {
        let (alice, multisig_account, _, _, _) = generate_multisig_for_alice::<T>(2, 2).unwrap();
    }: _(alice.origin(), multisig_account, 1)

    remove_creator_controls {
        let (alice, multisig_account, _, _, _) = generate_multisig_for_alice::<T>(2, 2).unwrap();
    }: _(alice.origin(), multisig_account)
}
