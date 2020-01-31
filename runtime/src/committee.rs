//! # Committee Module
//!
//! The Committee module is used to create a committee of members who vote and ratify proposals.
//! This was based on Substrate's `srml-collective` but this module differs in the following way:
//! - Winning proposal is determined by a vote threshold which is set at genesis
//! - Vote threshold can be modified per instance
//! - Membership consists of DIDs
//!
//! ## Overview
//! Allows control of membership of a set of `IdentityId`s, useful for managing membership of a
//! committee.
//! - Add members to committee
//! - Members can propose a dispatchable
//! - Members vote on a proposal.
//! - Proposal automatically dispatches if it meets a vote threshold
//!
//! ### Dispatchable Functions
//! - `set_members` - Initialize membership. Called by Root.
//! - `propose` - Members can propose a new dispatchable
//! - `vote` - Members vote on proposals which are automatically dispatched if they meet vote threshold
//!
use crate::identity;
use primitives::{IdentityId, Key, Signer};
use sp_runtime::traits::{EnsureOrigin, Hash};
#[cfg(feature = "std")]
use sp_runtime::{Deserialize, Serialize};
use sp_std::{convert::TryFrom, prelude::*};

use frame_support::{
    codec::{Decode, Encode},
    decl_error, decl_event, decl_module, decl_storage,
    dispatch::{Dispatchable, Parameter},
    ensure,
    traits::{ChangeMembers, InitializeMembers},
    weights::SimpleDispatchInfo,
};
use frame_system::{self as system, ensure_root, ensure_signed};
use sp_core::u32_trait::Value as U32;

/// Simple index type for proposal counting.
pub type ProposalIndex = u32;

/// The number of committee members
pub type MemberCount = u32;

pub trait Trait<I = DefaultInstance>: frame_system::Trait + identity::Trait {
    /// The outer origin type.
    type Origin: From<RawOrigin<Self::AccountId, I>>;

    /// The outer call dispatch type.
    type Proposal: Parameter + Dispatchable<Origin = <Self as Trait<I>>::Origin>;

    /// Required origin for changing behaviour of this module.
    type CommitteeOrigin: EnsureOrigin<<Self as frame_system::Trait>::Origin>;

    /// The outer event type.
    type Event: From<Event<Self, I>> + Into<<Self as frame_system::Trait>::Event>;
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
    _Phantom(sp_std::marker::PhantomData<(AccountId, I)>),
}

/// Origin for the committee module.
pub type Origin<T, I = DefaultInstance> = RawOrigin<<T as system::Trait>::AccountId, I>;

#[derive(PartialEq, Eq, Clone, Encode, Decode, Debug)]
/// Info for keeping track of a motion being voted on.
pub struct PolymeshVotes<IdentityId> {
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
        pub Proposals get(fn proposals): Vec<T::Hash>;
        /// Actual proposal for a given hash.
        pub ProposalOf get(fn proposal_of): map T::Hash => Option<<T as Trait<I>>::Proposal>;
        /// PolymeshVotes on a given proposal, if it is ongoing.
        pub Voting get(fn voting): map T::Hash => Option<PolymeshVotes<IdentityId>>;
        /// Proposals so far.
        pub ProposalCount get(fn proposal_count): u32;
        /// The current members of the committee.
        pub Members get(fn members) config(): Vec<IdentityId>;
        /// Vote threshold for an approval.
        pub VoteThreshold get(fn vote_threshold) config(): (ProportionMatch, u32, u32);
    }
    add_extra_genesis {
        config(phantom): sp_std::marker::PhantomData<(T, I)>;
    }
}

decl_event!(
	pub enum Event<T, I=DefaultInstance> where
		<T as frame_system::Trait>::Hash,
	{
		/// A motion (given hash) has been proposed (by given account) with a threshold (given
		/// `MemberCount`).
		Proposed(IdentityId, ProposalIndex, Hash),
		/// A motion (given hash) has been voted on by given account, leaving
		/// a tally (yes votes, no votes and total seats given respectively as `MemberCount`).
		Voted(IdentityId, Hash, bool, MemberCount, MemberCount, MemberCount),
		/// A motion was approved by the required threshold with the following
		/// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
		Approved(Hash, MemberCount, MemberCount, MemberCount),
		/// A motion was rejected by the required threshold with the following
		/// tally (yes votes, no votes and total seats given respectively as `MemberCount`).
		Rejected(Hash, MemberCount, MemberCount, MemberCount),
		/// A motion was executed; `bool` is true if returned without error.
		Executed(Hash, bool),
	}
);

decl_error!(
    pub enum Error for Module<T: Trait<I>, I: Instance> {
        /// Duplicate vote ignored
        DuplicateVote,
    }
);

decl_module! {
    pub struct Module<T: Trait<I>, I: Instance=DefaultInstance> for enum Call where origin: <T as frame_system::Trait>::Origin {

        type Error = Error<T, I>;

        fn deposit_event() = default;

        /// Change the vote threshold the determines the winning proposal. For e.g., for a simple
        /// majority use (ProportionMatch.AtLeast, 1, 2) which represents the inequation ">= 1/2"
        ///
        /// # Arguments
        /// * `match_criteria` One of {AtLeast, MoreThan}
        /// * `n` Numerator of the fraction representing vote threshold
        /// * `d` Denominator of the fraction representing vote threshold
        /// * `match_criteria` One of {AtLeast, MoreThan}
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        fn set_vote_threshold(origin, match_criteria: ProportionMatch, n: u32, d: u32) {
            T::CommitteeOrigin::try_origin(origin)
                .map(|_| ())
                .or_else(ensure_root)
                .map_err(|_| "bad origin")?;
            <VoteThreshold<I>>::put((match_criteria, n, d));
        }

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

            let votes = PolymeshVotes { index, ayes: vec![did.clone()], nays: vec![] };
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
                    return Err(Error::<T, I>::DuplicateVote.into())
                }
                if let Some(pos) = position_no {
                    voting.nays.swap_remove(pos);
                }
            } else {
                if position_no.is_none() {
                    voting.nays.push(did.clone());
                } else {
                    return Err(Error::<T, I>::DuplicateVote.into())
                }
                if let Some(pos) = position_yes {
                    voting.ayes.swap_remove(pos);
                }
            }

            let seats = Self::members().len() as MemberCount;
            let yes_votes = voting.ayes.len() as MemberCount;
            let no_votes = voting.nays.len() as MemberCount;
            Self::deposit_event(RawEvent::Voted(did, proposal, approve, yes_votes, no_votes, seats));

            let threshold = <VoteThreshold<I>>::get();

            let approved = Self::is_threshold_satisfied(yes_votes, seats, threshold);
            let rejected = Self::is_threshold_satisfied(no_votes, seats, threshold);

            if approved || rejected {
                if approved {
                    Self::deposit_event(RawEvent::Approved(proposal, yes_votes, no_votes, seats));

                    // execute motion, assuming it exists.
                    if let Some(p) = <ProposalOf<T, I>>::take(&proposal) {
                        let origin = RawOrigin::Members(yes_votes, seats).into();
                        let ok = p.dispatch(origin).is_ok();
                        Self::deposit_event(RawEvent::Executed(proposal, ok));
                    }
                } else {
                    // rejected
                    Self::deposit_event(RawEvent::Rejected(proposal, yes_votes, no_votes, seats));
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

impl<T: Trait<I>, I: Instance> ChangeMembers<IdentityId> for Module<T, I> {
    fn change_members_sorted(
        _incoming: &[IdentityId],
        outgoing: &[IdentityId],
        new: &[IdentityId],
    ) {
        // remove accounts from all current voting in motions.
        let mut outgoing = outgoing.to_vec();
        outgoing.sort_unstable();

        for h in Self::proposals().into_iter() {
            <Voting<T, I>>::mutate(h, |v| {
                if let Some(mut votes) = v.take() {
                    votes.ayes = votes
                        .ayes
                        .into_iter()
                        .filter(|i| outgoing.binary_search(i).is_err())
                        .collect();

                    votes.nays = votes
                        .nays
                        .into_iter()
                        .filter(|i| outgoing.binary_search(i).is_err())
                        .collect();
                    *v = Some(votes);
                }
            });
        }
        <Members<I>>::put(new);
    }
}

impl<T: Trait<I>, I: Instance> InitializeMembers<IdentityId> for Module<T, I> {
    fn initialize_members(members: &[IdentityId]) {
        if !members.is_empty() {
            assert!(
                <Members<I>>::get().is_empty(),
                "Members are already initialized!"
            );
            <Members<I>>::put(members);
        }
    }
}

pub struct EnsureProportionMoreThan<N: U32, D: U32, AccountId, I = DefaultInstance>(
    sp_std::marker::PhantomData<(N, D, AccountId, I)>,
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
    sp_std::marker::PhantomData<(N, D, AccountId, I)>,
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
    use crate::{balances, committee, group, identity};
    use core::result::Result as StdResult;
    use frame_support::{
        assert_noop, assert_ok, dispatch::DispatchResult, parameter_types, Hashable,
    };
    use frame_system::EnsureSignedBy;
    use primitives::IdentityId;
    use sp_core::H256;
    use sp_runtime::{
        testing::Header,
        traits::{BlakeTwo256, Block as BlockT, IdentityLookup, Verify},
        AnySignature, BuildStorage, Perbill,
    };
    use test_client::{self, AccountKeyring};

    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: u32 = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::one();
    }

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Call = ();
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<Self::AccountId>;
        type Header = Header;
        type Event = ();
        type BlockHashCount = BlockHashCount;
        type MaximumBlockWeight = MaximumBlockWeight;
        type MaximumBlockLength = MaximumBlockLength;
        type AvailableBlockRatio = AvailableBlockRatio;
        type Version = ();
        type ModuleToIndex = ();
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
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type CreationFee = CreationFee;
        type Identity = crate::identity::Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance2> for Test {
        type Event = ();
        type AddOrigin = EnsureSignedBy<One, AccountId>;
        type RemoveOrigin = EnsureSignedBy<Two, AccountId>;
        type SwapOrigin = EnsureSignedBy<Three, AccountId>;
        type ResetOrigin = EnsureSignedBy<Four, AccountId>;
        type MembershipInitialized = ();
        type MembershipChanged = ();
    }

    impl identity::Trait for Test {
        type Event = ();
        type Proposal = Call;
        type AcceptTransferTarget = Test;
        type AddSignerMultiSigTarget = Test;
        type KYCServiceProviders = Test;
    }

    impl crate::group::GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!()
        }
        fn is_member(_did: &IdentityId) -> bool {
            unimplemented!()
        }
    }

    impl crate::asset::AcceptTransfer for Test {
        fn accept_ticker_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
        fn accept_token_ownership_transfer(_: IdentityId, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl crate::multisig::AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signer, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    type Identity = identity::Module<Test>;
    type AccountId = <AnySignature as Verify>::Signer;

    parameter_types! {
        pub const CommitteeOrigin: AccountId = AccountId::from(AccountKeyring::Alice);
    }

    impl Trait<Instance1> for Test {
        type Origin = Origin;
        type Proposal = Call;
        type CommitteeOrigin = EnsureSignedBy<CommitteeOrigin, AccountId>;
        type Event = ();
    }

    impl Trait for Test {
        type Origin = Origin;
        type Proposal = Call;
        type CommitteeOrigin = EnsureSignedBy<CommitteeOrigin, AccountId>;
        type Event = ();
    }

    pub type Block = sp_runtime::generic::Block<Header, UncheckedExtrinsic>;
    pub type UncheckedExtrinsic = sp_runtime::generic::UncheckedExtrinsic<u32, u64, Call, ()>;

    frame_support::construct_runtime!(
		pub enum Test where
			Block = Block,
			NodeBlock = Block,
			UncheckedExtrinsic = UncheckedExtrinsic
		{
			System: frame_system::{Module, Call, Event},
			Committee: committee::<Instance1>::{Module, Call, Event<T>, Origin<T>, Config<T>},
			DefaultCommittee: committee::{Module, Call, Event<T>, Origin<T>, Config<T>},
		}
	);

    fn make_ext() -> sp_io::TestExternalities {
        GenesisConfig {
            committee_Instance1: Some(committee::GenesisConfig {
                members: vec![
                    IdentityId::from(1),
                    IdentityId::from(2),
                    IdentityId::from(3),
                ],
                vote_threshold: (ProportionMatch::AtLeast, 1, 1),
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
                    IdentityId::from(3)
                ]
            );
            assert_eq!(Committee::proposals(), Vec::<H256>::new());
        });
    }

    fn make_proposal(value: u64) -> Call {
        Call::System(frame_system::Call::remark(value.encode()))
    }

    fn make_account(
        account_id: &AccountId,
    ) -> StdResult<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn propose_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            Committee::set_members(Origin::ROOT, vec![alice_did]).unwrap();

            let proposal = make_proposal(42);
            let hash = proposal.blake2_256().into();
            assert_ok!(Committee::propose(
                alice_signer.clone(),
                alice_did,
                Box::new(proposal.clone())
            ));
            assert_eq!(Committee::proposals(), vec![hash]);
            assert_eq!(Committee::proposal_of(&hash), Some(proposal));
            assert_eq!(
                Committee::voting(&hash),
                Some(PolymeshVotes {
                    index: 0,
                    ayes: vec![alice_did],
                    nays: vec![]
                })
            );
        });
    }

    #[test]
    fn preventing_motions_from_non_members_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            let proposal = make_proposal(42);
            assert_noop!(
                Committee::propose(alice_signer.clone(), alice_did, Box::new(proposal.clone())),
                "proposer is not a member"
            );
        });
    }

    #[test]
    fn preventing_voting_from_non_members_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (bob_signer, bob_did) = make_account(&bob_acc).unwrap();

            Committee::set_members(Origin::ROOT, vec![alice_did]).unwrap();

            let proposal = make_proposal(42);
            let hash: H256 = proposal.blake2_256().into();
            assert_ok!(Committee::propose(
                alice_signer.clone(),
                alice_did,
                Box::new(proposal.clone())
            ));
            assert_noop!(
                Committee::vote(bob_signer, bob_did, hash.clone(), 0, true),
                "voter is not a member"
            );
        });
    }

    #[test]
    fn motions_ignoring_bad_index_vote_works() {
        make_ext().execute_with(|| {
            System::set_block_number(3);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (bob_signer, bob_did) = make_account(&bob_acc).unwrap();

            Committee::set_members(Origin::ROOT, vec![alice_did, bob_did]).unwrap();

            let proposal = make_proposal(42);
            let hash: H256 = proposal.blake2_256().into();
            assert_ok!(Committee::propose(
                alice_signer.clone(),
                alice_did,
                Box::new(proposal.clone())
            ));
            assert_noop!(
                Committee::vote(bob_signer, bob_did, hash.clone(), 1, true),
                "mismatched index"
            );
        });
    }

    #[test]
    fn motions_revoting_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (_bob_signer, bob_did) = make_account(&bob_acc).unwrap();

            let charlie_acc = AccountId::from(AccountKeyring::Charlie);
            let (_charlie_signer, charlie_did) = make_account(&charlie_acc).unwrap();

            Committee::set_members(Origin::ROOT, vec![alice_did, bob_did, charlie_did]).unwrap();

            let proposal = make_proposal(42);
            let hash: H256 = proposal.blake2_256().into();
            assert_ok!(Committee::propose(
                alice_signer.clone(),
                alice_did,
                Box::new(proposal.clone())
            ));
            assert_eq!(
                Committee::voting(&hash),
                Some(PolymeshVotes {
                    index: 0,
                    ayes: vec![alice_did],
                    nays: vec![]
                })
            );
            assert_noop!(
                Committee::vote(alice_signer.clone(), alice_did, hash.clone(), 0, true),
                Error::<Test, Instance1>::DuplicateVote
            );
            assert_ok!(Committee::vote(
                alice_signer.clone(),
                alice_did,
                hash.clone(),
                0,
                false
            ));
            assert_eq!(
                Committee::voting(&hash),
                Some(PolymeshVotes {
                    index: 0,
                    ayes: vec![],
                    nays: vec![alice_did]
                })
            );
            assert_noop!(
                Committee::vote(alice_signer.clone(), alice_did, hash.clone(), 0, false),
                Error::<Test, Instance1>::DuplicateVote
            );
        });
    }

    #[test]
    fn voting_works() {
        make_ext().execute_with(|| {
            System::set_block_number(1);

            let alice_acc = AccountId::from(AccountKeyring::Alice);
            let (_alice_signer, alice_did) = make_account(&alice_acc).unwrap();

            let bob_acc = AccountId::from(AccountKeyring::Bob);
            let (bob_signer, bob_did) = make_account(&bob_acc).unwrap();

            let charlie_acc = AccountId::from(AccountKeyring::Charlie);
            let (charlie_signer, charlie_did) = make_account(&charlie_acc).unwrap();

            Committee::set_members(Origin::ROOT, vec![alice_did, bob_did, charlie_did]).unwrap();

            let proposal = make_proposal(69);
            let hash = BlakeTwo256::hash_of(&proposal);
            assert_ok!(Committee::propose(
                charlie_signer.clone(),
                charlie_did,
                Box::new(proposal.clone())
            ));
            assert_ok!(Committee::vote(
                bob_signer.clone(),
                bob_did,
                hash.clone(),
                0,
                false
            ));
            assert_eq!(
                Committee::voting(&hash),
                Some(PolymeshVotes {
                    index: 0,
                    ayes: vec![charlie_did],
                    nays: vec![bob_did]
                })
            );
        });
    }

    #[test]
    fn changing_vote_threshold_works() {
        make_ext().execute_with(|| {
            assert_eq!(
                Committee::vote_threshold(),
                (ProportionMatch::AtLeast, 1, 1)
            );
            assert_ok!(Committee::set_vote_threshold(
                Origin::signed(AccountId::from(AccountKeyring::Alice)),
                ProportionMatch::AtLeast,
                4,
                17
            ));
            assert_eq!(
                Committee::vote_threshold(),
                (ProportionMatch::AtLeast, 4, 17)
            );
        });
    }
}
