mod common;
use common::{
    storage::{get_identity_id, make_account, make_account_with_balance, Call, TestStorage},
    ExtBuilder,
};
use frame_support::{assert_err, assert_ok};
use frame_system;
use pallet_committee as committee;
use pallet_mips::{
    self as mips, DepositInfo, Error, MipDescription, MipsMetadata, MipsPriority, MipsState,
    PolymeshVotes, Referendum, Url,
};
use polymesh_runtime_balances as balances;
use polymesh_runtime_group as group;
use test_client::AccountKeyring;

type System = frame_system::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Mips = mips::Module<TestStorage>;
type Group = group::Module<TestStorage, group::Instance1>;
type Committee = committee::Module<TestStorage, committee::Instance1>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

fn make_proposal(value: u64) -> Call {
    Call::Mips(mips::Call::set_min_proposal_deposit(value.into()))
}

fn fast_forward_to(n: u64) {
    let block_number = System::block_number();
    (block_number..n).for_each(|block| {
        assert_ok!(Mips::end_block(block));
        System::set_block_number(block + 1);
    });
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
    let proposal_desc: MipDescription = b"Test description".into();

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account_with_balance(alice_acc, 300).unwrap();

    // Error when min deposit requirements are not met
    assert_err!(
        Mips::propose(
            alice_signer.clone(),
            Box::new(proposal.clone()),
            40,
            Some(proposal_url.clone()),
            Some(proposal_desc.clone())
        ),
        Error::<TestStorage>::InsufficientDeposit
    );

    // Account 6 starts a proposal with min deposit
    assert_ok!(Mips::propose(
        alice_signer.clone(),
        Box::new(proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc)
    ));

    assert_eq!(Balances::free_balance(&alice_acc), 158);

    assert_eq!(Mips::proposed_by(alice_acc.clone()), vec![0]);
    assert_eq!(
        Mips::voting(0),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![(alice_acc, 60)],
            nays: vec![],
        })
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
    let proposal_desc: MipDescription = b"Test description".into();

    // Voting majority
    let root = Origin::system(frame_system::RawOrigin::Root);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account_with_balance(alice_acc, 300).unwrap();

    // Alice starts a proposal with min deposit
    assert_ok!(Mips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc)
    ));

    assert_eq!(Balances::free_balance(&alice_acc), 168);
    assert_eq!(
        Mips::voting(0),
        Some(PolymeshVotes {
            index,
            ayes: vec![(alice_acc.clone(), 50)],
            nays: vec![],
        })
    );

    assert_ok!(Mips::kill_proposal(root, index));
    assert_eq!(Balances::free_balance(&alice_acc), 218);
    assert_eq!(Mips::voting(0), None);
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
    let proposal_desc: MipDescription = b"Test description".into();

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account_with_balance(alice_acc, 300).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, _) = make_account_with_balance(bob_acc, 200).unwrap();

    assert_ok!(Mips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url),
        Some(proposal_desc)
    ));

    assert_err!(
        Mips::vote(bob_signer.clone(), 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);
    assert_ok!(Mips::vote(bob_signer.clone(), 0, true, 50));

    assert_eq!(
        Mips::voting(0),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![(alice_acc.clone(), 50), (bob_acc.clone(), 50)],
            nays: vec![]
        })
    );

    assert_eq!(Balances::free_balance(&alice_acc), 168);
    assert_eq!(Balances::free_balance(&bob_acc), 109);

    fast_forward_to(120);

    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::Normal,
            state: MipsState::Ratified,
            enactment_period: 0,
            proposal: proposal.clone()
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
    let proposal_desc: MipDescription = b"Test description".into();

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account_with_balance(alice_acc, 300).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, _) = make_account_with_balance(bob_acc, 100).unwrap();

    // Voting majority
    let root = Origin::system(frame_system::RawOrigin::Root);

    assert_ok!(Mips::propose(
        alice_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc)
    ));

    assert_err!(
        Mips::vote(bob_signer.clone(), 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);

    assert_ok!(Mips::vote(bob_signer.clone(), 0, true, 50));

    assert_eq!(
        Mips::voting(0),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![(alice_acc.clone(), 50), (bob_acc.clone(), 50)],
            nays: vec![]
        })
    );

    fast_forward_to(120);

    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::Normal,
            state: MipsState::Ratified,
            enactment_period: 0,
            proposal: proposal.clone()
        })
    );

    assert_err!(
        Mips::enact_referendum(bob_signer.clone(), 0),
        Error::<TestStorage>::BadOrigin
    );
    assert_ok!(Mips::enact_referendum(root, 0));

    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::Normal,
            state: MipsState::Scheduled,
            enactment_period: 220,
            proposal: proposal.clone()
        })
    );

    fast_forward_to(221);

    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::Normal,
            state: MipsState::Executed,
            enactment_period: 220,
            proposal
        })
    );
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
    let proposal_desc: MipDescription = b"Test description".into();

    let root = Origin::system(frame_system::RawOrigin::Root);

    // Alice and Bob are committee members
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account_with_balance(alice_acc, 100).unwrap();
    let bob_acc = AccountKeyring::Bob.public();
    let (bob_signer, bob_did) = make_account_with_balance(bob_acc, 100).unwrap();

    let charlie_acc = AccountKeyring::Charlie.public();
    let (charlie_signer, _) = make_account_with_balance(charlie_acc, 300).unwrap();

    Group::reset_members(root.clone(), vec![alice_did, bob_did]).unwrap();

    assert_eq!(Committee::members(), vec![bob_did, alice_did]);

    assert_ok!(Mips::propose(
        charlie_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc)
    ));

    assert_err!(
        Mips::vote(bob_signer.clone(), index, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );

    // only a committee member can fast track a proposal
    assert_err!(
        Mips::fast_track_proposal(charlie_signer.clone(), index),
        Error::<TestStorage>::NotACommitteeMember
    );

    // Alice can fast track because she is a GC member
    assert_ok!(Mips::fast_track_proposal(alice_signer.clone(), index));
    assert_err!(
        Mips::enact_referendum(bob_signer.clone(), 0),
        Error::<TestStorage>::BadOrigin
    );
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Ratified,
            enactment_period: 0,
            proposal: proposal.clone()
        })
    );

    assert_ok!(Mips::enact_referendum(root, 0));
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Scheduled,
            enactment_period: 101,
            proposal: proposal.clone()
        })
    );

    // It executes automatically the referendum at block 101.
    fast_forward_to(120);
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Executed,
            enactment_period: 101,
            proposal
        })
    );
}

#[test]
fn submit_referendum_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(submit_referendum_works_we);
}

fn submit_referendum_works_we() {
    System::set_block_number(1);
    let proposal = make_proposal(42);

    let root = Origin::system(frame_system::RawOrigin::Root);

    // Alice and Bob are committee members
    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, alice_did) = make_account_with_balance(alice_acc, 60).unwrap();

    Group::reset_members(root.clone(), vec![alice_did]).unwrap();

    assert_eq!(Committee::members(), vec![alice_did]);

    // Alice is a committee member
    assert_ok!(Mips::submit_referendum(
        alice_signer.clone(),
        Box::new(proposal.clone())
    ));

    fast_forward_to(20);

    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Ratified,
            enactment_period: 0,
            proposal: proposal.clone()
        })
    );

    assert_err!(
        Mips::enact_referendum(alice_signer.clone(), 0),
        Error::<TestStorage>::BadOrigin
    );

    fast_forward_to(101);
    assert_ok!(Mips::enact_referendum(root, 0));
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Scheduled,
            enactment_period: 201,
            proposal: proposal.clone()
        })
    );

    fast_forward_to(202);
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Executed,
            enactment_period: 201,
            proposal
        })
    );
}

#[test]
fn updating_mips_variables_works() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(updating_mips_variables_works_we);
}

fn updating_mips_variables_works_we() {
    let root = Origin::system(frame_system::RawOrigin::Root);

    let alice_acc = AccountKeyring::Alice.public();
    let (alice_signer, _) = make_account_with_balance(alice_acc, 60).unwrap();

    // config variables can be updated only through committee
    assert_eq!(Mips::min_proposal_deposit(), 50);
    assert_err!(
        Mips::set_min_proposal_deposit(alice_signer.clone(), 10),
        Error::<TestStorage>::BadOrigin
    );
    assert_ok!(Mips::set_min_proposal_deposit(root.clone(), 10));
    assert_eq!(Mips::min_proposal_deposit(), 10);

    assert_eq!(Mips::quorum_threshold(), 70);
    assert_ok!(Mips::set_quorum_threshold(root.clone(), 100));
    assert_eq!(Mips::quorum_threshold(), 100);

    assert_eq!(Mips::proposal_duration(), 10);
    assert_ok!(Mips::set_proposal_duration(root.clone(), 100));
    assert_eq!(Mips::proposal_duration(), 100);
}

#[test]
fn amend_mips_details_during_cool_off_period() {
    ExtBuilder::default()
        .monied(true)
        .build()
        .execute_with(amend_mips_details_during_cool_off_period_we);
}

fn amend_mips_details_during_cool_off_period_we() {
    let proposal = make_proposal(42);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: MipDescription = b"Test description".into();

    let (alice, _) = make_account(AccountKeyring::Alice.public()).unwrap();
    let (bob, _) = make_account(AccountKeyring::Bob.public()).unwrap();

    // 1. Create Mips proposal
    assert_ok!(Mips::propose(
        alice.clone(),
        Box::new(proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc)
    ));
    fast_forward_to(50);

    // 2. Amend proposal during cool-off.
    let new_url: Url = b"www.xyz.com".into();
    let new_desc: MipDescription = b"New description".into();
    assert_ok!(Mips::amend_proposal(
        alice.clone(),
        0,
        Some(new_url.clone()),
        Some(new_desc.clone())
    ));

    assert_err!(
        Mips::amend_proposal(bob.clone(), 0, None, None),
        Error::<TestStorage>::BadOrigin
    );

    assert_eq!(
        Mips::proposal_meta(),
        vec![MipsMetadata {
            proposer: AccountKeyring::Alice.public(),
            index: 0,
            cool_off_until: 101,
            end: 111,
            url: Some(new_url),
            description: Some(new_desc)
        }]
    );

    // 3. Bound/Unbound additional POLYX.
    let alice_acc = AccountKeyring::Alice.public();
    assert_eq!(
        Mips::deposit_of(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 60
        }
    );
    assert_ok!(Mips::bond_additional_deposit(alice.clone(), 0, 100));
    assert_eq!(
        Mips::deposit_of(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 160
        }
    );
    assert_ok!(Mips::unbond_deposit(alice.clone(), 0, 50));
    assert_eq!(
        Mips::deposit_of(0, &alice_acc),
        DepositInfo {
            owner: alice_acc.clone(),
            amount: 110
        }
    );
    assert_err!(
        Mips::unbond_deposit(alice.clone(), 0, 90),
        Error::<TestStorage>::InsufficientDeposit
    );

    // 4. Move out the cool-off period and ensure Mips is inmutable.
    fast_forward_to(103);
    assert_err!(
        Mips::amend_proposal(alice.clone(), 0, None, None),
        Error::<TestStorage>::ProposalIsImmutable
    );
}

#[test]
fn cancel_mips_during_cool_off_period() {
    ExtBuilder::default()
        .build()
        .execute_with(cancel_mips_during_cool_off_period_we);
}

fn cancel_mips_during_cool_off_period_we() {
    let alice_proposal = make_proposal(42);
    let bob_proposal = make_proposal(1);
    let proposal_url: Url = b"www.abc.com".into();
    let proposal_desc: MipDescription = b"Test description".into();

    let (alice, _) = make_account(AccountKeyring::Alice.public()).unwrap();
    let (bob, _) = make_account(AccountKeyring::Bob.public()).unwrap();

    // 1. Create Mips proposals
    assert_ok!(Mips::propose(
        alice.clone(),
        Box::new(alice_proposal),
        60,
        Some(proposal_url),
        Some(proposal_desc)
    ));

    assert_ok!(Mips::propose(
        bob.clone(),
        Box::new(bob_proposal.clone()),
        60,
        None,
        None
    ));

    // 2. Cancel Alice's proposal during cool-off period.
    fast_forward_to(50);
    assert_ok!(Mips::cancel_proposal(alice.clone(), 0));

    // 3. Try to cancel Bob's proposal after cool-off period.
    fast_forward_to(101);
    assert_err!(
        Mips::cancel_proposal(bob.clone(), 1),
        Error::<TestStorage>::ProposalIsImmutable
    );

    // 4. Double check current proposals
    assert_eq!(
        Mips::proposal_meta(),
        vec![MipsMetadata {
            proposer: AccountKeyring::Bob.public(),
            index: 1,
            cool_off_until: 101,
            end: 111,
            url: None,
            description: None
        }]
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
    let root = Origin::system(frame_system::RawOrigin::Root);
    let alice = Origin::signed(AccountKeyring::Alice.public());
    let bob = Origin::signed(AccountKeyring::Bob.public());

    let proposal_a = make_proposal(42);
    let proposal_b = make_proposal(107);

    // Bob is the release coordinator.
    let bob_id = get_identity_id(AccountKeyring::Bob).expect("Bob is part of the committee");
    assert_ok!(Committee::set_release_coordinator(root.clone(), bob_id));

    // Alice submit 2 referendums in different moments.
    assert_ok!(Mips::submit_referendum(
        alice.clone(),
        Box::new(proposal_a.clone())
    ));
    assert_ok!(Mips::enact_referendum(root.clone(), 0));
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Scheduled,
            enactment_period: 101,
            proposal: proposal_a.clone()
        })
    );

    fast_forward_to(50);
    assert_ok!(Mips::submit_referendum(
        alice.clone(),
        Box::new(proposal_b.clone())
    ));
    assert_ok!(Mips::enact_referendum(root.clone(), 1));
    assert_eq!(
        Mips::referendums(1),
        Some(Referendum {
            index: 1,
            priority: MipsPriority::High,
            state: MipsState::Scheduled,
            enactment_period: 150,
            proposal: proposal_b.clone()
        })
    );

    // Alice cannot update the enact period.
    assert_err!(
        Mips::set_referendum_enactment_period(alice.clone(), 0, Some(200)),
        Error::<TestStorage>::BadOrigin
    );

    // Bob updates referendum to execute `b` now(next block), and `a` in the future.
    assert_ok!(Mips::set_referendum_enactment_period(bob.clone(), 1, None));
    fast_forward_to(52);
    assert_eq!(
        Mips::referendums(1),
        Some(Referendum {
            index: 1,
            priority: MipsPriority::High,
            state: MipsState::Executed,
            enactment_period: 51,
            proposal: proposal_b.clone()
        })
    );

    assert_ok!(Mips::set_referendum_enactment_period(
        bob.clone(),
        0,
        Some(200)
    ));
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Scheduled,
            enactment_period: 200,
            proposal: proposal_a.clone()
        })
    );

    // Bob cannot update if referendum is already executed.
    fast_forward_to(201);
    assert_eq!(
        Mips::referendums(0),
        Some(Referendum {
            index: 0,
            priority: MipsPriority::High,
            state: MipsState::Executed,
            enactment_period: 200,
            proposal: proposal_a.clone()
        })
    );
    assert_err!(
        Mips::set_referendum_enactment_period(bob.clone(), 0, Some(300)),
        Error::<TestStorage>::ReferendumIsImmutable
    );
}
