use super::{
    assert_event_exists,
    asset_test::max_len_bytes,
    committee_test::{gc_vmo, set_members},
    storage::{
        fast_forward_blocks, make_remark_proposal, root, EventTest, RuntimeCall, TestStorage, User,
    },
    ExtBuilder,
};
use frame_support::{
    assert_noop, assert_ok,
    dispatch::{DispatchError, DispatchResult},
    traits::{LockableCurrency, WithdrawReasons},
    StorageDoubleMap, StorageValue,
};
use frame_system::{self, EventRecord};
use pallet_pips::{
    DepositInfo, LiveQueue, Pip, PipDescription, PipId, PipsMetadata, ProposalState, Proposer,
    RawEvent as Event, SnapshotId, SnapshotMetadata, SnapshotResult, SnapshottedPip, Vote,
    VoteCount, VotingResult,
};
use pallet_treasury as treasury;
use polymesh_common_utilities::{MaybeBlock, GC_DID};
use polymesh_primitives::{AccountId, BlockNumber, Url};
use std::ops::Deref;
use test_client::AccountKeyring;

type System = frame_system::Pallet<TestStorage>;
type Balances = pallet_balances::Module<TestStorage>;
type Pips = pallet_pips::Module<TestStorage>;
type Group = pallet_group::Module<TestStorage, pallet_group::Instance1>;
type Committee = pallet_committee::Module<TestStorage, pallet_committee::Instance1>;
type Treasury = treasury::Module<TestStorage>;
type Error = pallet_pips::Error<TestStorage>;
type Deposits = pallet_pips::Deposits<TestStorage>;
type Votes = pallet_pips::ProposalVotes<TestStorage>;
type Scheduler = pallet_scheduler::Pallet<TestStorage>;
type Agenda = pallet_scheduler::Agenda<TestStorage>;

type Origin = <TestStorage as frame_system::Config>::RuntimeOrigin;

macro_rules! assert_last_event {
    ($event:pat) => {
        assert_last_event!($event, true);
    };
    ($event:pat, $cond:expr) => {
        assert!(matches!(
            &*System::events(),
            [.., EventRecord {
                event: EventTest::Pips($event),
                ..
            }]
            if $cond
        ));
    };
}

macro_rules! assert_bad_origin {
    ($e:expr) => {
        assert_noop!($e, DispatchError::BadOrigin);
    };
}

macro_rules! assert_bad_state {
    ($e:expr) => {{
        assert_noop!($e, Error::IncorrectProposalState);
    }};
}

macro_rules! assert_no_pip {
    ($e:expr) => {{
        assert_noop!($e, Error::NoSuchProposal);
    }};
}

fn spip(id: u32, dir: bool, power: u128) -> SnapshottedPip {
    SnapshottedPip {
        id: PipId(id),
        weight: (dir, power),
    }
}

fn make_proposal(value: u64) -> RuntimeCall {
    RuntimeCall::Pips(pallet_pips::Call::set_min_proposal_deposit {
        deposit: value.into(),
    })
}

fn proposal(
    signer: &Origin,
    proposer: &Proposer<AccountId>,
    proposal: RuntimeCall,
    deposit: u128,
    url: Option<Url>,
    desc: Option<PipDescription>,
) -> DispatchResult {
    let before = Pips::pip_id_sequence();
    let active = Pips::active_pip_count();
    let signer = signer.clone();
    let result = Pips::propose(signer, Box::new(proposal), deposit, url, desc);
    let add = result.map_or(0, |_| 1);
    if let Ok(_) = result {
        assert_last_event!(Event::ProposalCreated(_, _, id, ..), *id == before);
        assert_eq!(
            Pips::committee_pips().contains(&before),
            matches!(proposer, Proposer::Committee(_))
        );
        assert_eq!(&Pips::proposals(before).unwrap().proposer, proposer);
    }
    assert_eq!(Pips::pip_id_sequence(), PipId(before.0 + add));
    assert_eq!(Pips::active_pip_count(), active + add);
    result
}

fn standard_proposal(
    signer: &Origin,
    proposer: &Proposer<AccountId>,
    deposit: u128,
) -> DispatchResult {
    proposal(signer, proposer, make_proposal(42), deposit, None, None)
}

fn remark_proposal(proposer: User, deposit: u128) -> DispatchResult {
    proposal(
        &proposer.origin(),
        &Proposer::Community(proposer.acc()),
        make_remark_proposal(),
        deposit,
        None,
        None,
    )
}

const THE_COMMITTEE: Proposer<AccountId> = Proposer::Committee(pallet_pips::Committee::Upgrade);

fn committee_proposal(deposit: u128) -> DispatchResult {
    standard_proposal(
        &pallet_committee::Origin::<TestStorage, pallet_committee::Instance4>::Endorsed(
            <_>::default(),
        )
        .into(),
        &THE_COMMITTEE,
        deposit,
    )
}

fn community_proposal(user: User, deposit: u128) -> DispatchResult {
    standard_proposal(&user.origin(), &Proposer::Community(user.acc()), deposit)
}

fn consensus_call(call: pallet_pips::Call<TestStorage>, signers: &[&Origin]) {
    let call = Box::new(RuntimeCall::Pips(call));
    for signer in signers.iter().copied().cloned() {
        assert_ok!(Committee::vote_or_propose(signer, true, call.clone()));
    }
}

fn assert_state(id: PipId, care_about_pruned: bool, state: ProposalState) {
    let prop = Pips::proposals(id);
    if care_about_pruned && Pips::prune_historical_pips() {
        assert_eq!(prop, None);
    } else {
        assert_eq!(Pips::proposal_state(id).unwrap(), state);
    }
}

pub fn assert_balance(acc: AccountId, free: u128, locked: u128) {
    assert_eq!(Balances::free_balance(&acc), free);
    assert_eq!(Balances::usable_balance(&acc), free - locked);
}

#[test]
fn updating_pips_variables_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_eq!(Pips::prune_historical_pips(), false);
        assert_ok!(Pips::set_prune_historical_pips(root(), true));
        assert_last_event!(Event::HistoricalPipsPruned(_, false, true));
        assert_eq!(Pips::prune_historical_pips(), true);

        assert_eq!(Pips::min_proposal_deposit(), 50);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 10));
        assert_last_event!(Event::MinimumProposalDepositChanged(_, 50, 10));
        assert_eq!(Pips::min_proposal_deposit(), 10);

        assert_eq!(Pips::default_enactment_period(), 100);
        assert_ok!(Pips::set_default_enactment_period(root(), 10));
        assert_last_event!(Event::DefaultEnactmentPeriodChanged(_, 100, 10));
        assert_eq!(Pips::default_enactment_period(), 10);

        assert_eq!(Pips::pending_pip_expiry(), MaybeBlock::None);
        assert_ok!(Pips::set_pending_pip_expiry(root(), MaybeBlock::Some(13)));
        assert_last_event!(Event::PendingPipExpiryChanged(
            _,
            MaybeBlock::None,
            MaybeBlock::Some(13)
        ));
        assert_eq!(Pips::pending_pip_expiry(), MaybeBlock::Some(13));

        assert_eq!(Pips::max_pip_skip_count(), 1);
        assert_ok!(Pips::set_max_pip_skip_count(root(), 42));
        assert_last_event!(Event::MaxPipSkipCountChanged(_, 1, 42));
        assert_eq!(Pips::max_pip_skip_count(), 42);

        assert_eq!(Pips::active_pip_limit(), 5);
        assert_ok!(Pips::set_active_pip_limit(root(), 42));
        assert_last_event!(Event::ActivePipLimitChanged(_, 5, 42));
        assert_eq!(Pips::active_pip_limit(), 42);
    });
}

#[test]
fn updating_pips_variables_only_root() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let signer = Origin::signed(AccountKeyring::Alice.to_account_id());
        System::reset_events();

        assert_noop!(
            Pips::set_prune_historical_pips(signer.clone(), false),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            Pips::set_min_proposal_deposit(signer.clone(), 0),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            Pips::set_default_enactment_period(signer.clone(), 0),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            Pips::set_max_pip_skip_count(signer.clone(), 0),
            DispatchError::BadOrigin,
        );
        assert_noop!(
            Pips::set_active_pip_limit(signer.clone(), 0),
            DispatchError::BadOrigin,
        );

        assert_eq!(System::events(), vec![])
    });
}

#[test]
fn historical_prune_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        // We just test one case for brevity.
        System::set_block_number(1);
        assert_ok!(Pips::set_prune_historical_pips(root(), true));
        let proposer = User::new(AccountKeyring::Alice);
        let member = User::new(AccountKeyring::Bob);

        assert_pruned(rejected_proposal(proposer));
        set_members(vec![member.did]);
        assert_pruned(executed_community_proposal(proposer, member));
        assert_pruned(failed_community_proposal(proposer, member, PipId(1337)));
        assert_pruned(expired_proposal(proposer, 13));
    });
}

#[test]
fn min_deposit_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let deposit = 40;
        assert_ok!(Pips::set_min_proposal_deposit(root(), deposit + 1));

        let alice = User::new(AccountKeyring::Alice).balance(300);

        // Error when min deposit requirements are not met.
        assert_eq!(Pips::pip_id_sequence(), PipId(0));
        assert_eq!(Pips::active_pip_count(), 0);
        assert_noop!(community_proposal(alice, deposit), Error::IncorrectDeposit);

        // Now let's use enough.
        assert_ok!(community_proposal(alice, deposit + 1));
        assert_state(PipId(0), false, ProposalState::Pending);
        assert_eq!(
            Pips::proposals(PipId(0)).unwrap().proposer,
            Proposer::Community(alice.acc())
        );

        // Committees are exempt from min deposit.
        assert_ok!(committee_proposal(0));
        assert_state(PipId(1), false, ProposalState::Pending);
        assert_eq!(Pips::proposals(PipId(1)).unwrap().proposer, THE_COMMITTEE);
        assert_vote_details(PipId(1), VotingResult::default(), vec![], vec![]);
    })
}

#[test]
fn active_limit_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);

        assert_eq!(Pips::pip_id_sequence(), PipId(0));
        assert_eq!(Pips::active_pip_count(), 0);

        assert_ok!(community_proposal(proposer, 0));
        assert_eq!(Pips::active_pip_count(), 1);

        // Limit reached, so error.
        assert_ok!(Pips::set_active_pip_limit(root(), 1));
        assert_noop!(community_proposal(proposer, 0), Error::TooManyActivePips);
        assert_eq!(Pips::active_pip_count(), 1);

        // Bump limit; ok again.
        assert_ok!(Pips::set_active_pip_limit(root(), 2));
        assert_ok!(community_proposal(proposer, 0));
        assert_eq!(Pips::active_pip_count(), 2);

        // Reached again, so error.
        assert_noop!(community_proposal(proposer, 0), Error::TooManyActivePips);
        assert_eq!(Pips::active_pip_count(), 2);

        // Committees are exempt from limit.
        assert_ok!(committee_proposal(0));
        assert_eq!(Pips::active_pip_count(), 3);

        // Remove limit completely, and let's add more.
        assert_ok!(Pips::set_active_pip_limit(root(), 0));
        assert_ok!(community_proposal(proposer, 0));
        assert_eq!(Pips::active_pip_count(), 4);
    })
}

#[test]
fn default_enactment_period_works_community() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = User::new(AccountKeyring::Alice).balance(300);
        set_members(vec![alice.did]);

        let check_community = |period| {
            assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
            assert_ok!(community_proposal(alice, 0));
            let last_id = PipId(Pips::pip_id_sequence().0 - 1);
            fast_forward_blocks(1);
            assert_ok!(Pips::snapshot(alice.origin()));
            assert_ok!(Pips::set_default_enactment_period(root(), period));
            let block_at_approval = System::block_number();
            assert_ok!(Pips::enact_snapshot_results(
                gc_vmo(),
                vec![(last_id, SnapshotResult::Approve)]
            ));
            let expected = Pips::pip_to_schedule(last_id).unwrap();
            let period = period.max(1);
            assert_eq!(expected, block_at_approval + period);
            assert_eq!(1, Agenda::get(expected).len());
        };
        check_community(0);
        check_community(3);
        check_community(42);
        check_community(1337);
    });
}

#[test]
fn default_enactment_period_works_committee() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = User::new(AccountKeyring::Alice).balance(300);
        set_members(vec![alice.did]);

        let check_committee = |period| {
            assert_ok!(committee_proposal(0));
            let last_id = PipId(Pips::pip_id_sequence().0 - 1);
            fast_forward_blocks(1);
            assert_ok!(Pips::set_default_enactment_period(root(), period));
            let block_at_approval = System::block_number();
            assert_ok!(Pips::approve_committee_proposal(gc_vmo(), last_id));
            let expected = Pips::pip_to_schedule(last_id).unwrap();
            let period = period.max(1);
            assert_eq!(expected, block_at_approval + period);
            assert_eq!(1, Agenda::get(expected).len());
        };
        check_committee(0);
        check_committee(3);
        check_committee(42);
        check_committee(1337);
    })
}

#[test]
fn skip_limit_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let alice = User::new(AccountKeyring::Alice).balance(300);
        set_members(vec![alice.did]);

        let snap = || Pips::snapshot(alice.origin()).unwrap();
        let count = |n| Pips::set_max_pip_skip_count(root(), n).unwrap();
        let commit =
            || Pips::enact_snapshot_results(gc_vmo(), vec![(PipId(0), SnapshotResult::Skip)]);

        assert_ok!(community_proposal(alice, 0));

        snap();
        count(0);
        assert_noop!(commit(), Error::CannotSkipPip);

        snap();
        count(1);
        assert_ok!(commit());
        snap();
        assert_noop!(commit(), Error::CannotSkipPip);

        snap();
        count(2);
        assert_ok!(commit());
        snap();
        assert_noop!(commit(), Error::CannotSkipPip);
    });
}

fn assert_vote_details(
    id: PipId,
    results: VotingResult,
    deposits: Vec<DepositInfo<AccountId>>,
    votes: Vec<Vote>,
) {
    assert_eq!(results, Pips::proposal_result(id));
    assert_eq!(
        deposits,
        Deposits::iter_prefix_values(id).collect::<Vec<_>>(),
    );
    assert_eq!(votes, Votes::iter_prefix_values(id).collect::<Vec<_>>());
}

fn assert_votes(id: PipId, owner: AccountId, amount: u128) {
    assert_vote_details(
        id,
        VotingResult {
            ayes_count: 1,
            ayes_stake: amount,
            nays_count: 0,
            nays_stake: 0,
        },
        vec![DepositInfo { owner, amount }],
        vec![Vote(true, amount)],
    );
}

#[test]
fn proposal_details_are_correct() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(42);

        let alice = User::new(AccountKeyring::Alice).balance(300);

        let call = make_remark_proposal();
        let proposal_url: Url = b"www.abc.com".into();
        let proposal_desc: PipDescription = b"Test description".into();

        let proposer = Proposer::Community(alice.acc());

        // Alice starts a proposal with min deposit.
        assert_ok!(proposal(
            &alice.origin(),
            &proposer,
            call.clone(),
            60,
            Some(proposal_url.clone()),
            Some(proposal_desc.clone()),
        ));
        assert_last_event!(Event::ProposalCreated(..));

        let expected = Pip {
            id: PipId(0),
            proposal: call,
            proposer,
        };
        assert_eq!(Pips::proposals(PipId(0)).unwrap(), expected);
        assert_eq!(
            Pips::proposal_state(PipId(0)).unwrap(),
            ProposalState::Pending
        );

        let expected = PipsMetadata {
            id: PipId(0),
            created_at: 42,
            url: Some(proposal_url),
            description: Some(proposal_desc),
            transaction_version: 7,
            expiry: <_>::default(),
        };
        assert_eq!(Pips::proposal_metadata(PipId(0)).unwrap(), expected);

        assert_balance(alice.acc(), 300, 60);
        assert_votes(PipId(0), alice.acc(), 60);
    });
}

#[test]
fn proposal_limits_are_enforced() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(42);
        let proposer = User::new(AccountKeyring::Alice).balance(300);
        let propose = |url, desc| {
            proposal(
                &proposer.origin(),
                &Proposer::Community(proposer.acc()),
                make_remark_proposal(),
                60,
                Some(url),
                Some(desc),
            )
        };
        assert_too_long!(propose(max_len_bytes(1), max_len_bytes(0)));
        assert_too_long!(propose(max_len_bytes(0), max_len_bytes(1)));
        assert_ok!(propose(max_len_bytes(0), max_len_bytes(0)));
    });
}

#[test]
fn propose_committee_pip_only_zero_deposit() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(committee_proposal(0));
        assert_noop!(committee_proposal(1337), Error::NotFromCommunity);
    });
}

#[test]
fn vote_no_such_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let voter = User::new(AccountKeyring::Bob);
        assert_no_pip!(Pips::vote(voter.origin(), PipId(0), false, 0));
    });
}

#[test]
fn vote_not_pending() {
    let op_and_check = |op_and_check: &dyn Fn(Origin, PipId)| {
        ExtBuilder::default().build().execute_with(|| {
            System::set_block_number(1);
            assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
            assert_ok!(Pips::set_prune_historical_pips(root(), false));

            let proposer = User::new(AccountKeyring::Alice);
            let member = User::new(AccountKeyring::Bob);
            set_members(vec![member.did]);

            op_and_check(member.origin(), rejected_proposal(proposer));
            op_and_check(
                member.origin(),
                executed_community_proposal(proposer, member),
            );
            op_and_check(
                member.origin(),
                failed_community_proposal(proposer, member, PipId(1337)),
            );
            op_and_check(member.origin(), scheduled_proposal(proposer, member, 0));
            op_and_check(member.origin(), expired_proposal(proposer, 24));
        })
    };
    op_and_check(&|o, id| assert_bad_state!(Pips::vote(o, id, false, 0)));
}

#[test]
fn vote_bond_additional_deposit_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);
        let init_free = Balances::free_balance(&proposer.acc());
        let init_amount = 300;
        let then_amount = 137;
        let amount = init_amount + then_amount;

        assert_balance(proposer.acc(), init_free, 0);

        assert_ok!(community_proposal(proposer, init_amount));
        assert_balance(proposer.acc(), init_free, init_amount);
        assert_ok!(Pips::vote(proposer.origin(), PipId(0), true, amount));
        assert_balance(proposer.acc(), init_free, amount);
        assert_last_event!(Event::Voted(.., true, _));
        assert_votes(PipId(0), proposer.acc(), amount);
    });
}

#[test]
fn vote_owner_below_min_deposit() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        let min = 50;
        let sub = min - 1;
        assert_ok!(Pips::set_min_proposal_deposit(root(), min));

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);

        assert_ok!(community_proposal(alice, 100));
        assert_noop!(
            Pips::vote(alice.origin(), PipId(0), true, sub),
            Error::IncorrectDeposit
        );
        assert_votes(PipId(0), alice.acc(), 100);
        // Doesn't apply to Bob though, as they didn't propose it.
        assert_ok!(Pips::vote(bob.origin(), PipId(0), true, sub));
        assert_vote_details(
            PipId(0),
            VotingResult {
                ayes_count: 2,
                ayes_stake: 100 + sub,
                ..VotingResult::default()
            },
            vec![
                DepositInfo {
                    owner: alice.acc(),
                    amount: 100,
                },
                DepositInfo {
                    owner: bob.acc(),
                    amount: sub,
                },
            ],
            vec![Vote(true, 100), Vote(true, sub)],
        );
    });
}

#[test]
fn vote_unbond_deposit_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);
        let init_free = Balances::free_balance(&proposer.acc());
        let init_amount = 200;
        let then_amount = 100;

        assert_eq!(Balances::free_balance(&proposer.acc()), init_free);

        assert_ok!(community_proposal(proposer, init_amount));
        assert_balance(proposer.acc(), init_free, init_amount);
        assert_ok!(Pips::vote(proposer.origin(), PipId(0), true, then_amount));
        assert_balance(proposer.acc(), init_free, then_amount);
        assert_last_event!(Event::Voted(.., true, _));
        assert_votes(PipId(0), proposer.acc(), then_amount);
    });
}

#[test]
fn vote_on_community_only() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(committee_proposal(0));
        let voter = User::new(AccountKeyring::Alice);
        assert_noop!(
            Pips::vote(voter.origin(), PipId(0), false, 0),
            Error::NotFromCommunity
        );
    });
}

#[test]
fn vote_duplicate_ok() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let proposer = User::new(AccountKeyring::Alice);

        let vote = |aye, power| Pips::vote(proposer.origin(), PipId(0), aye, power);
        let res = || Pips::proposal_result(PipId(0));

        assert_ok!(community_proposal(proposer, 42));
        assert_eq!(
            res(),
            VotingResult {
                ayes_count: 1,
                ayes_stake: 42,
                ..VotingResult::default()
            }
        );
        assert_ok!(vote(true, 21));
        assert_eq!(
            res(),
            VotingResult {
                ayes_count: 1,
                ayes_stake: 21,
                ..VotingResult::default()
            }
        );
        assert_ok!(vote(false, 21));
        assert_eq!(
            res(),
            VotingResult {
                nays_count: 1,
                nays_stake: 21,
                ..VotingResult::default()
            }
        );
        assert_ok!(vote(false, 42));
        assert_eq!(
            res(),
            VotingResult {
                nays_count: 1,
                nays_stake: 42,
                ..VotingResult::default()
            }
        );
        assert_ok!(vote(true, 42));
        assert_eq!(
            res(),
            VotingResult {
                ayes_count: 1,
                ayes_stake: 42,
                ..VotingResult::default()
            }
        );
    });
}

#[test]
fn vote_stake_overflow() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = User::new(AccountKeyring::Alice).balance(u128::MAX);
        let bob = User::new(AccountKeyring::Bob).balance(100);
        let id = PipId(0);

        assert_ok!(community_proposal(alice, u128::MAX));
        assert_eq!(
            Pips::proposal_result(id),
            VotingResult {
                ayes_count: 1,
                ayes_stake: u128::MAX,
                ..VotingResult::default()
            }
        );
        assert_noop!(
            Pips::vote(bob.origin(), id, true, 1),
            Error::StakeAmountOfVotesExceeded,
        );
        assert_ok!(Pips::vote(bob.origin(), id, false, 1));
        assert_noop!(
            Pips::vote(alice.origin(), id, false, u128::MAX),
            Error::StakeAmountOfVotesExceeded,
        );
    });
}

#[test]
fn vote_insufficient_reserve() {
    ExtBuilder::default()
        .monied(false)
        .build()
        .execute_with(|| {
            System::set_block_number(1);
            assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
            let proposer = User::new(AccountKeyring::Alice).balance(0);
            let id = PipId(0);
            assert_ok!(community_proposal(proposer, 0));
            assert_noop!(
                Pips::vote(proposer.origin(), id, false, 50),
                Error::InsufficientDeposit
            );
            assert_noop!(
                Pips::vote(proposer.origin(), id, true, 1),
                Error::InsufficientDeposit
            );
        });
}

#[test]
fn vote_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let bob_balance = Balances::free_balance(&bob.acc());
        let charlie = User::new(AccountKeyring::Charlie);
        let charlie_balance = Balances::free_balance(&charlie.acc());

        let id = PipId(0);
        assert_ok!(community_proposal(alice, 100));
        assert_balance(bob.acc(), bob_balance, 0);
        assert_balance(charlie.acc(), charlie_balance, 0);
        assert_ok!(Pips::vote(bob.origin(), id, false, 1337));
        assert_last_event!(Event::Voted(.., false, 1337));
        assert_ok!(Pips::vote(charlie.origin(), id, true, 2441));
        assert_last_event!(Event::Voted(.., true, 2441));
        assert_balance(bob.acc(), bob_balance, 1337);
        assert_balance(charlie.acc(), charlie_balance, 2441);
        assert_vote_details(
            id,
            VotingResult {
                ayes_count: 2,
                ayes_stake: 2541,
                nays_count: 1,
                nays_stake: 1337,
            },
            vec![
                DepositInfo {
                    owner: alice.acc(),
                    amount: 100,
                },
                DepositInfo {
                    owner: bob.acc(),
                    amount: 1337,
                },
                DepositInfo {
                    owner: charlie.acc(),
                    amount: 2441,
                },
            ],
            vec![Vote(true, 100), Vote(false, 1337), Vote(true, 2441)],
        );
    });
}

#[test]
fn voting_for_pip_uses_stack_over_overlay() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        // Initialize with 100 POLYX.
        let alice = User::new(AccountKeyring::Alice).balance(100);
        // Lock all but 10.
        Balances::set_lock(*b"deadbeef", &alice.acc(), 90, WithdrawReasons::all());
        assert_balance(alice.acc(), 100, 90);
        // OK, because we're overlaying with 90 tokens already locked.
        assert_ok!(community_proposal(alice, 50));
        assert_balance(alice.acc(), 100, 90);
        // OK, because we're still overlaying, but also increasing it by 10.
        assert_ok!(community_proposal(alice, 50));
        assert_balance(alice.acc(), 100, 100);
        // Error, because we don't have 101 tokens to bond.
        assert_noop!(community_proposal(alice, 1), Error::InsufficientDeposit);
    });
}

#[test]
fn approve_committee_proposal_not_pending() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), false));

        let proposer = User::new(AccountKeyring::Bob);
        let member = User::new(AccountKeyring::Alice);
        set_members(vec![member.did]);

        let acp_bad_state = |id| assert_bad_state!(Pips::approve_committee_proposal(gc_vmo(), id));
        acp_bad_state(rejected_proposal(proposer));
        acp_bad_state(executed_community_proposal(proposer, member));
        acp_bad_state(failed_community_proposal(proposer, member, PipId(1337)));
        acp_bad_state(scheduled_proposal(proposer, member, 0));
        acp_bad_state(expired_proposal(proposer, 9));
    });
}

#[test]
fn approve_committee_proposal_no_such_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_no_pip!(Pips::approve_committee_proposal(gc_vmo(), PipId(0)));
    });
}

#[test]
fn approve_committee_proposal_not_by_committee() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let proposer = User::new(AccountKeyring::Bob);
        assert_ok!(community_proposal(proposer, 0));
        assert_noop!(
            Pips::approve_committee_proposal(gc_vmo(), PipId(0)),
            Error::NotByCommittee,
        );
    });
}

#[test]
fn only_gc_majority_stuff() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Dave);

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        set_members(vec![bob.did, charlie.did]);

        // Make a proposal
        let id = Pips::pip_id_sequence();
        assert_eq!(id, PipId(0));
        assert_eq!(Pips::active_pip_count(), 0);
        assert_ok!(community_proposal(proposer, 0));
        // Alice not part of GC and cannot reject.
        assert_bad_origin!(Pips::reject_proposal(alice.origin(), id));
        // Bob & Charlie but need to seek consensus.
        assert_bad_origin!(Pips::reject_proposal(bob.origin(), id));
        assert_bad_origin!(Pips::reject_proposal(charlie.origin(), id));
        // Ditto for pruning.
        assert_bad_origin!(Pips::prune_proposal(alice.origin(), id));
        // Bob & Charlie but need to seek consensus.
        assert_bad_origin!(Pips::prune_proposal(bob.origin(), id));
        assert_bad_origin!(Pips::prune_proposal(charlie.origin(), id));
        // Ditto for `approve_committee_proposal`.
        assert_bad_origin!(Pips::approve_committee_proposal(alice.origin(), id));
        // Bob & Charlie but need to seek consensus.
        assert_bad_origin!(Pips::approve_committee_proposal(bob.origin(), id));
        assert_bad_origin!(Pips::approve_committee_proposal(charlie.origin(), id));
        // Ditto for `enact_snapshot_result`.
        assert_bad_origin!(Pips::enact_snapshot_results(alice.origin(), vec![]));
        // Bob & Charlie but need to seek consensus.
        assert_bad_origin!(Pips::enact_snapshot_results(bob.origin(), vec![]));
        assert_bad_origin!(Pips::enact_snapshot_results(charlie.origin(), vec![]));

        // VMO can reject.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_eq!(Pips::pip_id_sequence(), PipId(1));
        assert_eq!(Pips::active_pip_count(), 0);
        assert_state(id, false, ProposalState::Rejected);
        // VMO can also prune.
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        assert_eq!(Pips::proposals(id), None);
        // VMO can also `approve_committee_proposal`.
        let id = Pips::pip_id_sequence();
        assert_ok!(committee_proposal(0));
        assert_ok!(Pips::approve_committee_proposal(gc_vmo(), id));
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        // VMO can also `enact_snapshot_results`.
        let id = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(Pips::snapshot(bob.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(id, SnapshotResult::Reject)]
        ));

        let consensus_call = |call| consensus_call(call, &[&bob.origin(), &charlie.origin()]);

        // Bob & Charlie seek consensus and successfully reject.
        let id = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 0));
        assert_eq!(Pips::pip_id_sequence().0, id.0 + 1);
        assert_eq!(Pips::active_pip_count(), 1);
        consensus_call(pallet_pips::Call::reject_proposal { id });
        assert_eq!(Pips::pip_id_sequence().0, id.0 + 1);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_state(id, false, ProposalState::Rejected);
        // And now they seek consensus to and do prune.
        consensus_call(pallet_pips::Call::prune_proposal { id });
        assert_eq!(Pips::proposals(id), None);

        // Bob & Charlie seek consensus.
        // They successfully do `approve_committee_proposal` & `enact_snapshot_results`.
        let id_committee = Pips::pip_id_sequence();
        assert_ok!(committee_proposal(0));
        let id_snapshot = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(Pips::snapshot(bob.origin()));
        consensus_call(pallet_pips::Call::approve_committee_proposal { id: id_committee });
        consensus_call(pallet_pips::Call::enact_snapshot_results {
            results: vec![(id_snapshot, SnapshotResult::Approve)],
        });
        assert_state(id_committee, false, ProposalState::Scheduled);
        assert_state(id_snapshot, false, ProposalState::Scheduled);
    });
}

#[test]
fn cannot_reject_no_such_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        // Rejecting PIP that doesn't exist errors.
        let id = PipId(0);
        assert_eq!(Pips::pip_id_sequence(), id);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_no_pip!(Pips::reject_proposal(gc_vmo(), id));
        assert_eq!(Pips::pip_id_sequence(), id);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_no_pip!(Pips::prune_proposal(gc_vmo(), id));
        assert_eq!(Pips::pip_id_sequence(), id);
        assert_eq!(Pips::active_pip_count(), 0);
    });
}

fn scheduled_proposal(proposer: User, member: User, deposit: u128) -> PipId {
    let next_id = Pips::pip_id_sequence();
    assert_ok!(community_proposal(proposer, deposit));
    let active = Pips::active_pip_count();
    assert_ok!(Pips::snapshot(member.origin()));
    assert_ok!(Pips::enact_snapshot_results(
        gc_vmo(),
        vec![(next_id, SnapshotResult::Approve)]
    ));
    assert_event_exists!(
        EventTest::Scheduler(pallet_scheduler::Event::Scheduled { when, .. }),
        *when == System::block_number() + Pips::default_enactment_period()
    );
    assert_state(next_id, false, ProposalState::Scheduled);
    assert_eq!(Pips::active_pip_count(), active);
    next_id
}

fn executed_community_proposal(proposer: User, member: User) -> PipId {
    let deposit = Pips::min_proposal_deposit();
    let next_id = scheduled_proposal(proposer, member, deposit);
    let active = Pips::active_pip_count();
    fast_forward_blocks(Pips::default_enactment_period() + 1);
    assert_ok!(Pips::set_min_proposal_deposit(root(), deposit));
    assert_state(next_id, true, ProposalState::Executed);
    assert_eq!(Pips::active_pip_count(), active - 1);
    next_id
}

fn failed_community_proposal(proposer: User, member: User, bad_id: PipId) -> PipId {
    let next_id = Pips::pip_id_sequence();
    let deposit = Pips::min_proposal_deposit();
    assert_ok!(proposal(
        &proposer.origin(),
        &Proposer::Community(proposer.acc()),
        RuntimeCall::Pips(pallet_pips::Call::reject_proposal { id: bad_id }),
        deposit,
        None,
        None
    ));
    let active = Pips::active_pip_count();
    assert_ok!(Pips::snapshot(member.origin()));
    assert_ok!(Pips::enact_snapshot_results(
        gc_vmo(),
        vec![(next_id, SnapshotResult::Approve)]
    ));
    assert_state(next_id, false, ProposalState::Scheduled);
    assert_eq!(Pips::active_pip_count(), active);
    fast_forward_blocks(Pips::default_enactment_period() + 1);
    assert_state(next_id, true, ProposalState::Failed);
    assert_eq!(Pips::active_pip_count(), active - 1);
    next_id
}

fn rejected_proposal(proposer: User) -> PipId {
    let next_id = Pips::pip_id_sequence();
    assert_ok!(community_proposal(proposer, Pips::min_proposal_deposit()));
    let active = Pips::active_pip_count();
    assert_ok!(Pips::reject_proposal(gc_vmo(), next_id));
    assert_state(next_id, true, ProposalState::Rejected);
    assert_eq!(Pips::active_pip_count(), active - 1);
    assert_eq!(Pips::pip_id_sequence().0, next_id.0 + 1);
    next_id
}

fn expired_proposal(proposer: User, expiry: BlockNumber) -> PipId {
    let next_id = Pips::pip_id_sequence();

    // Save old config data and set new ones for expiry.
    let old_expiry = Pips::pending_pip_expiry();
    assert_ok!(Pips::set_pending_pip_expiry(
        root(),
        MaybeBlock::Some(expiry)
    ));

    // Create a proposal and verify its pending.
    let active = Pips::active_pip_count();
    assert_ok!(community_proposal(proposer, Pips::min_proposal_deposit()));
    assert_state(next_id, false, ProposalState::Pending);
    assert_eq!(
        Pips::proposal_metadata(next_id).unwrap().expiry,
        MaybeBlock::Some(expiry + System::block_number())
    );

    // Now fast forward.
    fast_forward_blocks(expiry + 1); // Forward exactly to expiry point + 1.
    assert_eq!(Pips::active_pip_count(), active);

    // Restore config to before function was called.
    assert_ok!(Pips::set_pending_pip_expiry(root(), old_expiry));

    next_id
}

#[test]
fn cannot_reject_incorrect_state() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), false));

        let proposer = User::new(AccountKeyring::Alice);
        let member = User::new(AccountKeyring::Bob);
        set_members(vec![member.did]);

        let reject_bad_state = |id| assert_bad_state!(Pips::reject_proposal(gc_vmo(), id));
        // Cannot reject executed, failed, and rejected:
        let exec_id = executed_community_proposal(proposer, member);
        reject_bad_state(exec_id);
        reject_bad_state(failed_community_proposal(proposer, member, exec_id));
        reject_bad_state(rejected_proposal(proposer));
        reject_bad_state(expired_proposal(proposer, 23));
    });
}

fn assert_pruned(id: PipId) {
    assert_eq!(Pips::proposal_metadata(id), None);
    assert_eq!(Deposits::iter_prefix_values(id).count(), 0);
    assert_eq!(Pips::proposals(id), None);
    assert_vote_details(id, VotingResult::default(), vec![], vec![]);
    assert_eq!(Pips::pip_to_schedule(id), None);
    // TODO: Check that the PIP has been removed from the schedule. This should be easily done after
    // fixing this issue: https://github.com/PolymeshAssociation/substrate/issues/7449
    assert!(Pips::snapshot_queue().iter().all(|p| p.id != id));
    assert_eq!(Pips::pip_skip_count(id), 0);
}

#[test]
fn can_prune_states_that_cannot_be_rejected() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_prune_historical_pips(root(), false));

        let init_bal = 1000;
        let proposer = User::new(AccountKeyring::Alice).balance(init_bal);
        let member = User::new(AccountKeyring::Bob);
        set_members(vec![member.did]);

        // Can prune executed:
        assert_balance(proposer.acc(), init_bal, 0);
        assert_eq!(Balances::free_balance(&proposer.acc()), init_bal);
        let id = PipId(0);
        assert_ok!(community_proposal(proposer, 200));
        assert_balance(proposer.acc(), init_bal, 200);
        assert_eq!(Pips::pip_id_sequence(), PipId(1));
        assert_eq!(Pips::active_pip_count(), 1);
        assert_ok!(Pips::snapshot(member.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(id, SnapshotResult::Approve)]
        ));
        assert_state(id, false, ProposalState::Scheduled);
        assert_balance(proposer.acc(), init_bal, 200);
        assert_eq!(Pips::active_pip_count(), 1);
        fast_forward_blocks(Pips::default_enactment_period() + 1);
        assert_state(id, false, ProposalState::Executed);
        assert_balance(proposer.acc(), init_bal, 0);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        assert_balance(proposer.acc(), init_bal, 0);
        assert_pruned(id);

        // Can prune failed:
        let id = PipId(1);
        assert_ok!(proposal(
            &proposer.origin(),
            &Proposer::Community(proposer.acc()),
            RuntimeCall::Pips(pallet_pips::Call::reject_proposal { id: PipId(1337) }),
            300,
            None,
            None
        ));
        assert_balance(proposer.acc(), init_bal, 300);
        assert_eq!(Pips::pip_id_sequence(), PipId(2));
        assert_eq!(Pips::active_pip_count(), 1);
        assert_ok!(Pips::snapshot(member.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(id, SnapshotResult::Approve)]
        ));
        assert_state(id, false, ProposalState::Scheduled);
        assert_balance(proposer.acc(), init_bal, 300);
        assert_eq!(Pips::active_pip_count(), 1);
        fast_forward_blocks(Pips::default_enactment_period() + 1);
        assert_state(id, false, ProposalState::Failed);
        assert_balance(proposer.acc(), init_bal, 0);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        assert_balance(proposer.acc(), init_bal, 0);
        assert_pruned(id);

        // Can prune rejected:
        let id = PipId(2);
        assert_ok!(community_proposal(proposer, 400));
        assert_balance(proposer.acc(), init_bal, 400);
        assert_eq!(Pips::pip_id_sequence(), PipId(3));
        assert_eq!(Pips::active_pip_count(), 1);
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_balance(proposer.acc(), init_bal, 0);
        assert_state(id, false, ProposalState::Rejected);
        assert_eq!(Pips::active_pip_count(), 0);
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        assert_balance(proposer.acc(), init_bal, 0);
        assert_pruned(id);
    });
}

#[test]
fn cannot_prune_active() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);

        let init_bal = 300;
        let proposer = User::new(AccountKeyring::Bob).balance(init_bal);
        let member = User::new(AccountKeyring::Alice);
        set_members(vec![member.did]);

        // Alice starts a proposal with some deposit.
        let id = PipId(0);
        assert_ok!(community_proposal(proposer, 50));
        assert_balance(proposer.acc(), init_bal, 50);
        // Now remove that PIP and check that funds are back.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_state(id, false, ProposalState::Pending);
        assert_bad_state!(Pips::prune_proposal(gc_vmo(), id));
        let id = PipId(1);
        assert_eq!(Pips::pip_id_sequence(), id);
        assert_eq!(Pips::active_pip_count(), 1);
        assert_balance(proposer.acc(), init_bal, 50);

        // Alice starts a proposal with some deposit.
        assert_ok!(community_proposal(proposer, 60));
        assert_balance(proposer.acc(), init_bal, 50 + 60);
        // Schedule the PIP.
        assert_ok!(Pips::snapshot(member.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(id, SnapshotResult::Approve)]
        ));
        assert_state(id, false, ProposalState::Scheduled);
        // Now remove that PIP and check that funds are back.
        assert_bad_state!(Pips::prune_proposal(gc_vmo(), id));
        assert_eq!(Pips::pip_id_sequence(), PipId(2));
        assert_eq!(Pips::active_pip_count(), 2);
        assert_balance(proposer.acc(), init_bal, 50 + 60);
    });
}

#[test]
fn reject_proposal_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);

        let init_bal = 300;
        let proposer = User::new(AccountKeyring::Bob).balance(init_bal);
        let member = User::new(AccountKeyring::Alice);
        set_members(vec![member.did]);

        // Alice starts a proposal with some deposit.
        let id = PipId(0);
        assert_ok!(community_proposal(proposer, 50));
        assert_balance(proposer.acc(), init_bal, 50);
        let result = VotingResult {
            ayes_count: 1,
            ayes_stake: 50,
            nays_count: 0,
            nays_stake: 0,
        };
        assert_eq!(Pips::proposal_result(id), result);

        // Now remove that PIP and check that funds are back.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_state(id, false, ProposalState::Pending);
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_eq!(Pips::active_pip_count(), 0);
        assert_eq!(
            Pips::proposals(id).unwrap(),
            Pip {
                id,
                proposal: make_proposal(42),
                proposer: Proposer::Community(proposer.acc()),
            }
        );
        assert_eq!(Pips::proposal_state(id).unwrap(), ProposalState::Rejected);
        assert_balance(proposer.acc(), init_bal, 0);
        assert_eq!(Deposits::iter_prefix_values(id).count(), 0);
        // We keep this info for posterity.
        assert_eq!(Votes::iter_prefix_values(id).count(), 1);
        assert_eq!(Pips::proposal_result(id), result);
        let id = PipId(1);
        assert_eq!(Pips::pip_id_sequence(), id);

        // Alice starts a proposal with some deposit.
        assert_ok!(community_proposal(proposer, 60));
        assert_balance(proposer.acc(), init_bal, 60);
        let result = VotingResult {
            ayes_count: 1,
            ayes_stake: 60,
            nays_count: 0,
            nays_stake: 0,
        };
        assert_eq!(Pips::proposal_result(id), result);

        // Schedule the PIP.
        assert_ok!(Pips::snapshot(member.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(id, SnapshotResult::Approve)]
        ));
        assert_state(id, false, ProposalState::Scheduled);

        // Now remove that PIP and check that funds are back.
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_eq!(Pips::pip_id_sequence(), PipId(2));
        assert_eq!(Pips::active_pip_count(), 0);
        assert_eq!(
            Pips::proposals(id).unwrap(),
            Pip {
                id,
                proposal: make_proposal(42),
                proposer: Proposer::Community(proposer.acc()),
            }
        );
        assert_eq!(Pips::proposal_state(id).unwrap(), ProposalState::Rejected);
        assert_balance(proposer.acc(), init_bal, 0);
        assert_eq!(Deposits::iter_prefix_values(id).count(), 0);
        // We keep this info for posterity.
        assert_eq!(Votes::iter_prefix_values(id).count(), 1);
        assert_eq!(Pips::proposal_result(id), result);
    });
}

#[test]
fn reject_proposal_will_unsnapshot() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), false));

        let proposer = User::new(AccountKeyring::Alice);
        let member = User::new(AccountKeyring::Bob).balance(300);
        set_members(vec![member.did]);

        let id = PipId(0);
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(Pips::snapshot(member.origin()));
        assert_eq!(Pips::snapshot_queue()[0].id, id);
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_eq!(Pips::snapshot_queue(), vec![]);
    });
}

#[test]
fn reject_proposal_will_unschedule() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), false));

        let alice = User::new(AccountKeyring::Alice).balance(300);
        set_members(vec![alice.did]);

        let check = |id: PipId| {
            let scheduled_at = Pips::pip_to_schedule(id).unwrap();
            assert_ok!(Pips::reject_proposal(gc_vmo(), id));
            assert_eq!(Pips::pip_to_schedule(id), None);
            assert_event_exists!(
                EventTest::Scheduler(pallet_scheduler::Event::Canceled { when, .. }),
                *when == scheduled_at
            );
        };

        // Test snapshot method.
        assert_ok!(community_proposal(alice, 0));
        assert_ok!(Pips::snapshot(alice.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(PipId(0), SnapshotResult::Approve)]
        ));
        check(PipId(0));

        // Test committee method.
        assert_ok!(committee_proposal(0));
        assert_ok!(Pips::approve_committee_proposal(gc_vmo(), PipId(1)));
        check(PipId(1));
    });
}

#[test]
fn reschedule_execution_only_release_coordinator() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);
        set_members(vec![alice.did, bob.did, charlie.did]);
        assert_ok!(Committee::set_release_coordinator(gc_vmo(), charlie.did));

        let id = PipId(0);
        assert_bad_origin!(Pips::reschedule_execution(root(), id, None));
        assert_noop!(
            Pips::reschedule_execution(alice.origin(), id, None),
            Error::RescheduleNotByReleaseCoordinator
        );
        assert_noop!(
            Pips::reschedule_execution(bob.origin(), id, None),
            Error::RescheduleNotByReleaseCoordinator
        );

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let id = scheduled_proposal(alice, alice, 0);
        let scheduled_at = Pips::pip_to_schedule(id);
        consensus_call(
            pallet_pips::Call::reschedule_execution { id, until: None },
            &[&alice.origin(), &bob.origin(), &charlie.origin()],
        );
        assert_eq!(scheduled_at, Pips::pip_to_schedule(id));
        assert_ok!(Pips::reschedule_execution(charlie.origin(), id, None));
        assert_ne!(scheduled_at, Pips::pip_to_schedule(id));
    });
}

fn init_rc() -> User {
    let user = User::new(AccountKeyring::Alice);
    set_members(vec![user.did]);
    assert_ok!(Committee::set_release_coordinator(gc_vmo(), user.did));
    user
}

#[test]
fn reschedule_execution_no_such_proposal() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let rc = init_rc();
        assert_no_pip!(Pips::reschedule_execution(rc.origin(), PipId(0), None));
    });
}

#[test]
fn reschedule_execution_not_scheduled() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let rc = init_rc();
        let proposer = User::new(AccountKeyring::Bob);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let id = Pips::pip_id_sequence();
        assert_ok!(community_proposal(proposer, 0));
        assert_bad_state!(Pips::reschedule_execution(rc.origin(), id, None));
        assert_ok!(Pips::reject_proposal(gc_vmo(), id));
        assert_ok!(Pips::prune_proposal(gc_vmo(), id));
        let id = rejected_proposal(proposer);
        assert_bad_state!(Pips::reschedule_execution(rc.origin(), id, None));
        let id = executed_community_proposal(proposer, rc);
        assert_bad_state!(Pips::reschedule_execution(rc.origin(), id, None));
        let id = failed_community_proposal(proposer, rc, PipId(1337));
        assert_bad_state!(Pips::reschedule_execution(rc.origin(), id, None));
        let id = expired_proposal(proposer, 2);
        assert_bad_state!(Pips::reschedule_execution(rc.origin(), id, None));
    });
}

#[test]
fn reschedule_execution_in_the_past() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let rc = init_rc();
        let proposer = User::new(AccountKeyring::Bob);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        let id = scheduled_proposal(proposer, rc, 0);
        let next = System::block_number() + 1;
        assert_noop!(
            Pips::reschedule_execution(rc.origin(), id, Some(next - 1)),
            Error::InvalidFutureBlockNumber
        );
        assert_noop!(
            Pips::reschedule_execution(rc.origin(), id, Some(next - 2)),
            Error::InvalidFutureBlockNumber
        );
    });
}

#[test]
fn reschedule_execution_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        // General setup.
        let rc = init_rc();
        let proposer = User::new(AccountKeyring::Bob);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        // Schedule a proposal and verify that it is.
        assert_eq!(Pips::active_pip_count(), 0);
        let id = scheduled_proposal(proposer, rc, 0);
        assert_eq!(Pips::active_pip_count(), 1);
        let scheduled_at = Pips::pip_to_schedule(id).unwrap();
        assert!(matches!(Agenda::get(scheduled_at).deref()[..], [Some(_)]));

        // Reschedule execution for next block.
        let next = System::block_number() + 1;
        assert_ok!(Pips::reschedule_execution(rc.origin(), id, None));
        // Regression test for <https://polymath.atlassian.net/browse/GTN-2172>.
        assert_eq!(Pips::active_pip_count(), 1);
        // Rescheduling currently works by cancelling + then scheduling again. Verify this.
        assert_event_exists!(EventTest::Scheduler(
            pallet_scheduler::Event::Canceled { .. }
        ));
        assert_eq!(Pips::pip_to_schedule(id).unwrap(), next);
        assert_eq!(Agenda::get(scheduled_at), vec![]);
        assert!(matches!(Agenda::get(next).deref()[..], [Some(_)]));

        // Reschedule execution for 50 blocks ahead.
        assert_ok!(Pips::reschedule_execution(rc.origin(), id, Some(next + 50)));
        assert_eq!(Pips::active_pip_count(), 1);
        assert_eq!(Pips::pip_to_schedule(id).unwrap(), next + 50);
        assert_eq!(Agenda::get(scheduled_at), vec![]);
        assert_eq!(Agenda::get(next), vec![]);
        assert!(matches!(Agenda::get(next + 50).deref()[..], [Some(_)]));
    });
}

#[test]
fn clear_snapshot_not_gc_member() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        init_rc();
        assert_bad_origin!(Pips::clear_snapshot(root()));
        let bob = User::new(AccountKeyring::Bob);
        assert_noop!(
            Pips::clear_snapshot(bob.origin()),
            Error::NotACommitteeMember,
        );
    });
}

#[test]
fn clear_snapshot_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        let rc = init_rc();
        // No snapshot, but we can still clear.
        assert_eq!(Pips::snapshot_queue(), vec![]);
        assert_eq!(Pips::snapshot_metadata(), None);
        assert_ok!(Pips::clear_snapshot(rc.origin()));
        assert_eq!(Pips::snapshot_queue(), vec![]);
        assert_eq!(Pips::snapshot_metadata(), None);

        // Make a snapshot with something and clear it.
        let proposer = User::new(AccountKeyring::Bob);
        assert_ok!(community_proposal(proposer, 100));
        assert_ok!(community_proposal(proposer, 200));
        assert_ok!(community_proposal(proposer, 400));
        assert_ok!(Pips::snapshot(rc.origin()));
        assert_ne!(Pips::snapshot_queue(), vec![]);
        assert_ne!(Pips::snapshot_metadata(), None);
        assert_ok!(Pips::clear_snapshot(rc.origin()));
        assert_eq!(Pips::snapshot_queue(), vec![]);
        assert_eq!(Pips::snapshot_metadata(), None);
    });
}

#[test]
fn snapshot_not_gc_member() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        init_rc();
        assert_bad_origin!(Pips::snapshot(root()));
        let bob = User::new(AccountKeyring::Bob);
        assert_noop!(Pips::snapshot(bob.origin()), Error::NotACommitteeMember);
    });
}

#[test]
fn snapshot_only_pending_hot_community() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        let rc = init_rc();

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Bob);

        let r = rejected_proposal(proposer);
        let e = executed_community_proposal(proposer, rc);
        let f = failed_community_proposal(proposer, rc, PipId(1337));
        let s = scheduled_proposal(proposer, rc, 0);
        let ex = expired_proposal(proposer, 1);
        assert_ok!(community_proposal(proposer, 0));
        let p = PipId(Pips::pip_id_sequence().0 - 1);
        for id in &[r, e, f, s, p, ex] {
            assert_eq!(
                Pips::proposals(*id).unwrap().proposer,
                Proposer::Community(proposer.acc())
            );
        }
        assert_ok!(committee_proposal(0));

        assert_ok!(Pips::snapshot(rc.origin()));
        assert_eq!(Pips::snapshot_queue(), vec![spip(p.0, true, 0)]);
        assert_ne!(Pips::snapshot_metadata(), None);
    });
}

#[test]
fn snapshot_works() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);

        let proposer = User::new(AccountKeyring::Alice);
        let propose = |d| {
            assert_ok!(community_proposal(proposer, d));
        };

        let member = User::new(AccountKeyring::Bob);
        set_members(vec![member.did]);
        let vote = |id, aye, deposit| Pips::vote(member.origin(), PipId(id), aye, deposit);
        let snapshot = || Pips::snapshot(member.origin());

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_active_pip_limit(root(), 0));

        propose(0); // 0
        assert_ok!(vote(0, true, 100));

        propose(0); // 1
        assert_ok!(vote(1, false, 100));

        propose(0); // 2
        propose(0); // 3

        propose(50); // 4
        assert_ok!(vote(4, true, 100));

        propose(50); // 5
        assert_ok!(vote(5, false, 100));

        assert_ok!(snapshot());
        assert_eq!(
            Pips::snapshot_queue(),
            vec![
                spip(1, false, 100),
                spip(5, false, 50),
                spip(3, true, 0),
                spip(2, true, 0),
                spip(0, true, 100),
                spip(4, true, 150),
            ]
        );

        let assert_snapshot = |id| {
            assert_eq!(
                Pips::snapshot_metadata(),
                Some(SnapshotMetadata {
                    created_at: 1,
                    made_by: member.acc(),
                    id: SnapshotId(id),
                })
            );
        };
        assert_snapshot(0);

        assert_ok!(snapshot());
        assert_snapshot(1);

        assert_ok!(snapshot());
        assert_snapshot(2);
    });
}

#[test]
fn enact_snapshot_results_input_too_large() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let proposer = User::new(AccountKeyring::Alice);
        let member = User::new(AccountKeyring::Bob);
        set_members(vec![member.did]);

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        assert_ok!(Pips::snapshot(member.origin()));
        assert_noop!(
            Pips::enact_snapshot_results(gc_vmo(), vec![(PipId(0), SnapshotResult::Skip)]),
            Error::SnapshotResultTooLarge,
        );
        assert_noop!(
            Pips::enact_snapshot_results(
                gc_vmo(),
                vec![
                    (PipId(0), SnapshotResult::Reject),
                    (PipId(1), SnapshotResult::Approve)
                ]
            ),
            Error::SnapshotResultTooLarge,
        );
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(Pips::snapshot(member.origin()));
        assert_noop!(
            Pips::enact_snapshot_results(
                gc_vmo(),
                vec![
                    (PipId(0), SnapshotResult::Reject),
                    (PipId(1), SnapshotResult::Approve),
                    (PipId(2), SnapshotResult::Skip)
                ]
            ),
            Error::SnapshotResultTooLarge,
        );
    });
}

#[test]
fn enact_snapshot_results_id_mismatch() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let user = User::new(AccountKeyring::Bob);
        set_members(vec![user.did]);

        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);
        assert_ok!(community_proposal(proposer, 0));
        assert_ok!(community_proposal(proposer, 0));

        assert_ok!(Pips::snapshot(user.origin()));
        assert_noop!(
            Pips::enact_snapshot_results(gc_vmo(), vec![(PipId(1), SnapshotResult::Skip)]),
            Error::SnapshotIdMismatch,
        );
        assert_noop!(
            Pips::enact_snapshot_results(
                gc_vmo(),
                vec![
                    (PipId(0), SnapshotResult::Reject),
                    (PipId(2), SnapshotResult::Approve)
                ]
            ),
            Error::SnapshotIdMismatch,
        );
    });
}

#[test]
fn enact_snapshot_results_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);

        let proposer = User::new(AccountKeyring::Alice);
        let propose = || {
            assert_ok!(community_proposal(proposer, 0));
        };

        let member = User::new(AccountKeyring::Bob);
        set_members(vec![member.did]);

        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let mk_queue = |ids: &[u32]| ids.iter().map(|&id| spip(id, true, 0)).collect::<Vec<_>>();

        // Make 3 PIPs, snapshot, and enact results for all, emptying the queue.
        propose();
        propose();
        propose();
        assert_ok!(Pips::snapshot(member.origin()));
        assert_eq!(Pips::snapshot_queue(), mk_queue(&[2, 1, 0]));
        assert_eq!(Pips::pip_skip_count(PipId(1)), 0);
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![
                (PipId(0), SnapshotResult::Reject),
                (PipId(1), SnapshotResult::Skip),
                (PipId(2), SnapshotResult::Approve),
            ],
        ));
        assert_state(PipId(0), false, ProposalState::Rejected);
        assert_state(PipId(1), false, ProposalState::Pending);
        assert_eq!(Pips::pip_skip_count(PipId(1)), 1);
        assert_state(PipId(2), false, ProposalState::Scheduled);
        assert_eq!(Pips::snapshot_queue(), vec![]);
        assert_ne!(Pips::snapshot_metadata(), None);

        // Add another proposal; we previously skipped one, so queue size is 2.
        // Only enact for 1 proposal, leaving the last added PIP in the queue.
        propose();
        assert_ok!(Pips::snapshot(member.origin()));
        assert_eq!(Pips::snapshot_queue(), mk_queue(&[3, 1]));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(PipId(1), SnapshotResult::Approve)]
        ));
        assert_last_event!(
            Event::SnapshotResultsEnacted(_, Some(SnapshotId(1)), a, b, c),
            a.is_empty() && b.is_empty() && c == &[PipId(1)]
        );
        assert_state(PipId(1), false, ProposalState::Scheduled);
        assert_eq!(Pips::snapshot_queue(), mk_queue(&[3]));

        // Cleared queue + enacting zero-length results => noop.
        assert_ok!(Pips::clear_snapshot(member.origin()));
        assert_ok!(Pips::enact_snapshot_results(gc_vmo(), vec![]));
        assert_last_event!(
            Event::SnapshotResultsEnacted(_, None, a, b, c),
            a.is_empty() && b.is_empty() && c.is_empty()
        );
    });
}

#[test]
fn expiry_works() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        // Test non-prune logic. Prune logic is tested elsewhere.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        let proposer = User::new(AccountKeyring::Bob);
        let id = expired_proposal(proposer, 13);
        assert_state(id, true, ProposalState::Expired);
        // Travel back in time, and ensure expiry is sticky.
        // This doesn't arise in a real world scenario, but tests an edge case in the code.
        System::set_block_number(1);
        assert_state(id, true, ProposalState::Expired);

        // Make sure non-pending PIPs cannot expire.
        assert_ok!(Pips::set_prune_historical_pips(root(), false));
        assert_ok!(Pips::set_pending_pip_expiry(root(), MaybeBlock::Some(13)));
        let member = User::new(AccountKeyring::Alice);
        set_members(vec![member.did]);
        let r = rejected_proposal(proposer);
        let e = executed_community_proposal(proposer, member);
        let f = failed_community_proposal(proposer, member, PipId(1337));
        let s = scheduled_proposal(proposer, member, 0);
        fast_forward_blocks(13 + 100);
        for id in &[r, e, f, s] {
            assert_ne!(Pips::proposal_state(id).unwrap(), ProposalState::Expired);
        }
    });
}

#[test]
#[should_panic = "called `Result::unwrap_err()` on an `Ok` value: 0"]
fn propose_dupe_live_insert_panics() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        // Manipulate storage to provoke panic in `insert_live_queue`.
        LiveQueue::mutate(|queue| *queue = vec![spip(0, true, 0)]);

        // Triggers a panic, assertion never reached.
        assert_ok!(community_proposal(User::new(AccountKeyring::Alice), 0));
    });
}

#[test]
fn execute_scheduled_pip() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), true));
        let pip_id = Pips::pip_id_sequence();
        let user = User::new(AccountKeyring::Alice);
        assert_ok!(remark_proposal(user, 0));
        set_members(vec![user.did]);
        assert_ok!(Pips::snapshot(user.origin()));
        assert_ok!(Pips::enact_snapshot_results(
            gc_vmo(),
            vec![(pip_id, SnapshotResult::Approve)],
        ));
        assert_state(pip_id, false, ProposalState::Scheduled);
        assert_ok!(Pips::execute_scheduled_pip(root(), pip_id));
        assert_pruned(pip_id);
    });
}

#[test]
fn expire_scheduled_pip() {
    ExtBuilder::default().build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_prune_historical_pips(root(), true));
        let pip_id = Pips::pip_id_sequence();
        let user = User::new(AccountKeyring::Alice);
        assert_ok!(remark_proposal(user, 0));
        assert_state(pip_id, false, ProposalState::Pending);
        assert_ok!(Pips::expire_scheduled_pip(root(), GC_DID, pip_id));
        assert_pruned(pip_id);
    });
}

#[test]
fn live_queue_off_by_one_insertion_regression_test() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);
        assert_ok!(community_proposal(proposer, 2));
        assert_ok!(community_proposal(proposer, 4));
        assert_eq!(Pips::live_queue(), vec![spip(0, true, 2), spip(1, true, 4)]);

        let user = User::new(AccountKeyring::Bob);
        assert_ok!(Pips::vote(user.origin(), PipId(0), true, 1));
        assert_eq!(Pips::live_queue(), vec![spip(0, true, 3), spip(1, true, 4)]);
    });
}

#[test]
fn live_queue_off_by_one_insertion_regression_test2() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));
        assert_ok!(Pips::set_active_pip_limit(root(), 0));

        let proposer = User::new(AccountKeyring::Alice);
        let voter = User::new(AccountKeyring::Bob);

        assert_ok!(community_proposal(proposer, 0)); // 0
        assert_ok!(community_proposal(proposer, 0)); // 1
        assert_ok!(community_proposal(proposer, 50)); // 2
        assert_ok!(Pips::vote(voter.origin(), PipId(0), false, 100));
        assert_ok!(Pips::vote(voter.origin(), PipId(2), false, 100));
        assert_eq!(
            Pips::live_queue(),
            vec![spip(0, false, 100), spip(2, false, 50), spip(1, true, 0)]
        );
    });
}

#[test]
fn pips_rpcs() {
    ExtBuilder::default().monied(true).build().execute_with(|| {
        let alice = User::new(AccountKeyring::Alice);
        let bob = User::new(AccountKeyring::Bob);
        let charlie = User::new(AccountKeyring::Charlie);

        System::set_block_number(1);
        assert_ok!(Pips::set_min_proposal_deposit(root(), 0));

        // Create two community proposals with IDs 0 and 1.
        assert_ok!(community_proposal(alice, 0));
        assert_ok!(community_proposal(alice, 0));
        let pip_id0 = PipId(0);
        let pip_id1 = PipId(1);

        let bob_vote_deposit = 100;
        let charlie_vote_deposit = 200;
        assert_ok!(Pips::vote(bob.origin(), pip_id0, false, bob_vote_deposit));
        assert_ok!(Pips::vote(bob.origin(), pip_id1, true, bob_vote_deposit));
        assert_ok!(Pips::vote(
            charlie.origin(),
            pip_id0,
            true,
            charlie_vote_deposit
        ));

        assert_eq!(
            Pips::get_votes(pip_id0),
            VoteCount::ProposalFound {
                ayes: charlie_vote_deposit,
                nays: bob_vote_deposit,
            }
        );
        assert_eq!(
            Pips::proposed_by(Proposer::Community(AccountKeyring::Alice.to_account_id())),
            vec![pip_id1, pip_id0],
        );
        assert_eq!(Pips::voted_on(bob.acc()), vec![pip_id1, pip_id0]);
    });
}
