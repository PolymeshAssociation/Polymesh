//! # MIPS Module
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
//! The Asset module provides functions for:
//!
//! - Creating Mesh Improvement Proposals
//! - Voting on Mesh Improvement Proposals
//! - Governance committee to ratify or reject proposals
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `propose` - Token holders can propose a new ballot.
//! - `vote` - Token holders can vote on a ballot.
//!
//! ### Public Functions
//!
//! - `token_details` - Returns details of the token

use rstd::prelude::*;
use sr_primitives::{
    traits::{Dispatchable, EnsureOrigin, Hash},
    weights::SimpleDispatchInfo,
};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{
        Currency, Get, LockIdentifier, LockableCurrency, OnFreeBalanceZero, ReservableCurrency,
        WithdrawReason,
    },
    Parameter,
};
use system::ensure_signed;

/// Mesh Improvement Proposal index.
pub type ProposalIndex = u32;

/// Balance
type BalanceOf<T> = <<T as Trait>::Currency as Currency<<T as system::Trait>::AccountId>>::Balance;

/// A voting decision
#[derive(codec::Encode, codec::Decode, Clone, Copy, PartialEq, Eq, Debug, PartialOrd, Ord)]
pub enum Vote {
    Abstain,
    Aye,
    Nay,
}

impl Default for Vote {
    fn default() -> Self {
        Vote::Abstain
    }
}

/// The module's configuration trait.
pub trait Trait: system::Trait {
    /// Currency type for this module.
    type Currency: ReservableCurrency<Self::AccountId>
        + LockableCurrency<Self::AccountId, Moment = Self::BlockNumber>;

    /// A proposal is a dispatchable call
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin>;

    /// The minimum amount to be used as a deposit for a proposal.
    type MinimumProposalDeposit: Get<BalanceOf<Self>>;

    /// How long (in blocks) a ballot runs
    type VotingPeriod: Get<Self::BlockNumber>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MIPS {
        /// Proposals so far. Index can be used to keep track of MIPs off-chain.
        pub ProposalCount get(proposal_count): u32;

        /// The hashes of the active proposals.
        pub Proposals get(proposals): Vec<T::Hash>;

        /// Those who have locked a deposit.
        /// proposal index -> (deposit, proposer)
        pub DepositOf get(deposit_of): map T::Hash => Option<(T::AccountId, BalanceOf<T>)>;

        /// Actual proposal for a given hash, if it's current.
        /// proposal hash -> proposal
        pub ProposalOf get(proposal_of): map T::Hash => Option<T::Proposal>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = BalanceOf<T>,
        AccountId = <T as system::Trait>::AccountId,
    {
        Proposed(AccountId, Balance),
        Voted(AccountId),
    }
);

// The module's dispatchable functions.
decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        /// The minimum amount to be used as a deposit for a public referendum proposal.
        const MinimumProposalDeposit: BalanceOf<T> = T::MinimumProposalDeposit::get();

        /// How long (in blocks) a ballot runs
        const VotingPeriod: T::BlockNumber = T::VotingPeriod::get();

        fn deposit_event() = default;

        /// A network member creates a Mesh Improvement Proposal by submitting a dispatchable which
        /// changes the network in someway. A minimum deposit is required to open a new proposal.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `deposit` minimum deposit value
        #[weight = SimpleDispatchInfo::FixedNormal(5_000_000)]
        pub fn propose(origin, proposal: Box<T::Proposal>, deposit: BalanceOf<T>) -> Result {
            let proposer = ensure_signed(origin)?;
            let proposal_hash = T::Hashing::hash_of(&proposal);

            // Pre conditions: caller must have min balance
            ensure!(deposit >= T::MinimumProposalDeposit::get(), "minimum deposit required to start a proposal");
            // Proposal must be new
            ensure!(!<ProposalOf<T>>::exists(proposal_hash), "duplicate proposals are not allowed");

            // Reserve the minimum deposit
            T::Currency::reserve(&proposer, deposit).map_err(|_| "proposer can't afford to lock minimum deposit")?;

            let index = Self::proposal_count();
            <ProposalCount>::mutate(|i| *i += 1);
            <Proposals<T>>::mutate(|proposals| proposals.push(proposal_hash));

            <DepositOf<T>>::insert(proposal_hash, (proposer.clone(), deposit));

            Self::deposit_event(RawEvent::Proposed(proposer, deposit));
            Ok(())
        }

        /// A network member can vote on any Mesh Improvement Proposal by selecting the hash that
        /// corresponds ot the dispatchable action and vote with some balance.
        ///
        /// # Arguments
        /// * `proposal` a dispatchable call
        /// * `deposit` minimum deposit value
        pub fn vote(origin, proposal_hash: T::Hash, #[compact] index: ProposalIndex) -> Result {
            let proposer = ensure_signed(origin)?;

            let mut proposal = Self::proposal_of(&proposal_hash).ok_or("proposal must exist")?;

            Self::deposit_event(RawEvent::Voted(proposer));
            Ok(())
        }

        /// At the end of each block check if it's time for a ballot to end. If ballot ends,
        /// proceed to ratification process.
        fn on_initialize(n: T::BlockNumber) {
            if let Err(e) = Self::end_block(n) {
            }
        }

    }
}

impl<T: Trait> Module<T> {
    /// Runs ratification process
    fn end_block(block_number: T::BlockNumber) -> Result {
        Ok(())
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;

    use primitives::{Blake2Hasher, H256};
    use runtime_io::with_externalities;
    use sr_primitives::weights::Weight;
    use sr_primitives::Perbill;
    use sr_primitives::{
        testing::Header,
        traits::{BlakeTwo256, IdentityLookup},
    };
    use support::{assert_ok, impl_outer_origin, parameter_types};

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;
    parameter_types! {
        pub const BlockHashCount: u64 = 250;
        pub const MaximumBlockWeight: Weight = 1024;
        pub const MaximumBlockLength: u32 = 2 * 1024;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }
    impl system::Trait for Test {
        type Origin = Origin;
        type Call = ();
        type Index = u64;
        type BlockNumber = u64;
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
    impl Trait for Test {
        type Event = ();
    }
    type MIPS = Module<Test>;

    // This function basically just builds a genesis storage key/value store according to
    // our desired mockup.
    fn new_test_ext() -> runtime_io::TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    #[test]
    fn it_works_for_default_value() {
        with_externalities(&mut new_test_ext(), || {
            // Just a dummy test for the dummy funtion `do_something`
            // calling the `do_something` function with a value 42
            assert_ok!(MIPS::do_something(Origin::signed(1), 42));
            // asserting that the stored value is equal to what we stored
            assert_eq!(MIPS::something(), Some(42));
        });
    }
}
