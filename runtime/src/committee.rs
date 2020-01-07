//! # Committee Module
//!
//! The Group module is used to manage a set of identities. A group of identities can be a
//! collection of KYC providers, council members for governance and so on. This is an instantiable
//! module.
//!
//! ## Overview
//! Allows control of membership of a set of `IdentityId`s, useful for managing membership of a
//! collective.
//!
//!
//! ### Dispatchable Functions
//!
//!
use crate::identity;
use primitives::{IdentityId, Key, Signer};
use rstd::{convert::TryFrom, prelude::*, result};
use sr_primitives::traits::{EnsureOrigin, Hash};
use sr_primitives::weights::SimpleDispatchInfo;
#[cfg(feature = "std")]
use sr_primitives::{Deserialize, Serialize};
use srml_support::{
    codec::{Decode, Encode},
    decl_event, decl_module, decl_storage,
    dispatch::{Dispatchable, Parameter},
    ensure,
    traits::{ChangeMembers, InitializeMembers},
};
use substrate_primitives::u32_trait::Value as U32;
use system::{self, ensure_root, ensure_signed};

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// The number of committee members
pub type MemberCount = u32;

pub trait Trait<I = DefaultInstance>: system::Trait + identity::Trait {
    /// The outer origin type.
    type Origin: From<RawOrigin<Self::AccountId, I>>;

    /// The outer call dispatch type.
    type Proposal: Parameter + Dispatchable<Origin = <Self as Trait<I>>::Origin>;

    /// The outer event type.
    type Event: From<Event<Self, I>> + Into<<Self as system::Trait>::Event>;
}

#[derive(Clone, Copy, PartialEq, Eq, Encode, Decode, Debug)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub enum ProportionMatch {
    AtLeast,
    MoreThan,
}

impl Default for ProportionMatch {
    fn default() -> Self {
        ProportionMatch::MoreThan
    }
}

/// Origin for the committee module.
#[derive(PartialEq, Eq, Clone, Debug)]
pub enum RawOrigin<AccountId, I> {
    /// It has been condoned by M of N members of this committee.
    Members(MemberCount, MemberCount),
    /// Dummy to manage the fact we have instancing.
    _Phantom(rstd::marker::PhantomData<(AccountId, I)>),
}

/// Origin for the committee module.
pub type Origin<T, I = DefaultInstance> = RawOrigin<<T as system::Trait>::AccountId, I>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
/// Info for keeping track of a motion being voted on.
pub struct Votes<IdentityId> {
    /// The proposal's unique index.
    index: ProposalIndex,
    /// The current set of commmittee members that approved it.
    ayes: Vec<IdentityId>,
    /// The current set of commmittee members that rejected it.
    nays: Vec<IdentityId>,
}

decl_storage! {
    trait Store for Module<T: Trait<I>, I: Instance=DefaultInstance> as Committee {
        /// The hashes of the active proposals.
        pub Proposals get(proposals): Vec<T::Hash>;
        /// Actual proposal for a given hash.
        pub ProposalOf get(proposal_of): map T::Hash => Option<<T as Trait<I>>::Proposal>;
        /// Votes on a given proposal, if it is ongoing.
        pub Voting get(voting): map T::Hash => Option<Votes<IdentityId>>;
        /// Proposals so far.
        pub ProposalCount get(proposal_count): u32;
        /// The current members of the committee.
        pub Members get(members) config(): Vec<IdentityId>;
        /// Vote threshold for an approval.
        pub VoteThreshold get(vote_threshold) config(): (ProportionMatch, u32, u32);
    }
    add_extra_genesis {
        config(phantom): rstd::marker::PhantomData<(T, I)>;
    }
}

decl_event!(
	pub enum Event<T, I=DefaultInstance> where
		<T as system::Trait>::Hash,
	{
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed(IdentityId, ProposalIndex, Hash),
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes and no votes given respectively as `MemberCount`).
		Voted(IdentityId, Hash, bool, MemberCount, MemberCount),
		/// A motion was approved by the required threshold.
		Approved(Hash),
		/// A motion was rejected by the required threshold.
		Rejected(Hash),
		/// A motion was executed; `bool` is true if returned without error.
		Executed(Hash, bool),
	}
);

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance> for enum Call where origin: <T as system::Trait>::Origin {
        fn deposit_event() = default;

        /// Set the committee's membership manually to `new_members`.
        /// Requires root origin.
        ///
        /// # Arguments
        /// * `origin` Root
        /// * `new_members` Members to be initialized as committee.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        fn set_members(origin, new_members: Vec<IdentityId>) {
            ensure_root(origin)?;

            let mut new_members = new_members;
            new_members.sort();
            <Members<I>>::mutate(|m| {
                *m = new_members;
            });
        }

        /// Any committee member proposes a dispatchable.
        ///
        /// # Arguments
        /// * `did` Identity of the proposer
        /// * `proposal` A dispatchable call
        #[weight = SimpleDispatchInfo::FixedOperational(5_000_000)]
        fn propose(origin, did: IdentityId, proposal: Box<<T as Trait<I>>::Proposal>) {
            let who = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(who.encode())?);

            // Ensure sender can sign for the given identity
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Only committee members can propose
            ensure!(Self::is_member(&did), "proposer is not a member");

            // Reject duplicate proposals
            let proposal_hash = T::Hashing::hash_of(&proposal);
            ensure!(!<ProposalOf<T, I>>::exists(proposal_hash), "duplicate proposals not allowed");

            let index = Self::proposal_count();
            <ProposalCount<I>>::mutate(|i| *i += 1);
            <Proposals<T, I>>::mutate(|proposals| proposals.push(proposal_hash));
            <ProposalOf<T, I>>::insert(proposal_hash, *proposal);

            let votes = Votes { index, ayes: vec![did.clone()], nays: vec![] };
            <Voting<T, I>>::insert(proposal_hash, votes);

            Self::deposit_event(RawEvent::Proposed(did, index, proposal_hash));
        }

        /// Member casts a vote.
        ///
        /// # Arguments
        /// * `did` Identity of the proposer
        /// * `proposal` Hash of proposal to be voted on
        /// * `index` Proposal index
        /// * `approve` Represents a `for` or `against` vote
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        fn vote(origin, did: IdentityId, proposal: T::Hash, #[compact] index: ProposalIndex, approve: bool) {
            let who = ensure_signed(origin)?;
            let signer = Signer::Key(Key::try_from(who.encode())?);

            // Ensure sender can sign for the given identity
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &signer), "sender must be a signing key for DID");

            // Only committee members can vote
            ensure!(Self::is_member(&did), "voter is not a member");

            let mut voting = Self::voting(&proposal).ok_or("proposal must exist")?;
            ensure!(voting.index == index, "mismatched index");

            let position_yes = voting.ayes.iter().position(|a| a == &did);
            let position_no = voting.nays.iter().position(|a| a == &did);

            if approve {
                if position_yes.is_none() {
                    voting.ayes.push(did.clone());
                } else {
                    return Err("duplicate vote ignored")
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push(did.clone());
                } else {
                    return Err("duplicate vote ignored")
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;
            Self::deposit_event(RawEvent::Voted(did, proposal, approve, yes_votes, no_votes));

            let threshold = <VoteThreshold<I>>::get();
            let seats = Self::members().len() as MemberCount;

            let approved = Self::is_threshold_satisfied(yes_votes, seats, threshold);
            let rejected = Self::is_threshold_satisfied(no_votes, seats, threshold);

            if approved || rejected {
                if approved {
                    Self::deposit_event(RawEvent::Approved(proposal));

                    // execute motion, assuming it exists.
                    if let Some(p) = <ProposalOf<T, I>>::take(&proposal) {
                        let origin = RawOrigin::Members(yes_votes, seats).into();
                        let ok = p.dispatch(origin).is_ok();
                        Self::deposit_event(RawEvent::Executed(proposal, ok));
                    }
                } else {
                    // rejected
                    Self::deposit_event(RawEvent::Rejected(proposal));
                }

                // remove vote
                <Voting<T, I>>::remove(&proposal);
                <Proposals<T, I>>::mutate(|proposals| proposals.retain(|h| h != &proposal));
            } else {
                // update voting
                <Voting<T, I>>::insert(&proposal, voting);
            }
        }
    }
}

impl<T: Trait<I>, I: Instance> Module<T, I> {
    /// Returns true if given did is contained in `Members` set. `false`, otherwise.
    pub fn is_member(who: &IdentityId) -> bool {
        Self::members().contains(who)
    }

    /// Given `votes` number of votes out of `total` votes, this function compares`votes`/`total`
    /// in relation to the threshold proporion `n`/`d`.   
    fn is_threshold_satisfied(
        votes: u32,
        total: u32,
        (threshold, n, d): (ProportionMatch, u32, u32),
    ) -> bool {
        match threshold {
            ProportionMatch::AtLeast => votes * d >= n * total,
            ProportionMatch::MoreThan => votes * d > n * total,
        }
    }
}

pub struct EnsureProportionMoreThan<N: U32, D: U32, AccountId, I = DefaultInstance>(
    rstd::marker::PhantomData<(N, D, AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        N: U32,
        D: U32,
        AccountId,
        I,
    > EnsureOrigin<O> for EnsureProportionMoreThan<N, D, AccountId, I>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D::VALUE > N::VALUE * m => Ok(()),
            r => Err(O::from(r)),
        })
    }
}

pub struct EnsureProportionAtLeast<N: U32, D: U32, AccountId, I = DefaultInstance>(
    rstd::marker::PhantomData<(N, D, AccountId, I)>,
);
impl<
        O: Into<Result<RawOrigin<AccountId, I>, O>> + From<RawOrigin<AccountId, I>>,
        N: U32,
        D: U32,
        AccountId,
        I,
    > EnsureOrigin<O> for EnsureProportionAtLeast<N, D, AccountId, I>
{
    type Success = ();
    fn try_origin(o: O) -> Result<Self::Success, O> {
        o.into().and_then(|o| match o {
            RawOrigin::Members(n, m) if n * D::VALUE >= N::VALUE * m => Ok(()),
            r => Err(O::from(r)),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate as committee;
    use crate::{balances, identity};
    use hex_literal::hex;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, Block as BlockT, ConvertInto, IdentityLookup, Verify},
        AnySignature, BuildStorage, Perbill,
    };
    use srml_support::{
        assert_noop, assert_ok,
        dispatch::{DispatchError, DispatchResult},
        parameter_types, Hashable,
    };
    use substrate_primitives::{Blake2Hasher, H256};
    use system::{EventRecord, Phase};

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    type AccountId = <AnySignature as Verify>::Signer;

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = u64;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type WeightMultiplierUpdate = ();
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
    }

    parameter_types! {
        pub const ExistentialDeposit: u64 = 0;
        pub const TransferFee: u64 = 0;
        pub const CreationFee: u64 = 0;
        pub const TransactionBaseFee: u64 = 0;
        pub const TransactionByteFee: u64 = 0;
    }

    impl balances::Trait for Test {
        type Balance = u128;
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type TransactionPayment = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type TransactionBaseFee = TransactionBaseFee;
        type TransactionByteFee = TransactionByteFee;
        type WeightToFee = ConvertInto;
        type Identity = identity::Module<Test>;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    #[derive(codec::Encode, codec::Decode, Debug, Clone, Eq, PartialEq)]
    pub struct IdentityProposal {
        pub dummy: u8,
    }

    impl sr_primitives::traits::Dispatchable for IdentityProposal {
        type Origin = Origin;
        type Trait = Test;
        type Error = DispatchError;

        fn dispatch(self, _origin: Self::Origin) -> DispatchResult<Self::Error> {
            Ok(())
        }
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = IdentityProposal;
    }

    impl Trait<Instance1> for Test {
        type Origin = Origin;
        type Proposal = Call;
        type Event = Event;
    }

    impl Trait for Test {
        type Origin = Origin;
        type Proposal = Call;
        type Event = Event;
    }

    pub type Block = sr_primitives::generic::Block<Header, UncheckedExtrinsic>;
    pub type UncheckedExtrinsic = sr_primitives::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

    srml_support::construct_runtime!(
        pub enum Test where
            Block = Block,
            NodeBlock = Block,
            UncheckedExtrinsic = UncheckedExtrinsic
        {
            System: system::{Module, Call, Event},
            Committee: committee::<Instance1>::{Module, Call, Event<T>, Origin<T>, Config<T>},
            DefaultCommittee: committee::{Module, Call, Event<T>, Origin<T>, Config<T>},
        }
    );

    fn make_ext() -> sr_io::TestExternalities<Blake2Hasher> {
        GenesisConfig {
            committee_Instance1: Some(committee::GenesisConfig {
                members: vec![
                    IdentityId::from(1),
                    IdentityId::from(2),
                    IdentityId::from(3),
                ],
                vote_threshold: (ProportionMatch::AtLeast, 1, 2),
                phantom: Default::default(),
            }),
            committee: None,
        }
        .build_storage()
        .unwrap()
        .into()
    }

    #[test]
    fn motions_basic_environment_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);
            assert_eq!(
                Committee::members(),
                vec![
                    IdentityId::from(1),
                    IdentityId::from(2),
                    IdentityId::from(3),
                ]
            );
            assert_eq!(Committee::proposals(), Vec::<H256>::new());
        });
    }

    fn make_proposal(value: u64) -> Call {
        Call::System(system::Call::remark(value.encode()))
    }

    //    #[test]
    //    fn removal_of_old_voters_votes_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash = BlakeTwo256::hash_of(&proposal);
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(2), hash.clone(), 0, true));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 3,
    //                    ayes: vec![1, 2],
    //                    nays: vec![]
    //                })
    //            );
    //            Committee::change_members_sorted(&[4], &[1], &[2, 3, 4]);
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 3,
    //                    ayes: vec![2],
    //                    nays: vec![]
    //                })
    //            );
    //
    //            let proposal = make_proposal(69);
    //            let hash = BlakeTwo256::hash_of(&proposal);
    //            assert_ok!(Committee::propose(
    //                Origin::signed(2),
    //                2,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(3), hash.clone(), 1, false));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 1,
    //                    threshold: 2,
    //                    ayes: vec![2],
    //                    nays: vec![3]
    //                })
    //            );
    //            Committee::change_members_sorted(&[], &[3], &[2, 4]);
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 1,
    //                    threshold: 2,
    //                    ayes: vec![2],
    //                    nays: vec![]
    //                })
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn removal_of_old_voters_votes_works_with_set_members() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash = BlakeTwo256::hash_of(&proposal);
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(2), hash.clone(), 0, true));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 3,
    //                    ayes: vec![1, 2],
    //                    nays: vec![]
    //                })
    //            );
    //            assert_ok!(Committee::set_members(Origin::ROOT, vec![2, 3, 4]));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 3,
    //                    ayes: vec![2],
    //                    nays: vec![]
    //                })
    //            );
    //
    //            let proposal = make_proposal(69);
    //            let hash = BlakeTwo256::hash_of(&proposal);
    //            assert_ok!(Committee::propose(
    //                Origin::signed(2),
    //                2,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(3), hash.clone(), 1, false));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 1,
    //                    threshold: 2,
    //                    ayes: vec![2],
    //                    nays: vec![3]
    //                })
    //            );
    //            assert_ok!(Committee::set_members(Origin::ROOT, vec![2, 4]));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 1,
    //                    threshold: 2,
    //                    ayes: vec![2],
    //                    nays: vec![]
    //                })
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn propose_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_eq!(Committee::proposals(), vec![hash]);
    //            assert_eq!(Committee::proposal_of(&hash), Some(proposal));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 3,
    //                    ayes: vec![1],
    //                    nays: vec![]
    //                })
    //            );
    //
    //            assert_eq!(
    //                System::events(),
    //                vec![EventRecord {
    //                    phase: Phase::Finalization,
    //                    event: Event::committee_Instance1(RawEvent::Proposed(
    //                        1,
    //                        0,
    //                        hex!["68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"]
    //                            .into(),
    //                        3,
    //                    )),
    //                    topics: vec![],
    //                }]
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_ignoring_non_committee_proposals_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            assert_noop!(
    //                Committee::propose(Origin::signed(42), 3, Box::new(proposal.clone())),
    //                "proposer not a member"
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_ignoring_non_committee_votes_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash: H256 = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_noop!(
    //                Committee::vote(Origin::signed(42), hash.clone(), 0, true),
    //                "voter not a member",
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_ignoring_bad_index_committee_vote_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(3);
    //            let proposal = make_proposal(42);
    //            let hash: H256 = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_noop!(
    //                Committee::vote(Origin::signed(2), hash.clone(), 1, true),
    //                "mismatched index",
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_revoting_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash: H256 = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                2,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 2,
    //                    ayes: vec![1],
    //                    nays: vec![]
    //                })
    //            );
    //            assert_noop!(
    //                Committee::vote(Origin::signed(1), hash.clone(), 0, true),
    //                "duplicate vote ignored",
    //            );
    //            assert_ok!(Committee::vote(Origin::signed(1), hash.clone(), 0, false));
    //            assert_eq!(
    //                Committee::voting(&hash),
    //                Some(Votes {
    //                    index: 0,
    //                    threshold: 2,
    //                    ayes: vec![],
    //                    nays: vec![1]
    //                })
    //            );
    //            assert_noop!(
    //                Committee::vote(Origin::signed(1), hash.clone(), 0, false),
    //                "duplicate vote ignored",
    //            );
    //
    //            assert_eq!(
    //                System::events(),
    //                vec![
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Proposed(
    //                            1,
    //                            0,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            2,
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Voted(
    //                            1,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            false,
    //                            0,
    //                            1,
    //                        )),
    //                        topics: vec![],
    //                    }
    //                ]
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_disapproval_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash: H256 = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                3,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(2), hash.clone(), 0, false));
    //
    //            assert_eq!(
    //                System::events(),
    //                vec![
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Proposed(
    //                            1,
    //                            0,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            3,
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Voted(
    //                            2,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            false,
    //                            1,
    //                            1,
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Disapproved(
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                        )),
    //                        topics: vec![],
    //                    }
    //                ]
    //            );
    //        });
    //    }
    //
    //    #[test]
    //    fn motions_approval_works() {
    //        make_ext().execute_with(|| {
    //            System::set_block_number(1);
    //            let proposal = make_proposal(42);
    //            let hash: H256 = proposal.blake2_256().into();
    //            assert_ok!(Committee::propose(
    //                Origin::signed(1),
    //                2,
    //                Box::new(proposal.clone())
    //            ));
    //            assert_ok!(Committee::vote(Origin::signed(2), hash.clone(), 0, true));
    //
    //            assert_eq!(
    //                System::events(),
    //                vec![
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Proposed(
    //                            1,
    //                            0,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            2,
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Voted(
    //                            2,
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            true,
    //                            2,
    //                            0,
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Approved(
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                        )),
    //                        topics: vec![],
    //                    },
    //                    EventRecord {
    //                        phase: Phase::Finalization,
    //                        event: Event::committee_Instance1(RawEvent::Executed(
    //                            hex![
    //                                "68eea8f20b542ec656c6ac2d10435ae3bd1729efc34d1354ab85af840aad2d35"
    //                            ]
    //                            .into(),
    //                            false,
    //                        )),
    //                        topics: vec![],
    //                    }
    //                ]
    //            );
    //        });
    //    }
}
