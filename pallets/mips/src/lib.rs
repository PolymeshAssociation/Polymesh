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
    decl_error, decl_event, decl_module, decl_storage,
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
    traits::group::GroupTrait,
    Context,
};
use polymesh_runtime_identity as identity;
use sp_runtime::{
    traits::{CheckedSub, Dispatchable, EnsureOrigin, Hash, Zero},
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
pub struct MipsMetadata<AcccountId: Parameter, BlockNumber: Parameter, Hash: Parameter> {
    /// The creator
    pub proposer: AcccountId,
    /// The proposal's unique index.
    pub index: MipsIndex,
    /// When voting will end.
    pub end: BlockNumber,
    /// The proposal being voted on.
    pub proposal_hash: Hash,
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
pub enum MipsPriority {
    /// A proposal made by the committee for e.g.,
    High,
    /// By default all proposals have a normal priority
    Normal,
}

impl Default for MipsPriority {
    fn default() -> Self {
        MipsPriority::Normal
    }
}

/// Properties of a referendum
#[derive(Encode, Decode, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PolymeshReferendumInfo<Hash: Parameter> {
    /// The proposal's unique index.
    pub index: MipsIndex,
    /// Priority.
    pub priority: MipsPriority,
    /// The proposal being voted on.
    pub proposal_hash: Hash,
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
    type GovernanceCommittee: GroupTrait<<Self as pallet_timestamp::Trait>::Moment>;

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
        pub ProposalCount get(fn proposal_count): u32;

        /// The hashes of the active proposals.
        pub ProposalMetadata get(fn proposal_meta): Vec<MipsMetadata<T::AccountId, T::BlockNumber, T::Hash>>;

        /// Those who have locked a deposit.
        /// proposal (hash, proposer) -> deposit
        pub Deposits get(fn deposit_of): double_map hasher(twox_64_concat) T::Hash, hasher(twox_64_concat) T::AccountId => DepositInfo<T::AccountId, BalanceOf<T>>;

        /// Actual proposal for a given hash, if it's current.
        /// proposal hash -> proposal
        pub Proposals get(fn proposals): map hasher(twox_64_concat) T::Hash => Option<MIP<T::Proposal>>;

        /// Lookup proposal hash by a proposal's index
        /// MIP index -> proposal hash
        pub ProposalByIndex get(fn proposal_by_index): map hasher(twox_64_concat) MipsIndex => T::Hash;

        /// PolymeshVotes on a given proposal, if it is ongoing.
        /// proposal hash -> vote count
        pub Voting get(fn voting): map hasher(twox_64_concat) T::Hash => Option<PolymeshVotes<T::AccountId, BalanceOf<T>>>;

        /// Active referendums.
        pub ReferendumMetadata get(fn referendum_meta): Vec<PolymeshReferendumInfo<T::Hash>>;

        /// Proposals that have met the quorum threshold to be put forward to a governance committee
        /// proposal hash -> proposal
        pub Referendums get(fn referendums): map hasher(twox_64_concat) T::Hash => Option<T::Proposal>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        <T as frame_system::Trait>::Hash,
        <T as frame_system::Trait>::AccountId,
    {
        /// A Mesh Improvement Proposal was made with a `Balance` stake
        Proposed(AccountId, Balance, MipsIndex, Hash),
        /// `AccountId` voted `bool` on the proposal referenced by `Hash`
        Voted(AccountId, MipsIndex, Hash, bool),
        /// Proposal referenced by `Hash` has been closed
        ProposalClosed(MipsIndex, Hash),
        /// Proposal referenced by `Hash` has been closed
        ProposalFastTracked(MipsIndex, Hash),
        /// Referendum created for proposal referenced by `Hash`
        ReferendumCreated(MipsIndex, MipsPriority, Hash),
        /// Proposal referenced by `Hash` was dispatched with the result `bool`
        ReferendumEnacted(Hash, bool),
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
        /// Proposal is inmutable after cool-off period.
        ProposalIsInmutable,
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
            <ProposalDuration<T>>::put(duration);
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
            let proposal_hash = T::Hashing::hash_of(&proposal);

            // Pre conditions: caller must have min balance
            ensure!(
                deposit >= Self::min_proposal_deposit(),
                Error::<T>::InsufficientDeposit
            );
            // Proposal must be new
            ensure!(
                !<Proposals<T>>::contains_key(proposal_hash),
                Error::<T>::DuplicateProposal
            );

            // Reserve the minimum deposit
            <T as Trait>::Currency::reserve(&proposer, deposit).map_err(|_| Error::<T>::InsufficientDeposit)?;
            <T as IdentityTrait>::ProtocolFee::charge_fee(
                &signer,
                ProtocolOp::MipsPropose
            )?;
            let index = Self::proposal_count();
            <ProposalCount>::mutate(|i| *i += 1);

            let curr_block_number = <system::Module<T>>::block_number();
            let cool_off_until = curr_block_number + Self::proposal_cool_off_period();
            let end = cool_off_until + Self::proposal_duration();
            let proposal_meta = MipsMetadata {
                proposer: proposer.clone(),
                index,
                cool_off_until,
                end,
                proposal_hash,
                url,
                description,
            };
            <ProposalMetadata<T>>::mutate(|metadata| metadata.push(proposal_meta));

            let deposit_info = DepositInfo {
                owner: proposer.clone(),
                amount: deposit
            };
            <Deposits<T>>::insert(&proposal_hash, &proposer, deposit_info);

            let mip = MIP {
                index,
                proposal: *proposal,
            };
            <Proposals<T>>::insert(proposal_hash, mip);
            <ProposalByIndex<T>>::insert(index, proposal_hash);

            let vote = PolymeshVotes {
                index,
                ayes: vec![(proposer.clone(), deposit)],
                nays: vec![],
            };
            <Voting<T>>::insert(proposal_hash, vote);

            Self::deposit_event(RawEvent::Proposed(proposer, deposit, index, proposal_hash));
            Ok(())
        }

        /// It amends the `url` and the `description` of the proposal with index `index`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsInmutable`: A proposals is mutable only during its cool off period.
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
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsInmutable);

            // 3. Update proposal metadata.
            <ProposalMetadata<T>>::mutate( |metas| {
                let meta = metas.iter_mut().find( |meta| meta.index == index);
                if let Some(meta) = meta {
                    meta.url = url;
                    meta.description = description;
                }
            });

            Ok(())
        }

        /// It cancels the proposal of the index `index`.
        ///
        /// Proposals can be cancelled only during its _cool-off period.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can amend it.
        /// * `ProposalIsInmutable`: A Proposal is mutable only during its cool off period.
        #[weight = SimpleDispatchInfo::FixedNormal(1_000_000)]
        pub fn cancel_proposal(origin, index: MipsIndex) -> DispatchResult {
            // 0. Initial info.
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can cancel it.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be cancelled *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsInmutable);

            // 3. Close that proposal.
            Self::close_proposal( index, meta.proposal_hash);
            Ok(())
        }

        /// Id bonds an additional deposit to proposal with index `index`.
        /// That amount is added to the current deposit.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can bond an additional deposit.
        /// * `ProposalIsInmutable`: A Proposal is mutable only during its cool off period.
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn bond_additional_deposit(origin,
            index: MipsIndex,
            additional_deposit: BalanceOf<T>
        ) -> DispatchResult {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // 1. Only owner can cancel it.
            ensure!( meta.proposer == proposer, Error::<T>::BadOrigin);

            // 2. Proposal can be cancelled *ONLY* during its cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsInmutable);

            // 3. Reserve extra deposit & update deposit info for this proposal
            <T as Trait>::Currency::reserve(&proposer, additional_deposit)
                .map_err(|_| Error::<T>::InsufficientDeposit)?;

            <Deposits<T>>::mutate(
                &meta.proposal_hash,
                &proposer,
                |depo_info| depo_info.amount += additional_deposit);
            Ok(())
        }

        /// It unbonds any amount from the deposit of the proposal with index `index`.
        ///
        /// # Errors
        /// * `BadOrigin`: Only the owner of the proposal can release part of the deposit.
        /// * `ProposalIsInmutable`: A Proposal is mutable only during its cool off period.
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
            ensure!( meta.cool_off_until > curr_block_number, Error::<T>::ProposalIsInmutable);

            // 3. Double-check that `amount` is valid.
            let mut depo_info = <Deposits<T>>::get(&meta.proposal_hash, &proposer);
            let new_deposit = depo_info.amount.checked_sub(&amount)
                    .ok_or_else(|| Error::<T>::InsufficientDeposit)?;
            ensure!(
                new_deposit >= Self::min_proposal_deposit(),
                Error::<T>::InsufficientDeposit);
            let diff_amount = depo_info.amount - new_deposit;
            depo_info.amount = new_deposit;

            // 3.1. Unreserve and update deposit info.
            <T as Trait>::Currency::unreserve(&depo_info.owner, diff_amount);
            <Deposits<T>>::insert(&meta.proposal_hash, &proposer, depo_info);
            Ok(())
        }

        /// A network member can vote on any Mesh Improvement Proposal by selecting the hash that
        /// corresponds ot the dispatchable action and vote with some balance.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `index` proposal index
        /// * `aye_or_nay` a bool representing for or against vote
        /// * `deposit` minimum deposit value
        #[weight = SimpleDispatchInfo::FixedNormal(200_000)]
        pub fn vote(origin, proposal_hash: T::Hash, index: MipsIndex, aye_or_nay: bool, deposit: BalanceOf<T>) {
            let proposer = ensure_signed(origin)?;
            let meta = Self::proposal_meta_by_index(index)?;

            // No one should be able to vote during the proposal cool-off period.
            let curr_block_number = <system::Module<T>>::block_number();
            ensure!( meta.cool_off_until <= curr_block_number, Error::<T>::ProposalOnCoolOffPeriod);

            let mut voting = Self::voting(&proposal_hash).ok_or(Error::<T>::NoSuchProposal)?;
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
                <Deposits<T>>::insert(&proposal_hash, &proposer, depo_info);

                <Voting<T>>::remove(&proposal_hash);
                <Voting<T>>::insert(&proposal_hash, voting);
                Self::deposit_event(RawEvent::Voted(proposer, index, proposal_hash, aye_or_nay));
            } else {
                return Err(Error::<T>::DuplicateVote.into())
            }
        }

        /// An emergency stop measure to kill a proposal. Governance committee can kill
        /// a proposal at any time.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn kill_proposal(origin, index: MipsIndex, proposal_hash: T::Hash) {
            T::CommitteeOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

            let mip = Self::proposals(&proposal_hash).ok_or(Error::<T>::NoSuchProposal)?;
            ensure!(mip.index == index, Error::<T>::MismatchedProposalIndex);

            Self::close_proposal(index, proposal_hash);
        }

        /// Any governance committee member can fast track a proposal and turn it into a referendum
        /// that will be voted on by the committee.
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        pub fn fast_track_proposal(origin, index: MipsIndex, proposal_hash: T::Hash) {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            let mip = Self::proposals(&proposal_hash).ok_or("proposal does not exist")?;
            ensure!(mip.index == index, Error::<T>::MismatchedProposalIndex);

            Self::create_referendum(
                index,
                MipsPriority::High,
                proposal_hash,
                mip.proposal,
            );

            Self::deposit_event(RawEvent::ProposalFastTracked(
                index,
                proposal_hash,
            ));

            Self::close_proposal(index, proposal_hash);
        }

        /// Governance committee can make a proposal that automatically becomes a referendum on
        /// which the committee can vote on.
        #[weight = SimpleDispatchInfo::FixedOperational(200_000)]
        pub fn submit_referendum(origin, proposal: Box<T::Proposal>) {
            let sender_key = AccountKey::try_from(ensure_signed(origin)?.encode())?;
            let did = Context::current_identity_or::<Identity<T>>(&sender_key)?;

            ensure!(
                T::GovernanceCommittee::is_member(&did),
                Error::<T>::NotACommitteeMember
            );

            let proposal_hash = T::Hashing::hash_of(&proposal);

            // Proposal must be new
            ensure!(!<Proposals<T>>::contains_key(proposal_hash), Error::<T>::DuplicateProposal);

            let index = Self::proposal_count();
            <ProposalCount>::mutate(|i| *i += 1);

            Self::create_referendum(
                index,
                MipsPriority::High,
                proposal_hash,
                *proposal
            );
        }

        /// Moves a referendum instance into dispatch queue.
        #[weight = SimpleDispatchInfo::FixedOperational(100_000)]
        pub fn enact_referendum(origin, proposal_hash: T::Hash) {
            T::VotingMajorityOrigin::try_origin(origin).map_err(|_| Error::<T>::BadOrigin)?;

            Self::prepare_to_dispatch(proposal_hash);
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
    pub fn proposals_maturing_at(n: T::BlockNumber) -> Vec<(MipsIndex, T::Hash)> {
        Self::proposal_meta()
            .into_iter()
            .filter(|meta| meta.end == n)
            .map(|meta| (meta.index, meta.proposal_hash))
            .collect()
    }

    // Private functions

    /// Runs the following procedure:
    /// 1. Find all proposals that need to end as of this block and close voting
    /// 2. Tally votes
    /// 3. Submit any proposals that meet the quorum threshold, to the governance committee
    pub fn end_block(block_number: T::BlockNumber) -> DispatchResult {
        // Find all matured proposals...
        for (index, hash) in Self::proposals_maturing_at(block_number).into_iter() {
            // Tally votes and create referendums
            Self::tally_votes(index, hash);

            // And close proposals
            Self::close_proposal(index, hash);
        }

        Ok(())
    }

    /// Summarize voting and create referendums if proposals meet or exceed quorum threshold
    fn tally_votes(index: MipsIndex, proposal_hash: T::Hash) {
        if let Some(voting) = <Voting<T>>::get(proposal_hash) {
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
                if let Some(mip) = <Proposals<T>>::get(&proposal_hash) {
                    Self::create_referendum(
                        index,
                        MipsPriority::Normal,
                        proposal_hash,
                        mip.proposal,
                    );
                }
            }
        }
    }

    /// Create a referendum object from a proposal. If governance committee is composed of less
    /// than 2 members, enact it immediately. Otherwise, committee votes on this referendum and
    /// decides whether it should be enacted.
    fn create_referendum(
        index: MipsIndex,
        priority: MipsPriority,
        proposal_hash: T::Hash,
        proposal: T::Proposal,
    ) {
        let ri = PolymeshReferendumInfo {
            index,
            priority,
            proposal_hash,
        };

        <ReferendumMetadata<T>>::mutate(|metadata| metadata.push(ri));
        <Referendums<T>>::insert(proposal_hash, proposal);

        Self::deposit_event(RawEvent::ReferendumCreated(index, priority, proposal_hash));

        // If committee size is too small, enact it.
        if T::GovernanceCommittee::member_count() < 2 {
            Self::prepare_to_dispatch(proposal_hash);
        }
    }

    /// Close a proposal. Voting ceases and proposal is removed from storage.
    /// All deposits are unlocked and returned to respective stakers.
    fn close_proposal(index: MipsIndex, proposal_hash: T::Hash) {
        if <Voting<T>>::get(proposal_hash).is_some() {
            Self::unreserve_deposits(&proposal_hash);
        }

        if <Proposals<T>>::take(&proposal_hash).is_some() {
            <Voting<T>>::remove(&proposal_hash);
            let hash = proposal_hash;
            <ProposalMetadata<T>>::mutate(|metadata| metadata.retain(|m| m.proposal_hash != hash));
            <ProposalByIndex<T>>::remove(index);

            Self::deposit_event(RawEvent::ProposalClosed(index, hash));
        }
    }

    /// It returns back each deposit to their owners for an specific `proposal_hash`.
    fn unreserve_deposits(proposal_hash: &T::Hash) {
        <Deposits<T>>::iter_prefix(proposal_hash).for_each(|depo_info| {
            let _ = <T as Trait>::Currency::unreserve(&depo_info.owner, depo_info.amount);
        });
        <Deposits<T>>::remove_prefix(proposal_hash);
    }

    fn prepare_to_dispatch(hash: T::Hash) {
        if let Some(referendum) = <Referendums<T>>::get(&hash) {
            let result = match referendum.dispatch(system::RawOrigin::Root.into()) {
                Ok(_) => true,
                Err(e) => {
                    let e: DispatchError = e;
                    sp_runtime::print(e);
                    false
                }
            };
            Self::deposit_event(RawEvent::ReferendumEnacted(hash, result));
        }
    }

    /// It returns the proposal metadata of proposal with index `index` or
    /// a `MismatchedProposalIndex` error.
    fn proposal_meta_by_index(
        index: MipsIndex,
    ) -> Result<MipsMetadata<T::AccountId, T::BlockNumber, T::Hash>, DispatchError> {
        Self::proposal_meta()
            .into_iter()
            .find(|meta| meta.index == index)
            .ok_or(Error::<T>::MismatchedProposalIndex.into())
    }
}

impl<T: Trait> Module<T> {
    /// Retrieve votes for a proposal represented by MipsIndex `index`.
    pub fn get_votes(index: MipsIndex) -> VoteCount<BalanceOf<T>>
    where
        T: Send + Sync,
        BalanceOf<T>: Send + Sync,
    {
        let proposal_hash: T::Hash = <ProposalByIndex<T>>::get(index);
        if let Some(voting) = <Voting<T>>::get(&proposal_hash) {
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
            if let Some(votes) = Self::voting(&meta.proposal_hash) {
                if votes.ayes.iter().any(|(a, _)| a == &address)
                    || votes.nays.iter().any(|(a, _)| a == &address)
                {
                    indices.push(votes.index);
                }
            }
        }
        indices
    }
}
