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

use crate::balances;
use rstd::prelude::*;
use sr_primitives::{traits::Dispatchable, weights::SimpleDispatchInfo};
use srml_support::{
    decl_event, decl_module, decl_storage,
    dispatch::Result,
    ensure,
    traits::{Currency, Get, ReservableCurrency},
    Parameter,
};
use system::ensure_signed;

/// Mesh Improvement Proposal index.
pub type ProposalIndex = u32;

/// The module's configuration trait.
pub trait Trait: system::Trait + balances::Trait {
    /// A proposal is a dispatchable call
    type Proposal: Parameter + Dispatchable<Origin = Self::Origin>;

    /// The minimum amount to be used as a deposit for a proposal.
    type MinimumProposalDeposit: Get<Self::Balance>;

    /// How long (in blocks) a ballot runs
    type VotingPeriod: Get<Self::BlockNumber>;

    /// The overarching event type.
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

// This module's storage items.
decl_storage! {
    trait Store for Module<T: Trait> as MIPS {
        /// Proposal index used to keep track of MIPs off-chain.
//        pub ProposalCount get(fn proposal_count) build(|_| 0 as ProposalIndex) : ProposalIndex;

        /// Unsorted MIPs.
        /// proposal index -> (dispatchable proposal, proposer)
//        pub Proposals get(proposals): map ProposalIndex => (T::Proposal, T::AccountId);

        /// Those who have locked a deposit.
        /// proposal index -> (deposit, proposer)
        pub Deposits get(deposits): map ProposalIndex => Option<(T::Balance, T::AccountId)>;
    }
}

decl_event!(
    pub enum Event<T>
    where
        AccountId = <T as system::Trait>::AccountId,
        Balance = <T as balances::Trait>::Balance,
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
        const MinimumProposalDeposit: T::Balance = T::MinimumProposalDeposit::get();

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
        pub fn propose(origin, proposal: Box<T::Proposal>, deposit: T::Balance) -> Result {
            let proposer = ensure_signed(origin)?;

            // Pre conditions
            ensure!(deposit >= T::MinimumProposalDeposit::get(), "minimum deposit required to start a proposal");

//            // Reserve the minimum deposit
//            T::Balance::reserve(&proposer, deposit).map_err(|_| "proposer can't afford to lock minimum deposit")?;

            Self::deposit_event(RawEvent::Proposed(proposer, deposit));
            Ok(())
        }

        pub fn vote(origin) -> Result {
            let who = ensure_signed(origin)?;

            Self::deposit_event(RawEvent::Voted(who));
            Ok(())
        }
    }
}

impl<T: Trait> Module<T> {}

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
