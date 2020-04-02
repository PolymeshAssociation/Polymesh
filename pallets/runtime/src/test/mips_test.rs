use crate::{
    asset,
    test::{
        storage::{make_account_with_balance, Call, TestStorage},
        ExtBuilder,
    },
};
use pallet_committee as committee;
use pallet_mips::{
    self as mips, Error, MipDescription, MipsPriority, PolymeshReferendumInfo, PolymeshVotes, Url,
};
use polymesh_primitives::Ticker;
use polymesh_runtime_balances as balances;
use polymesh_runtime_group as group;

use codec::Encode;
use frame_support::{assert_err, assert_ok};
use frame_system;
use sp_runtime::traits::{BlakeTwo256, Hash};
use test_client::AccountKeyring;

type System = frame_system::Module<TestStorage>;
type Asset = asset::Module<TestStorage>;
type Balances = balances::Module<TestStorage>;
type Mips = mips::Module<TestStorage>;
type Group = group::Module<TestStorage, group::Instance1>;
type Committee = committee::Module<TestStorage, committee::Instance1>;

type Origin = <TestStorage as frame_system::Trait>::Origin;

fn make_proposal(value: u64) -> Call {
    let ticker = Ticker::from(value.encode().as_slice());
    Call::Asset(asset::Call::register_ticker(ticker))
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
    let hash = BlakeTwo256::hash_of(&proposal);
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
        Mips::voting(&hash),
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
    let hash = BlakeTwo256::hash_of(&proposal);
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
        Mips::voting(&hash),
        Some(PolymeshVotes {
            index,
            ayes: vec![(alice_acc.clone(), 50)],
            nays: vec![],
        })
    );

    assert_ok!(Mips::kill_proposal(root, index, hash));
    assert_eq!(Balances::free_balance(&alice_acc), 218);
    assert_eq!(Mips::voting(&hash), None);
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
    let hash = BlakeTwo256::hash_of(&proposal);
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
        Mips::vote(bob_signer.clone(), hash, 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);
    assert_ok!(Mips::vote(bob_signer.clone(), hash, 0, true, 50));

    assert_eq!(
        Mips::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![(alice_acc.clone(), 50), (bob_acc.clone(), 50)],
            nays: vec![]
        })
    );

    assert_eq!(Balances::free_balance(&alice_acc), 168);
    assert_eq!(Balances::free_balance(&bob_acc), 109);

    fast_forward_to(120);

    assert_eq!(Mips::referendums(&hash), Some(proposal));

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
    let hash = BlakeTwo256::hash_of(&proposal);
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
        Mips::vote(bob_signer.clone(), hash, 0, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);

    assert_ok!(Mips::vote(bob_signer.clone(), hash, 0, true, 50));

    assert_eq!(
        Mips::voting(&hash),
        Some(PolymeshVotes {
            index: 0,
            ayes: vec![(alice_acc.clone(), 50), (bob_acc.clone(), 50)],
            nays: vec![]
        })
    );

    fast_forward_to(120);

    assert_eq!(Mips::referendums(&hash), Some(proposal));

    assert_err!(
        Mips::enact_referendum(bob_signer.clone(), hash),
        Error::<TestStorage>::BadOrigin
    );

    assert_ok!(Mips::enact_referendum(root, hash));
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
    let hash = BlakeTwo256::hash_of(&proposal);
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

    assert_eq!(Committee::members(), vec![alice_did, bob_did]);

    assert_ok!(Mips::propose(
        charlie_signer.clone(),
        Box::new(proposal.clone()),
        50,
        Some(proposal_url.clone()),
        Some(proposal_desc)
    ));

    assert_err!(
        Mips::vote(bob_signer.clone(), hash, index, true, 50),
        Error::<TestStorage>::ProposalOnCoolOffPeriod
    );
    fast_forward_to(101);
    assert_ok!(Mips::vote(bob_signer.clone(), hash, index, true, 50));

    // only a committee member can fast track a proposal
    assert_err!(
        Mips::fast_track_proposal(charlie_signer.clone(), index, hash),
        Error::<TestStorage>::NotACommitteeMember
    );

    // Alice can fast track because she is a GC member
    assert_ok!(Mips::fast_track_proposal(alice_signer.clone(), index, hash));

    fast_forward_to(120);

    assert_eq!(Mips::referendums(&hash), Some(proposal));

    assert_err!(
        Mips::enact_referendum(bob_signer.clone(), hash),
        Error::<TestStorage>::BadOrigin
    );

    assert_ok!(Mips::enact_referendum(root, hash));
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
    let index = 0;
    let hash = BlakeTwo256::hash_of(&proposal);

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

    assert_eq!(Mips::referendums(&hash), Some(proposal));

    assert_eq!(
        Mips::referendum_meta(),
        vec![PolymeshReferendumInfo {
            index,
            priority: MipsPriority::High,
            proposal_hash: hash
        }]
    );

    assert_err!(
        Mips::enact_referendum(alice_signer.clone(), hash),
        Error::<TestStorage>::BadOrigin
    );

    assert_ok!(Mips::enact_referendum(root, hash));
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
