use super::{
    storage::{
        get_identity_id, register_keyring_account, register_keyring_account_with_balance, Call,
        TestStorage,
    },
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok, dispatch::DispatchError};
use frame_system;
use pallet_balances as balances;
use pallet_committee as committee;
use pallet_group as group;
use pallet_pips::{
    self as pips, DepositInfo, Error, PipDescription, PipsMetadata, Referendum, ReferendumState,
    ReferendumType, Url, VotingResult,
};
use pallet_treasury as treasury;
use polymesh_common_utilities::traits::transaction_payment::CddAndFeeDetails;
use polymesh_primitives::{Beneficiary, Signatory};
use sp_core::sr25519::Public;
use test_client::AccountKeyring;

type System = frame_system::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Pips = pips::Module<TestStorage>;
type Group = group::Module<TestStorage, group::Instance1>;
type Committee = committee::Module<TestStorage, committee::Instance1>;
type Treasury = treasury::Module<TestStorage>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

fn make_proposal(value: u64) -> Call {
    Call::Pips(pips::Call::set_min_proposal_deposit(value.into()))
}

fn fast_forward_to(n: u64) {
    let block_number = System::block_number();
    (block_number..n).for_each(|block| {
        assert_ok!(Pips::end_block(block));
        System::set_block_number(block + 1);
    });
}

pub fn assert_balance(acc: Public, free: u128, locked: u128) {
    assert_eq!(Balances::free_balance(&acc), free);
    assert_eq!(Balances::usable_balance(&acc), free - locked);
}

#[test]
fn starting_a_proposal_works() {
    ExtBuilder::default()
        .build()
        .execute_with(starting_a_proposal_works_we)
}

fn starting_a_proposal_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    TestStorage::set_payer_context(Some(AccountKeyring::Alice.public()));
    let alice_acc = AccountKeyring::Alice.public();
    let alice_signer = Origin::signed(alice_acc.clone());
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 300).unwrap();

    // Error when min deposit requirements are not met
    assert_err!(
        Pips::propose(
            alice_signer.clone(),
            Box::new(proposal.clone()),
            40,
            Some(proposal_url.clone()),
            Some(proposal_desc.clone()),
            None,
        ),
        Error::<TestStorage>::IncorrectDeposit
    );

    // Account 6 starts a proposal with min deposit
    assert_ok!(Pips::propose(
        alice_signer.clone(),
        Box::new(proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc),
        None,
    ));

    assert_eq!(Balances::free_balance(&alice_acc), 158);

    assert_eq!(Pips::proposed_by(alice_acc.clone()), vec![0]);
    assert_eq!(
        Pips::proposal_result(0),
        VotingResult {
            ayes_count: 1,
            ayes_stake: 60,
            nays_count: 0,
            nays_stake: 0,
        }
    );
}

#[test]
fn closing_a_proposal_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(closing_a_proposal_works_we);
}

fn closing_a_proposal_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let index = 0;
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    // Voting majority
    let root = Origin::from(frame_system::RawOrigin::Root);

    TestStorage::set_payer_context(Some(AccountKeyring::Alice.public()));
    let alice_acc = AccountKeyring::Alice.public();
    let alice_signer = Origin::signed(alice_acc.clone());
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 300).unwrap();

    // Alice starts a proposal with min deposit
    assert_ok!(Pips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc),
        None
    ));

    assert_eq!(Balances::free_balance(&alice_acc), 168);
    assert_eq!(
        Pips::proposal_result(0),
        VotingResult {
            ayes_count: 1,
            ayes_stake: 50,
            nays_count: 0,
            nays_stake: 0,
        }
    );

    assert_ok!(Pips::kill_proposal(root, index));
    assert_eq!(Balances::free_balance(&alice_acc), 218);
}

#[test]
fn creating_a_referendum_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(creating_a_referendum_works_we);
}

fn creating_a_referendum_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let alice_acc = AccountKeyring::Alice.public();
    TestStorage::set_payer_context(Some(alice_acc.clone()));
    let alice_signer = Origin::signed(alice_acc.clone());
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 300).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    TestStorage::set_payer_context(Some(bob_acc));
    let bob_signer = Origin::signed(bob_acc.clone());
    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 200);
    // Voting majority
    let root = Origin::from(frame_system::RawOrigin::Root);

    TestStorage::set_payer_context(Some(alice_acc));
    assert_ok!(Pips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url),
        Some(proposal_desc),
        None
    ));

    // Cannot prune proposal at this stage
    assert_err!(
        Pips::prune_proposal(root.clone(), 0),
        Error::<TestStorage>::IncorrectProposalState
    );

    assert_err!(
        Pips::vote(bob_signer.clone(), 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);
    TestStorage::set_payer_context(Some(bob_acc));
    assert_ok!(Pips::vote(bob_signer.clone(), 0, true, 50));

    // Cannot prune proposal at this stage
    assert_err!(
        Pips::prune_proposal(root.clone(), 0),
        Error::<TestStorage>::IncorrectProposalState
    );

    assert_eq!(
        Pips::proposal_result(0),
        VotingResult {
            ayes_count: 2,
            ayes_stake: 100,
            nays_count: 0,
            nays_stake: 0,
        }
    );

    assert_eq!(Balances::free_balance(&alice_acc), 168);
    assert_eq!(Balances::free_balance(&bob_acc), 109);

    fast_forward_to(120);

    // Cannot prune referendum at this stage
    assert_err!(
        Pips::prune_proposal(root.clone(), 0),
        Error::<TestStorage>::IncorrectReferendumState
    );

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Pending,
            referendum_type: ReferendumType::Community,
            enactment_period: 0,
        })
    );

    assert_eq!(Balances::free_balance(&alice_acc), 218);
    assert_eq!(Balances::free_balance(&bob_acc), 159);
}

#[test]
fn enacting_a_referendum_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(enacting_a_referendum_works_we);
}

fn enacting_a_referendum_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let alice_signer = Origin::signed(AccountKeyring::Alice.public());
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 300).unwrap();
    let bob_signer = Origin::signed(AccountKeyring::Bob.public());
    let _ = register_keyring_account_with_balance(AccountKeyring::Bob, 100).unwrap();

    // Voting majority
    let root = Origin::from(frame_system::RawOrigin::Root);

    assert_ok!(Pips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc),
        None
    ));

    assert_err!(
        Pips::vote(bob_signer.clone(), 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);

    assert_ok!(Pips::vote(bob_signer.clone(), 0, true, 50));

    assert_eq!(
        Pips::proposal_result(0),
        VotingResult {
            ayes_count: 2,
            ayes_stake: 100,
            nays_count: 0,
            nays_stake: 0,
        }
    );

    fast_forward_to(120);

    // Cannot prune referendum at this stage
    assert_err!(
        Pips::prune_proposal(root.clone(), 0),
        Error::<TestStorage>::IncorrectReferendumState
    );

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Pending,
            referendum_type: ReferendumType::Community,
            enactment_period: 0,
        })
    );

    assert_err!(
        Pips::enact_referendum(bob_signer.clone(), 0),
        DispatchError::BadOrigin
    );
    assert_ok!(Pips::enact_referendum(root.clone(), 0));

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::Community,
            enactment_period: 220,
        })
    );

    // Cannot prune referendum at this stage
    assert_err!(
        Pips::prune_proposal(root.clone(), 0),
        Error::<TestStorage>::IncorrectReferendumState
    );

    fast_forward_to(221);

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::Community,
            enactment_period: 220,
        })
    );

    // Can now prune referendum
    assert_ok!(Pips::prune_proposal(root.clone(), 0));

    assert_eq!(Pips::referendums(0), None);
    assert_eq!(Pips::proposals(0), None);
    assert_eq!(Pips::proposal_metadata(0), None);
}

#[test]
fn fast_tracking_a_proposal_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(fast_tracking_a_proposal_works_we);
}

fn fast_tracking_a_proposal_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let index = 0;
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let root = Origin::from(frame_system::RawOrigin::Root);

    // Alice and Bob are committee members
    let alice_signer = Origin::signed(AccountKeyring::Alice.public());
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 100).unwrap();
    let bob_signer = Origin::signed(AccountKeyring::Bob.public());
    let bob_did = register_keyring_account_with_balance(AccountKeyring::Bob, 100).unwrap();

    let charlie_signer = Origin::signed(AccountKeyring::Charlie.public());
    let _ = register_keyring_account_with_balance(AccountKeyring::Charlie, 300).unwrap();

    Group::reset_members(root.clone(), vec![alice_did, bob_did]).unwrap();

    assert_eq!(Committee::members(), vec![bob_did, alice_did]);

    assert_ok!(Pips::propose(
        charlie_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc),
        None
    ));

    assert_err!(
        Pips::vote(bob_signer.clone(), index, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );

    // only a committee member can fast track a proposal
    assert_err!(
        Pips::fast_track_proposal(charlie_signer.clone(), index),
        Error::<TestStorage>::NotACommitteeMember
    );

    // Alice can fast track because she is a GC member
    assert_ok!(Pips::fast_track_proposal(alice_signer.clone(), index));
    assert_err!(
        Pips::enact_referendum(bob_signer.clone(), 0),
        DispatchError::BadOrigin
    );
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Pending,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 0,
        })
    );

    assert_ok!(Pips::enact_referendum(root, 0));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 101,
        })
    );

    // It executes automatically the referendum at block 101.
    fast_forward_to(120);
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::FastTracked,
            enactment_period: 101,
        })
    );
}

#[test]

fn emergency_referendum_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(emergency_referendum_works_we);
}

fn emergency_referendum_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let root = Origin::from(frame_system::RawOrigin::Root);

    // Alice and Bob are committee members
    let alice_signer = Origin::signed(AccountKeyring::Alice.public());
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 60).unwrap();

    Group::reset_members(root.clone(), vec![alice_did]).unwrap();

    assert_eq!(Committee::members(), vec![alice_did]);

    // Alice is a committee member

    assert_ok!(Pips::emergency_referendum(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        Some(proposal_url.clone()),
        Some(proposal_desc),
        None,
    ));

    fast_forward_to(20);

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Pending,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 0,
        })
    );

    assert_err!(
        Pips::enact_referendum(alice_signer.clone(), 0),
        DispatchError::BadOrigin
    );

    fast_forward_to(101);
    assert_ok!(Pips::enact_referendum(root, 0));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 201,
        })
    );

    fast_forward_to(202);
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 201,
        })
    );
}

#[test]
fn reject_referendum_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(reject_referendum_works_we);
}

fn reject_referendum_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let root = Origin::from(frame_system::RawOrigin::Root);

    // Alice and Bob are committee members
    let alice_signer = Origin::signed(AccountKeyring::Alice.public());
    let alice_did = register_keyring_account_with_balance(AccountKeyring::Alice, 60).unwrap();

    Group::reset_members(root.clone(), vec![alice_did]).unwrap();

    assert_eq!(Committee::members(), vec![alice_did]);

    // Alice is a committee member
    assert_ok!(Pips::emergency_referendum(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        Some(proposal_url.clone()),
        Some(proposal_desc),
        None,
    ));

    fast_forward_to(20);

    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Pending,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 0,
        })
    );

    assert_err!(
        Pips::enact_referendum(alice_signer.clone(), 0),
        DispatchError::BadOrigin
    );

    fast_forward_to(101);
    assert_ok!(Pips::reject_referendum(root, 0));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Rejected,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 0,
        })
    );
}

#[test]
fn updating_pips_variables_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(updating_pips_variables_works_we);
}

fn updating_pips_variables_works_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);

    let alice_signer = Origin::signed(AccountKeyring::Alice.public());
    let _ = register_keyring_account_with_balance(AccountKeyring::Alice, 60).unwrap();
    // config variables can be updated only through committee
    assert_eq!(Pips::min_proposal_deposit(), 50);
    assert_err!(
        Pips::set_min_proposal_deposit(alice_signer.clone(), 10),
        DispatchError::BadOrigin
    );
    assert_ok!(Pips::set_min_proposal_deposit(root.clone(), 10));
    assert_eq!(Pips::min_proposal_deposit(), 10);

    assert_eq!(Pips::quorum_threshold(), 70);
    assert_ok!(Pips::set_quorum_threshold(root.clone(), 100));
    assert_eq!(Pips::quorum_threshold(), 100);

    assert_eq!(Pips::proposal_duration(), 10);
    assert_ok!(Pips::set_proposal_duration(root.clone(), 100));
    assert_eq!(Pips::proposal_duration(), 100);

    assert_eq!(Pips::proposal_cool_off_period(), 100);
    assert_ok!(Pips::set_proposal_cool_off_period(root.clone(), 10));
    assert_eq!(Pips::proposal_cool_off_period(), 10);

    assert_eq!(Pips::default_enactment_period(), 100);
    assert_ok!(Pips::set_default_enactment_period(root.clone(), 10));
    assert_eq!(Pips::default_enactment_period(), 10);
}

#[test]
fn amend_pips_details_during_cool_off_period() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(amend_pips_details_during_cool_off_period_we);
}

fn amend_pips_details_during_cool_off_period_we() {
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let alice_acc = AccountKeyring::Alice.public();
    let alice = Origin::signed(alice_acc.clone());
    let _ = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let _ = register_keyring_account(AccountKeyring::Bob).unwrap();
    print!(
        "Block no - {:?}",
        <frame_system::Module<TestStorage>>::block_number()
    );
    // 1. Create Pips proposal
    assert_ok!(Pips::propose(
        alice.clone(),
        Box::new(proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc),
        None
    ));
    fast_forward_to(50);

    // 2. Amend proposal during cool-off.
    let new_url: Url = b"www.xyz.com".into();
    let new_desc: PipDescription = b"New description".into();
    assert_ok!(Pips::amend_proposal(
        alice.clone(),
        0,
        Some(new_url.clone()),
        Some(new_desc.clone())
    ));

    assert_err!(
        Pips::amend_proposal(bob.clone(), 0, None, None),
        Error::<TestStorage>::BadOrigin
    );

    assert_eq!(
        Pips::proposal_metadata(0),
        Some(PipsMetadata {
            proposer: alice_acc.clone(),
            id: 0,
            cool_off_until: 100,
            end: 110,
            url: Some(new_url),
            description: Some(new_desc),
        })
    );

    // 3. Bound/Unbound additional POLYX.
    assert_eq!(
        Pips::deposits(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 60
        }
    );
    assert_ok!(Pips::bond_additional_deposit(alice.clone(), 0, 100));
    assert_eq!(
        Pips::deposits(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 160
        }
    );
    assert_ok!(Pips::unbond_deposit(alice.clone(), 0, 50));
    assert_eq!(
        Pips::deposits(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 110
        }
    );
    assert_err!(
        Pips::unbond_deposit(alice.clone(), 0, 90),
        Error::<TestStorage>::IncorrectDeposit
    );

    // 4. Move out the cool-off period and ensure Pips is inmutable.
    fast_forward_to(103);
    assert_err!(
        Pips::amend_proposal(alice.clone(), 0, None, None),
        Error::<TestStorage>::ProposalIsImmutable
    );
}

#[test]
fn cancel_pips_during_cool_off_period() {
    ExtBuilder::default()
        .build()
        .execute_with(cancel_pips_during_cool_off_period_we);
}

fn cancel_pips_during_cool_off_period_we() {
    let alice_proposal = make_proposal(42);
    let bob_proposal = make_proposal(1);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let alice = Origin::signed(AccountKeyring::Alice.public());
    let _ = register_keyring_account(AccountKeyring::Alice).unwrap();
    let bob = Origin::signed(AccountKeyring::Bob.public());
    let _ = register_keyring_account(AccountKeyring::Bob).unwrap();
    let root = Origin::from(frame_system::RawOrigin::Root);

    // 1. Create Pips proposals
    assert_ok!(Pips::propose(
        alice.clone(),
        Box::new(alice_proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc),
        None
    ));

    assert_ok!(Pips::propose(
        bob.clone(),
        Box::new(bob_proposal.clone()),
        60,
        None,
        None,
        None
    ));

    // 2. Cancel Alice's proposal during cool-off period.
    fast_forward_to(50);
    assert_ok!(Pips::cancel_proposal(alice.clone(), 0));

    // Can prune cancelled proposals
    assert_ok!(Pips::prune_proposal(root.clone(), 0));

    // Check proposal is pruned from storage
    assert_eq!(Pips::referendums(0), None);
    assert_eq!(Pips::proposals(0), None);
    assert_eq!(Pips::proposal_metadata(0), None);

    // 3. Try to cancel Bob's proposal after cool-off period.
    fast_forward_to(101);
    assert_err!(
        Pips::cancel_proposal(bob.clone(), 1),
        Error::<TestStorage>::ProposalIsImmutable
    );

    // 4. Double check current proposals
    assert_eq!(
        Pips::proposal_metadata(1),
        Some(PipsMetadata {
            proposer: AccountKeyring::Bob.public(),
            id: 1,
            cool_off_until: 100,
            end: 110,
            url: None,
            description: None,
        })
    );
}

#[test]
fn update_referendum_enactment_period() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .governance_committee_vote_threshold((2, 3))
        .build()
        .execute_with(update_referendum_enactment_period_we);
}

fn update_referendum_enactment_period_we() {
    let root = Origin::from(frame_system::RawOrigin::Root);
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();

    let proposal_a = make_proposal(42);
    let proposal_b = make_proposal(107);

    // Bob is the release coordinator.
    let bob_id = get_identity_id(AccountKeyring::Bob).expect("Bob is part of the committee");
    assert_ok!(Committee::set_release_coordinator(root.clone(), bob_id));

    // Alice submit 2 referendums in different moments.

    assert_ok!(Pips::emergency_referendum(
        alice.clone(),
        Box::new(proposal_a.clone()),
        None,
        None,
        None,
    ));
    assert_ok!(Pips::enact_referendum(root.clone(), 0));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 100,
        })
    );

    fast_forward_to(50);

    assert_ok!(Pips::emergency_referendum(
        alice.clone(),
        Box::new(proposal_b.clone()),
        Some(proposal_url),
        Some(proposal_desc),
        None,
    ));
    assert_ok!(Pips::enact_referendum(root.clone(), 1));
    assert_eq!(
        Pips::referendums(1),
        Some(Referendum {
            id: 1,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 150,
        })
    );

    // Alice cannot update the enact period.
    assert_err!(
        Pips::override_referendum_enactment_period(alice.clone(), 0, Some(200)),
        Error::<TestStorage>::BadOrigin
    );

    // Bob updates referendum to execute `b` now(next block), and `a` in the future.
    assert_ok!(Pips::override_referendum_enactment_period(
        bob.clone(),
        1,
        None
    ));
    fast_forward_to(52);
    assert_eq!(
        Pips::referendums(1),
        Some(Referendum {
            id: 1,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 51,
        })
    );

    assert_ok!(Pips::override_referendum_enactment_period(
        bob.clone(),
        0,
        Some(200)
    ));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Scheduled,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 200,
        })
    );

    // Bob cannot update if referendum is already executed.
    fast_forward_to(201);
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            state: ReferendumState::Executed,
            referendum_type: ReferendumType::Emergency,
            enactment_period: 200,
        })
    );
    assert_err!(
        Pips::override_referendum_enactment_period(bob.clone(), 0, Some(300)),
        Error::<TestStorage>::IncorrectReferendumState
    );
}

#[test]
fn proposal_with_beneficiares() {
    let committee = [AccountKeyring::Alice.public(), AccountKeyring::Bob.public()].to_vec();
    ExtBuilder::default()
        .governance_committee(committee)
        .governance_committee_vote_threshold((2, 3))
        .balance_factor(10)
        .build()
        .execute_with(proposal_with_beneficiares_we);
}

fn proposal_with_beneficiares_we() {
    // 0. Create accounts
    System::set_block_number(1);
    let root = Origin::from(frame_system::RawOrigin::Root);
    let alice = AccountKeyring::Alice.public();
    let charlie = AccountKeyring::Charlie.public();
    let dave = AccountKeyring::Dave.public();
    let eve = AccountKeyring::Eve.public();
    TestStorage::set_payer_context(Some(charlie));
    let charlie_id = register_keyring_account_with_balance(AccountKeyring::Charlie, 200).unwrap();
    TestStorage::set_payer_context(Some(dave));
    let dave_id = register_keyring_account_with_balance(AccountKeyring::Dave, 200).unwrap();
    TestStorage::set_payer_context(Some(eve));
    let _ = register_keyring_account_with_balance(AccountKeyring::Eve, 1_000_000).unwrap();
    let eve_acc = Origin::signed(AccountKeyring::Eve.public());

    // 2. Charlie creates a new proposal with 2 beneificiares
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: PipDescription = b"Test description".into();
    let beneficiaries = vec![
        Beneficiary {
            id: charlie_id,
            amount: 200,
        },
        Beneficiary {
            id: dave_id,
            amount: 800,
        },
    ];

    TestStorage::set_payer_context(Some(charlie));
    assert_ok!(Pips::propose(
        Origin::signed(charlie.clone()),
        Box::new(proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc),
        Some(beneficiaries),
    ));

    // 2. Alice can fast track because she is a GC member
    TestStorage::set_payer_context(Some(alice));
    assert_ok!(Pips::fast_track_proposal(Origin::signed(alice), 0));
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            referendum_type: ReferendumType::FastTracked,
            state: ReferendumState::Pending,
            enactment_period: 0,
        })
    );
    assert_ok!(Pips::enact_referendum(root, 0));

    // 3. It executes automatically the referendum at block 101.
    assert_eq!(Balances::free_balance(&charlie), 118);
    assert_eq!(Balances::free_balance(&dave), 159);

    // Top up the treasury account so it can disburse
    assert_ok!(Treasury::reimbursement(eve_acc.clone(), 1_000));

    fast_forward_to(120);
    assert_eq!(
        Pips::referendums(0),
        Some(Referendum {
            id: 0,
            referendum_type: ReferendumType::FastTracked,
            state: ReferendumState::Executed,
            enactment_period: 101,
        })
    );
}
