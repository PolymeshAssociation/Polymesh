//! # Mips Module
//!
//! MESH Improvement Proposals (MIPs) are proposals (ballots) that can then be proposed and voted on
//! by all MESH token holders. If a ballot passes this community vote it is then passed to the
//! governance council to ratify (or reject).
//! - minimum of 5,000 MESH needs to be staked by the proposer of the ballot
//! in order to create a new ballot.
//! - minimum of 100,000 MESH (quorum) needs to vote in favour of the ballot in order for the
//! ballot to be considered by the governing committee.
//! - ballots run for 1 week
//! - a simple majority is needed to pass the ballot so that it heads for the
//! next stage (governing committee)
//!
//! ## Overview
//!
//! The Mips module provides functions for:
//!
//! - Creating Mesh Improvement Proposals
//! - Voting on Mesh Improvement Proposals
//! - Governance committee to ratify or reject proposals
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `set_min_proposal_deposit` change min deposit to create a proposal
//! - `set_quorum_threshold` change stake required to make a proposal into a referendum
//! - `set_proposal_duration` change duration in blocks for which proposal stays active
//! - `propose` - Token holders can propose a new ballot.
//! - `vote` - Token holders can vote on a ballot.
//! - `kill_proposal` - close a proposal and refund all deposits
//! - `enact_referendum` committee calls to execute a referendum
//!
//! ### Public Functions
//!
//! - `end_block` - Returns details of the token
#![cfg_attr(not(feature = "std"), no_std)]

use codec::{Decode, Encode};
use frame_support::{
    debug, decl_error, decl_event, decl_module, decl_storage,
    dispatch::DispatchResult,
    ensure,
    traits::{Currency, LockableCurrency, ReservableCurrency},
    weights::SimpleDispatchInfo,
    Parameter,
};
use frame_system::{self as system, ensure_signed};
use pallet_mips_rpc_runtime_api::VoteCount;
use polymesh_primitives::{AccountKey, Signatory};
use polymesh_runtime_common::{
    identity::Trait as IdentityTrait,
    protocol_fee::{ChargeProtocolFee, ProtocolOp},
    traits::{governance_group::GovernanceGroupTrait, group::GroupTrait},
    Context,
};
use polymesh_runtime_identity as identity;
use sp_runtime::{
    traits::{CheckedSub, Dispatchable, EnsureOrigin, Zero},
    DispatchError,
};
use sp_std::{convert::TryFrom, prelude::*, vec};

/// Mesh Improvement Proposal index. Used offchain.
pub type MipsIndex = u32;

/// Balance
type BalanceOf<T> =
    <<T as Trait>::Currency as Currency<<T as frame_system::Trait>::AccountId>>::Balance;

/// Represents a proposal
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
pub struct MIP<Proposal> {
    /// The proposal's unique index.
    index: MipsIndex,
    /// The proposal being voted on.
    proposal: Proposal,
    /// The latest state
    state: ProposalState,
}

/// A wrapper for a proposal url.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Url(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for Url {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        Url(v)
    }
}

/// A wrapper for a proposal description.
#[derive(Decode, Encode, Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct MipDescription(pub Vec<u8>);

impl<T: AsRef<[u8]>> From<T> for MipDescription {
    fn from(s: T) -> Self {
        let s = s.as_ref();
        let mut v = Vec::with_capacity(s.len());
        v.extend_from_slice(s);
        MipDescription(v)
    }
}

/// Represents a proposal metadata
#[derive(Encode, Decode, Clone, PartialEq, Eq, Debug)]
pub struct MipsMetadata<AcccountId: Parameter, BlockNumber: Parameter> {
    /// The creator
    pub proposer: AcccountId,
    /// The proposal's unique index.
    pub index: MipsIndex,
    /// When voting will end.
    pub end: BlockNumber,
    /// The proposal url for proposal discussion.
    pub url: Option<Url>,
    /// The proposal description.
    pub description: Option<MipDescription>,
    /// This proposal allows any changes
    /// During Cool-off period, proposal owner can amend any MIP detail or cancel the entire
    pub cool_off_until: BlockNumber,
}

/// For keeping track of proposal being voted on.
#[derive(PartialEq, Eq, Clone, Encode, Decode)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PolymeshVotes<AccountId, Balance> {
    /// The proposal's unique index.
    pub index: MipsIndex,
    /// The current set of voters that approved with their stake.
    pub ayes: Vec<(AccountId, Balance)>,
    /// The current set of voters that rejected with their stake.
    pub nays: Vec<(AccountId, Balance)>,
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum ProposalState {
    /// Proposal is created and either in the cool-down period or open to voting
    Proposed,
    /// Proposal is cancelled by its owner
    Cancelled,
    /// Proposal was killed by the GC
    Killed,
    /// Proposal has moved to referendum stage
    Referendum,
    /// Proposal failed to pass by a community vote
    Rejected,
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum ReferendumState {
    /// Pending GC ratification
    Pending,
    /// Execution of this MIP is scheduled, i.e. it needs to wait its enactment period.
    Scheduled,
    /// Rejected by the GC
    Rejected,
    /// It has been executed, but execution failed.
    Failed,
    /// It has been successfully executed.
    Executed,
}

#[derive(Encode, Decode, Copy, Clone, Eq, PartialEq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub enum ReferendumType {
    /// Referendum pushed by GC (fast-tracked)
    FastTracked,
    /// Referendum created by GC
    Emergency,
    /// Created through a community vote
    Community,
}

impl Default for ProposalState {
    fn default() -> Self {
        ProposalState::Proposed
    }
}

/// Properties of a referendum
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct Referendum<BlockNumber: Default> {
    /// The proposal's unique index.
    pub index: MipsIndex,
    /// Current state of this Referendum.
    pub state: ReferendumState,
    /// The type of the referendum
    pub referendum_type: ReferendumType,
    /// Enactment period.
    pub enactment_period: BlockNumber,
}

/// Information about deposit.
#[derive(Encode, Decode, Clone, PartialEq, Eq, Default)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct DepositInfo<AccountId, Balance>
where
    AccountId: Default,
    Balance: Default,
{
    /// Owner of the deposit.
    pub owner: AccountId,
    /// Amount. It can be updated during the cool off period.
    pub amount: Balance,
}

type Identity<T> = identity::Module<T>;

/// The module's configuration trait.
pub trait Trait: frame_system::Trait + pallet_timestamp::Trait + IdentityTrait {
    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// Origin for proposals.
    type CommitteeOrigin: EnsureOrigin<Self::Origin>;

    /// Origin for enacting a referundum.
    type VotingMajorityOrigin: EnsureOrigin<Self::Origin>;

    /// Committee
    type GovernanceCommittee: GovernanceGroupTrait<<Self as pallet_timestamp::Trait>::Moment>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as Mips {
        /// The minimum amount to be used as a deposit for a public referendum proposal.
        pub MinimumProposalDeposit get(fn min_proposal_deposit) config(): BalanceOf<T>;

        /// Minimum stake a proposal must gather in order to be considered by the committee.
        pub QuorumThreshold get(fn quorum_threshold) config(): BalanceOf<T>;

        /// During Cool-off period, proposal owner can amend any MIP detail or cancel the entire
        /// proposal.
        pub ProposalCoolOffPeriod get(fn proposal_cool_off_period) config(): T::BlockNumber;

        /// How long (in blocks) a ballot runs
        pub ProposalDuration get(fn proposal_duration) config(): T::BlockNumber;

        /// Proposals so far. Index can be used to keep track of MIPs off-chain.
        SequenceIndex: u32;

        /// The metadata of the active proposals.
        pub ProposalMetadata get(fn proposal_meta): Vec<MipsMetadata<T::AccountId, T::BlockNumber>>;

        /// Those who have locked a deposit.
        /// proposal (Index, proposer) -> deposit
        pub Deposits get(fn deposit_of): double_map hasher(twox_64_concat) MipsIndex, hasher(twox_64_concat) T::AccountId => DepositInfo<T::AccountId, BalanceOf<T>>;

        /// Actual proposal for a given index, if it's current.
        /// proposal Index -> proposal
        pub Proposals get(fn proposals): map hasher(twox_64_concat) MipsIndex=> Option<MIP<T::Proposal>>;

        /// PolymeshVotes on a given proposal, if it is ongoing.
        /// proposal Index -> vote count
        pub Voting get(fn voting): map hasher(twox_64_concat) MipsIndex => Option<PolymeshVotes<T::AccountId, BalanceOf<T>>>;

        /// Proposals that have met the quorum threshold to be put forward to a governance committee
        /// proposal index -> proposal
        pub Referendums get(fn referendums): map hasher(twox_64_concat) MipsIndex => Option<Referendum<T::BlockNumber>>;

        /// List of Indexes of current scheduled referendums.
        /// block number -> Mip Index
        pub ScheduledReferendumsAt get(fn scheduled_referendums_at): map hasher(twox_64_concat) T::BlockNumber => Vec<MipsIndex>;

        /// Default enactment period that will be use after a proposal is accepted by GC.
        pub DefaultEnactmentPeriod get(fn default_enactment_period) config(): T::BlockNumber;
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        <T as frame_system::Trait>::AccountId,
        <T as frame_system::Trait>::BlockNumber,
    {
        /// A Mesh Improvement Proposal was made with a `Balance` stake
        ProposalCreated(AccountId, MipsIndex, Balance),
        /// A Mesh Improvement Proposal was amended with a possible to the deposit
        /// bool is +ve when bond is added, -ve when removed
        ProposalAmended(AccountId, MipsIndex, bool, Balance),
        /// `AccountId` voted `bool` on the proposal referenced by `Index`
        Voted(AccountId, MipsIndex, bool, Balance),
        /// Proposal has been closed
        ProposalClosed(MipsIndex, ProposalState),
        /// Referendum created for proposal.
        ReferendumCreated(MipsIndex, ReferendumType),
        /// Referendum execution has been scheduled at specific block.
        ReferendumScheduled(MipsIndex, BlockNumber, BlockNumber),
        /// Proposal was dispatched, bool indicates success or failure
        ReferendumExecuted(MipsIndex, bool),
        /// Proposal was rejected by the GC
        ReferendumRejected(MipsIndex),
        /// Default enactment period (in blocks) has been changed.
        /// (new period, old period)
        DefaultEnactmentPeriodChanged(BlockNumber, BlockNumber),
        /// Minimum deposit amount modified
        /// (old amount, new amount)
        MinimumProposalDepositChanged(Balance, Balance),
        /// Quorum threshold changed
        /// (old value, new value)
        QuorumThresholdChanged(Balance, Balance),
        /// Proposal duration changed
        /// (old value, new value)
        ProposalDurationChanged(BlockNumber, BlockNumber),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// Incorrect origin
        BadOrigin,
        /// Proposer can't afford to lock minimum deposit
        InsufficientDeposit,
        /// when voter vote gain
        DuplicateVote,
        /// Duplicate proposal.
        DuplicateProposal,
        /// The proposal does not exist.
        NoSuchProposal,
        /// Mismatched proposal index.
        MismatchedProposalIndex,
        /// Not part of governance committee.
        NotACommitteeMember,
        /// After Cool-off period, proposals are not cancelable.
        ProposalOnCoolOffPeriod,
        /// Proposal is immutable after cool-off period.
        ProposalIsImmutable,
        /// Referendum is still on its enactment period.
        ReferendumOnEnactmentPeriod,
        /// Referendum is immutable.
        ReferendumIsImmutable,
        /// When a block number is less than current block number.
        InvalidFutureBlockNumber,
    }
}

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        type Error = Error<T>;

        fn deposit_event() = default;

        /// Change the minimum proposal deposit amount required to start a proposal. Only Governance
        /// committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `deposit` the new min deposit required to start a proposal
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_min_proposal_deposit(origin, deposit: BalanceOf<T>) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            Self::deposit_event(RawEvent::MinimumProposalDepositChanged(Self::min_proposal_deposit(), deposit));
            <MinimumProposalDeposit<T>>::put(deposit);
        }

        /// Change the quorum threshold amount. This is the amount which a proposal must gather so
        /// as to be considered by a committee. Only Governance committee is allowed to change
        /// this value.
        ///
        /// # Arguments
        /// * `threshold` the new quorum threshold amount value
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_quorum_threshold(origin, threshold: BalanceOf<T>) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            Self::deposit_event(RawEvent::MinimumProposalDepositChanged(Self::quorum_threshold(), threshold));
            <QuorumThreshold<T>>::put(threshold);
        }

        /// Change the proposal duration value. This is the number of blocks for which votes are
        /// accepted on a proposal. Only Governance committee is allowed to change this value.
        ///
        /// # Arguments
        /// * `duration` proposal duration in blocks
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_proposal_duration(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            Self::deposit_event(RawEvent::ProposalDurationChanged(Self::proposal_duration(), duration));
            <ProposalDuration<T>>::put(duration);
        }

        /// Change the default enact period.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_default_enact_period(origin, duration: T::BlockNumber) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            let previous_duration = <DefaultEnactmentPeriod<T>>::get();
            <DefaultEnactmentPeriod<T>>::put(duration);
            Self::deposit_event(RawEvent::DefaultEnactmentPeriodChanged(duration, previous_duration));
        }

        /// A network member creates a Mesh Improvement Proposal by submitting a dispatchable which
        /// changes the network in someway. A minimum deposit is required to open a new proposal.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `deposit` minimum deposit value
        /// * `url` a link to a website for proposal discussion
        #[weight = SimpleDispatchInfo::FixedNormal(5_000_000)]
        pub fn propose(
            origin,
            proposal: Box<T::Proposal>,
            deposit: BalanceOf<T>,
            url: Option<Url>,
            description: Option<MipDescription>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let proposer_key = AccountKey::try_from(proposer.encode())?;
            let signer = Signatory::from(proposer_key);

            // Pre conditions: caller must have min balance
            ensure!(
                deposit >= Self::min_proposal_deposit(),
                Error::<T>::InsufficientDeposit
            );

            // Reserve the minimum deposit
            <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;
            <T as IdentityTrait>::ProtocolFee::charge_fee(
                &signer,
                ProtocolOp::MipsPropose
            )?;
            let index = Self::next_index();

            let curr_block_number = <system::Module<T>>::block_number();
            let cool_off_until = curr_block_number + Self::proposal_cool_off_period();
            let end = cool_off_until + Self::proposal_duration();
            let proposal_meta = MipsMetadata {
                proposer: proposer.clone(),
                index,
                end,
                url,
                description,
                cool_off_until,
            };
            <ProposalMetadata<T>>::mutate(|metadata| metadata.push(proposal_meta));

            let deposit_info = DepositInfo {
                owner: proposer.clone(),
                amount: deposit
            };
            <Deposits<T>>::insert(index, &proposer, deposit_info);

            let mip = MIP {
                index,
                proposal: *proposal,
                state: ProposalState::Proposed,
            };
            <Proposals<T>>::insert(index, mip);

            let vote = PolymeshVotes {
                index,
                ayes: vec![(proposer.clone(), deposit)],
                nays: vec![],
            };
            <Voting<T>>::insert(index, vote);

            Self::deposit_event(RawEvent::ProposalCreated(proposer, index, deposit));
            Ok(())
        }

        /// It amends the `url` and the `description` of the proposal with index `index`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsImmutable`: A proposals is mutable only during its cool off period.
        ///
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn amend_proposal(
                origin,
                index: MipsIndex,
                url: Option<Url>,
                description: Option<MipDescription>
                ) -> DispatchResult {
            // 0. Initial info.
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can cancel it.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be cancelled *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsImmutable);

            // 3. Update proposal metadata.
            <ProposalMetadata<T>>::mutate( |metas| {
                let meta = metas.iter_mut().find( |meta| meta.index == index);
                if let Some(meta) = meta {
                    meta.url = url;
                    meta.description = description;
                }
            });
            Self::deposit_event(RawEvent::ProposalAmended(proposer, index, true, Zero::zero()));

            Ok(())
        }

        /// It cancels the proposal of the index `index`.
        ///
        /// Proposals can be cancelled only during its _cool-off period.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn cancel_proposal(origin, index: MipsIndex) -> DispatchResult {
            // 0. Initial info.
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can cancel it.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be cancelled *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsImmutable);
            Self::update_proposal_state(index, ProposalState::Cancelled);
            Self::refund_proposal(index);
            // 3. Close that proposal.
            Self::close_proposal(index);

            Ok(())
        }

        /// Id bonds an additional deposit to proposal with index `index`.
        /// That amount is added to the current deposit.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can bond an additional deposit.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn bond_additional_deposit(origin,
            index: MipsIndex,
            additional_deposit: BalanceOf<T>
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can add additional deposit.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be amended *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsImmutable);

            // 3. Reserve extra deposit & update deposit info for this proposal
            <T as Trait>::Currency::reserve(&proposer, additional_deposit)
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            <Deposits<T>>::mutate(
                index,
                &proposer,
                |depo_info| depo_info.amount += additional_deposit);

            Self::deposit_event(RawEvent::ProposalAmended(proposer, index, true, additional_deposit));

            Ok(())
        }

        /// It unbonds any amount from the deposit of the proposal with index `index`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can release part of the deposit.
        /// * `ProposalIsImmutable`: A Proposal is mutable only during its cool off period.
        /// * `InsufficientDeposit`: If the final deposit will be less that the minimum deposit for
        /// a proposal.
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn unbond_deposit(origin,
            index: MipsIndex,
            amount: BalanceOf<T>
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can cancel it.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be cancelled *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsImmutable);

            // 3. Double-check that `amount` is valid.
            let mut depo_info = <Deposits<T>>::get(index, &proposer);
            let new_deposit = depo_info.amount.checked_sub(&amount)
                    .ok_or_else(|| Error::<T>::InsufficientDeposit)?;
            ensure!(
                new_deposit >= Self::min_proposal_deposit(),
                Error::<T>::InsufficientDeposit);
            let diff_amount = depo_info.amount - new_deposit;
            depo_info.amount = new_deposit;

            // 3.1. Unreserve and update deposit info.
            <T as Trait>::Currency::unreserve(&depo_info.owner, diff_amount);
            <Deposits<T>>::insert(index, &proposer, depo_info);
            Self::deposit_event(RawEvent::ProposalAmended(proposer, index, false, amount));
            Ok(())
        }

        /// A network member can vote on any Mesh Improvement Proposal by selecting the index that
        /// corresponds ot the dispatchable action and vote with some balance.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `index` proposal index
        /// * `aye_or_nay` a bool representing for or against vote
        /// * `deposit` minimum deposit value
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn vote(origin, index: MipsIndex, aye_or_nay: bool, deposit: BalanceOf<T>) {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // No one should be able to vote during the proposal cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);

            let mut voting = Self::voting(index).ok_or(Error::<T>::NoSuchProposal)?;
            ensure!(voting.index == index, Error::<T>::MismatchedProposalIndex);

            let position_yes = voting.ayes.iter().position(|(a, _)| a == &proposer);
            let position_no = voting.nays.iter().position(|(a, _)| a == &proposer);

            if position_yes.is_none() && position_no.is_none()  {
                if aye_or_nay {
                    voting.ayes.push((proposer.clone(), deposit));
                } else {
                    voting.nays.push((proposer.clone(), deposit));
                }

                // Reserve the deposit
                <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;

                let depo_info = DepositInfo {
                    owner: proposer.clone(),
                    amount: deposit,
                };
                <Deposits<T>>::insert(index, &proposer, depo_info);

                <Voting<T>>::remove(index);
                <Voting<T>>::insert(index, voting);
                Self::deposit_event(RawEvent::Voted(proposer, index, aye_or_nay, deposit));
            } else {
                return Err(Error::<T>::DuplicateVote.into())
            }
        }

        /// An emergency stop measure to kill a proposal. Governance committee can kill
        /// a proposal at any time.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn kill_proposal(origin, index: MipsIndex) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

            ensure!( <Proposals<T>>::contains_key(index), Error::<T>::NoSuchProposal);
            Self::update_proposal_state(index, ProposalState::Killed);
            Self::refund_proposal(index);
            Self::close_proposal(index);
        }

        /// Any governance committee member can fast track a proposal and turn it into a referendum
        /// that will be voted on by the committee.
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        pub fn fast_track_proposal(origin, index: MipsIndex) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            ensure!(Self::proposals(index).is_some(), Error::<T>::MismatchedProposalIndex);

            Self::create_referendum(
                index,
                ReferendumState::Pending,
                ReferendumType::FastTracked,
            );

            Self::update_proposal_state(index, ProposalState::Referendum);

            // Update proposal metadata so we don't re-execute it later
            // TODO: Improve data structures to avoid unbounded loop
            <ProposalMetadata<T>>::mutate( |metas| {
                let meta = metas.iter_mut().find( |meta| meta.index == index);
                if let Some(meta) = meta {
                    meta.end = Zero::zero();
                }
            });

            Self::refund_proposal(index);
            Ok(())
        }

        /// Governance committee can make a proposal that automatically becomes a referendum on
        /// which the committee can vote on.
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        pub fn emergency_referendum(
            origin,
            proposal: Box<T::Proposal>,
            url: Option<Url>,
            description: Option<MipDescription>,
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let proposer_key = AccountKey::try_from(proposer.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&proposer_key)?;

            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            let index = Self::next_index();
            let mip = MIP {
                index,
                proposal: *proposal,
                state: ProposalState::Referendum,
            };
            <Proposals<T>>::insert(index, mip);

            let proposal_meta = MipsMetadata {
                proposer: proposer.clone(),
                index,
                end: Zero::zero(),
                url,
                description,
                cool_off_until: Zero::zero(),
            };
            <ProposalMetadata<T>>::mutate(|metadata| metadata.push(proposal_meta));
            // TODO: Improve data structures to avoid unbounded loop

            Self::create_referendum(
                index,
                ReferendumState::Pending,
                ReferendumType::Emergency,
            );
            Ok(())
        }

        /// Moves a referendum instance into dispatch queue.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn enact_referendum(origin, index: MipsIndex) -> DispatchResult {
            T::VotingMajorityOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            Self::prepare_to_dispatch(index)
        }

        /// Moves a referendum instance into rejected state.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn reject_referendum(origin, index: MipsIndex) -> DispatchResult {
            T::VotingMajorityOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;
            // Update state of Referendum
            Self::update_referendum_state(index, ReferendumState::Rejected);
            // Close proposal
            Self::close_proposal(index);
            Self::deposit_event(RawEvent::ReferendumRejected(index));
            Ok(())
        }

        /// It updates the enactment period of a specific referendum.
        ///
        /// # Arguments
        /// * `until`, It defines the future block where the enactment period will finished.  A
        /// `None` value means that enactment period is going to finish in the next block.
        ///
        /// # Errors
        /// * `BadOrigin`, Only the release coordinator can update the enactment period.
        /// * ``,
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn set_referendum_enactment_period(origin, index: MipsIndex, until: Option<T::BlockNumber>) -> DispatchResult {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let id = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            // 1. Only release coordinator
            ensure!(
                Some(id) == T::GovernanceCommittee::release_coordinator(),
                Error::<T>::BadOrigin);

            // 2. New value should be valid block number.
            let next_block = <system::Module<T>>::block_number() + 1.into();
            let new_until = until.unwrap_or(next_block);
            ensure!( new_until >= next_block, Error::<T>::InvalidFutureBlockNumber);

            // 2. Valid referendum: check index & state == Scheduled
            let referendum = Self::referendums(index)
                .ok_or_else(|| Error::<T>::MismatchedProposalIndex)?;
            ensure!( referendum.state == ReferendumState::Scheduled, Error::<T>::ReferendumIsImmutable);

            // 3. Update enactment period.
            // 3.1 Update referendum.
            let old_until = referendum.enactment_period;

            <Referendums<T>>::mutate( index, |referendum| {
                if let Some(ref mut referendum) = referendum {
                    referendum.enactment_period = new_until;
                }
            });

            // 3.1. Re-schedule it
            <ScheduledReferendumsAt<T>>::mutate( old_until, |indexes| indexes.retain( |i| *i != index));
            <ScheduledReferendumsAt<T>>::mutate( new_until, |indexes| indexes.push( index));

            Self::deposit_event(RawEvent::ReferendumScheduled(index, old_until, new_until));
            Ok(())
        }

        /// When constructing a block check if it's time for a ballot to end. If ballot ends,
        /// proceed to ratification process.
        fn on_initialize(n: T::BlockNumber) {
            if let Err(e) = Self::end_block(n) {
                sp_runtime::print(e);
            }
        }

    }
}

impl<T: Trait> Module<T> {
    /// Retrieve all proposals that need to be closed as of block `n`.
    pub fn proposals_maturing_at(n: T::BlockNumber) -> Vec<MipsIndex> {
        Self::proposal_meta()
            .into_iter()
            .filter(|meta| meta.end == n)
            .map(|meta| meta.index)
            .collect()
    }

    // Private functions

    /// Runs the following procedure:
    /// 1. Find all proposals that need to end as of this block and close voting
    /// 2. Tally votes
    /// 3. Submit any proposals that meet the quorum threshold, to the governance committee
    /// 4. Automatically execute any referendum
    pub fn end_block(block_number: T::BlockNumber) -> DispatchResult {
        // Find all matured proposals...
        Self::proposals_maturing_at(block_number)
            .into_iter()
            .for_each(|index| {
                // Tally votes and create referendums
                let result = Self::tally_votes(index);

                // And update and / or close proposals
                if result {
                    Self::update_proposal_state(index, ProposalState::Referendum);
                } else {
                    Self::update_proposal_state(index, ProposalState::Rejected);
                    Self::refund_proposal(index);
                    Self::close_proposal(index);
                }
            });

        // Execute automatically referendums after its enactment period.
        let referendum_indexes = <ScheduledReferendumsAt<T>>::take(block_number);
        referendum_indexes
            .into_iter()
            .for_each(|index| Self::execute_referendum(index));

        Ok(())
    }

    /// Summarize voting and create referendums if proposals meet or exceed quorum threshold
    fn tally_votes(index: MipsIndex) -> bool {
        if let Some(voting) = <Voting<T>>::get(index) {
            let aye_stake = voting
                .ayes
                .iter()
                .fold(<BalanceOf<T>>::zero(), |acc, ayes| acc + ayes.1);

            let nay_stake = voting
                .nays
                .iter()
                .fold(<BalanceOf<T>>::zero(), |acc, nays| acc + nays.1);

            // 1. Ayes staked must be more than nays staked (simple majority)
            // 2. Ayes staked are more than the minimum quorum threshold
            if aye_stake > nay_stake && aye_stake >= Self::quorum_threshold() {
                Self::create_referendum(index, ReferendumState::Pending, ReferendumType::Community);
                Self::refund_proposal(index);
                return true;
            }
        }
        false
    }

    /// Create a referendum object from a proposal. If governance committee is composed of less
    /// than 2 members, enact it immediately. Otherwise, committee votes on this referendum and
    /// decides whether it should be enacted.
    fn create_referendum(
        index: MipsIndex,
        state: ReferendumState,
        referendum_type: ReferendumType,
    ) {
        let enactment_period: T::BlockNumber = 0.into();
        let referendum = Referendum {
            index,
            state,
            referendum_type,
            enactment_period,
        };
        <Referendums<T>>::insert(index, referendum);

        Self::deposit_event(RawEvent::ReferendumCreated(index, referendum_type));
    }

    /// Refunds any tokens used to vote or bond a proposal
    fn refund_proposal(index: MipsIndex) {
        if <Voting<T>>::get(index).is_some() {
            Self::unreserve_deposits(index);
        }
    }

    /// Close a proposal. Voting ceases and proposal is removed from storage.
    fn close_proposal(index: MipsIndex) {
        let proposal = Self::proposals(index);

        if let Some(proposal) = proposal {
            <Voting<T>>::remove(index);
            <ProposalMetadata<T>>::mutate(|metadata| metadata.retain(|m| m.index != index));

            Self::deposit_event(RawEvent::ProposalClosed(index, proposal.state));
        }
    }

    /// It returns back each deposit to their owners for an specific `index`.
    fn unreserve_deposits(index: MipsIndex) {
        <Deposits<T>>::iter_prefix(index).for_each(|depo_info| {
            let _ = <T as Trait>::Currency::unreserve(&depo_info.owner, depo_info.amount);
        });
        <Deposits<T>>::remove_prefix(index);
    }

    fn prepare_to_dispatch(index: MipsIndex) -> DispatchResult {
        ensure!(
            <Referendums<T>>::contains_key(index),
            Error::<T>::MismatchedProposalIndex
        );

        // Set the default enactment period and move it to `Scheduled`
        let curr_block_number = <system::Module<T>>::block_number();
        let enactment_period = curr_block_number + Self::default_enactment_period();

        <Referendums<T>>::mutate(index, |referendum| {
            if let Some(ref mut referendum) = referendum {
                referendum.enactment_period = enactment_period;
                referendum.state = ReferendumState::Scheduled;
            }
        });
        <ScheduledReferendumsAt<T>>::mutate(enactment_period, |indexes| indexes.push(index));

        Self::deposit_event(RawEvent::ReferendumScheduled(
            index,
            Zero::zero(),
            enactment_period,
        ));
        Ok(())
    }

    fn execute_referendum(index: MipsIndex) {
        if let Some(proposal) = Self::proposals(index) {
            match proposal.proposal.dispatch(system::RawOrigin::Root.into()) {
                Ok(_) => {
                    Self::update_referendum_state(index, ReferendumState::Executed);
                    Self::close_proposal(index);
                    Self::deposit_event(RawEvent::ReferendumExecuted(index, true));
                }
                Err(e) => {
                    Self::update_referendum_state(index, ReferendumState::Failed);
                    Self::close_proposal(index);
                    Self::deposit_event(RawEvent::ReferendumExecuted(index, false));
                    debug::error!("Referendum {}, its execution fails: {:?}", index, e);
                }
            }
        }
    }

    /// It returns the proposal metadata of proposal with index `index` or
    /// a `MismatchedProposalIndex` error.
    fn proposal_meta_by_index(
        index: MipsIndex,
    ) -> Result<MipsMetadata<T::AccountId, T::BlockNumber>, DispatchError> {
        Self::proposal_meta()
            .into_iter()
            .find(|meta| meta.index == index)
            .ok_or(Error::<T>::MismatchedProposalIndex.into())
    }

    fn update_proposal_state(index: MipsIndex, new_state: ProposalState) {
        <Proposals<T>>::mutate(index, |proposal| {
            if let Some(ref mut proposal) = proposal {
                proposal.state = new_state;
            }
        });
    }

    fn update_referendum_state(index: MipsIndex, new_state: ReferendumState) {
        <Referendums<T>>::mutate(index, |referendum| {
            if let Some(ref mut referendum) = referendum {
                referendum.state = new_state;
            }
        });
    }
}

impl<T: Trait> Module<T> {
    /// Retrieve votes for a proposal represented by MipsIndex `index`.
    pub fn get_votes(index: MipsIndex) -> VoteCount<BalanceOf<T>>
    where
        T: Send + Sync,
        BalanceOf<T>: Send + Sync,
    {
        if let Some(voting) = <Voting<T>>::get(index) {
            let aye_stake = voting
                .ayes
                .iter()
                .fold(<BalanceOf<T>>::zero(), |acc, ayes| acc + ayes.1);

            let nay_stake = voting
                .nays
                .iter()
                .fold(<BalanceOf<T>>::zero(), |acc, nays| acc + nays.1);

            VoteCount::Success {
                ayes: aye_stake,
                nays: nay_stake,
            }
        } else {
            VoteCount::ProposalNotFound
        }
    }

    /// Retrieve proposals made by `address`.
    pub fn proposed_by(address: T::AccountId) -> Vec<MipsIndex> {
        Self::proposal_meta()
            .into_iter()
            .filter(|meta| meta.proposer == address)
            .map(|meta| meta.index)
            .collect()
    }

    /// Retrieve proposals `address` voted on
    pub fn voted_on(address: T::AccountId) -> Vec<MipsIndex> {
        let mut indices = Vec::new();
        for meta in Self::proposal_meta().into_iter() {
            if let Some(votes) = Self::voting(&meta.index) {
                if votes.ayes.iter().any(|(a, _)| a == &address)
                    || votes.nays.iter().any(|(a, _)| a == &address)
                {
                    indices.push(votes.index);
                }
            }
        }
        indices
    }

    /// It generates the next index for proposals and referendums.
    fn next_index() -> u32 {
        let index = <SequenceIndex>::get();
        <SequenceIndex>::put(index + 1);

        index
    }
}
