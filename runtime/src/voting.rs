//! # Voting Module
//!
//! The Voting module provides functionality for corporate voting.
//!
//! ## Overview
//!
//! The Voting module provides functions for:
//!
//! - Creating ballots that can include multiple proposals
//! - Voting on proposals
//! - Cancelling ballots
//!
//! ### Terminology
//!
//! - **Ballot:** It is a collection of proposals on which a tokenholder can vote.
//!     Additional parameters include voting start date, voting end date and checkpoint id.
//!     Checkpoint id is used to prevent double voting with same coins. When voting on a ballot,
//!     the total number of votes that a tokenholder can cast is equal to their balance at the checkpoint.
//!     Voters can distribute their votes accross all the proposals in the ballot.
//! - **Proposal:** It is a suggestion or a question that can have an infinite number of choices that can be voted on.
//!     Additional parameters include title of the proposal and a link from where more info can be fetched.
//!     The most common proposal is of accept/reject type where the proposal has two choices, yes/no.
//!     Any voting power that is not used is considered as abstain.
//!
//! ## Interface
//!
//! ### Dispatchable Functions
//!
//! - `add_ballot` - Creates a ballot.
//! - `vote` - Casts a vote.
//! - `cancel_ballot` - Cancels an existing ballot.

use crate::{
    asset::{self, AssetTrait},
    balances, identity, utils,
};
use codec::Encode;
use primitives::{IdentityId, Key};
use rstd::{convert::TryFrom, prelude::*};
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait + identity::Trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
    type Asset: asset::AssetTrait<Self::Balance>;
}

/// Details about ballots
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Ballot<V> {
    /// The user's historic balance at this checkpoint is used as maximum vote weight
    checkpoint_id: u64,

    /// Timestamp at which voting should start
    voting_start: V,

    /// Timestamp at which voting should end
    voting_end: V,

    /// Array of proposals that can be voted on
    proposals: Vec<Proposal>,
}

/// Details about proposals
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Proposal {
    /// Title of the proposal
    title: Vec<u8>,

    /// Link from where more information about the proposal can be fetched
    info_link: Vec<u8>,

    /// Choices for the proposal excluding abstain
    /// Voting power not used is considered abstained
    choices: Vec<Vec<u8>>,
}

decl_storage! {
    trait Store for Module<T: Trait> as Voting {
        /// Mapping of ticker and ballot name -> ballot details
        pub Ballots get(ballots): linked_map(Vec<u8>, Vec<u8>) => Ballot<T::Moment>;

        /// Helper data to make voting cheaper.
        /// (ticker, BallotName) -> NoOfChoices
        pub TotalChoices get(total_choices): map (Vec<u8>, Vec<u8>) => u64;

        /// (Ticker, BallotName, DID) -> Vector of vote weights.
        /// weight at 0 index means weight for choice 1 of proposal 1.
        /// weight at 1 index means weight for choice 2 of proposal 1.
        /// User must enter 0 vote weight if they don't want to vote for a choice.
        pub Votes get(votes): map (Vec<u8>, Vec<u8>, IdentityId) => Vec<T::Balance>;

        /// (Ticker, BallotName) -> Vector of current vote weights.
        /// weight at 0 index means weight for choice 1 of proposal 1.
        /// weight at 1 index means weight for choice 2 of proposal 1.
        pub Results get(results): map (Vec<u8>, Vec<u8>) => Vec<T::Balance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {

        // Initializing events
        fn deposit_event() = default;

        /// Adds a ballot
        ///
        /// # Arguments
        /// * `did` - DID of the token owner. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which ballot is to be created
        /// * `ballot_name` - Name of the ballot
        /// * `ballot_details` - Other details of the ballot
        pub fn add_ballot(origin, did: IdentityId, ticker: Vec<u8>, ballot_name: Vec<u8>, ballot_details: Ballot<T::Moment>) -> Result {
            let sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&upper_ticker, did),"Sender must be the token owner");

            // This avoids cloning the variables to make the same tupple again and again.
            let upper_ticker_ballot_name = (upper_ticker.clone(), ballot_name.clone());

            // Ensure the uniqueness of the ballot
            ensure!(!<Ballots<T>>::exists(&upper_ticker_ballot_name), "A ballot with same name already exisits");

            let now = <timestamp::Module<T>>::get();

            ensure!(now < ballot_details.voting_end, "Voting end date in past");
            ensure!(ballot_details.voting_end > ballot_details.voting_start, "Voting end date before voting start date");
            ensure!(ballot_details.proposals.len() > 0, "No proposal submitted");

            // NB: Checkpoint ID is not verified here to allow creating ballots that will become active in future.
            // Voting will only be allowed on checkpoints that exist.

            let mut total_choices:usize = 0usize;

            for proposal in &ballot_details.proposals {
                ensure!(proposal.choices.len() > 0, "No choice submitted");
                total_choices += proposal.choices.len();
            }

            if let Ok(total_choices_u64) = u64::try_from(total_choices) {
                <TotalChoices>::insert(&upper_ticker_ballot_name, total_choices_u64);
            } else {
                return Err("Could not decode choices")
            }

            <Ballots<T>>::insert(&upper_ticker_ballot_name, ballot_details.clone());

            let initial_results = vec![0.into(); total_choices];
            <Results<T>>::insert(&upper_ticker_ballot_name, initial_results);

            Self::deposit_event(RawEvent::BallotCreated(upper_ticker, ballot_name, ballot_details));

            Ok(())
        }

        /// Casts a vote
        ///
        /// # Arguments
        /// * `did` - DID of the voter. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which vote is to be cast
        /// * `ballot_name` - Name of the ballot
        /// * `votes` - The actual vote to be cast
        pub fn vote(origin, did: IdentityId, ticker: Vec<u8>, ballot_name: Vec<u8>, votes: Vec<T::Balance>) -> Result {
            let sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // This avoids cloning the variables to make the same tupple again and again
            let upper_ticker_ballot_name = (upper_ticker.clone(), ballot_name.clone());

            // Ensure validity the ballot
            ensure!(<Ballots<T>>::exists(&upper_ticker_ballot_name), "Ballot does not exist");
            let ballot = <Ballots<T>>::get(&upper_ticker_ballot_name);
            let now = <timestamp::Module<T>>::get();
            ensure!(ballot.voting_start <= now, "Voting hasn't started yet");
            ensure!(ballot.voting_end > now, "Voting ended already");

            // Ensure validity of checkpoint
            ensure!(<asset::TotalCheckpoints>::exists(&upper_ticker), "No checkpoints created");
            let count = <asset::TotalCheckpoints>::get(&upper_ticker);
            ensure!(ballot.checkpoint_id <= count, "Checkpoint has not been created yet");

            // Ensure vote is valid
            if let Ok(votes_len) = u64::try_from(votes.len()) {
                ensure!(votes_len == <TotalChoices>::get(&upper_ticker_ballot_name), "Invalid vote");
            } else {
                return Err("Invalid vote")
            }

            let mut total_votes: T::Balance = 0.into();
            for vote in &votes {
                total_votes += *vote;
            }
            ensure!(total_votes <= T::Asset::get_balance_at(&upper_ticker, did, ballot.checkpoint_id), "Not enough balance");

            // This avoids cloning the variables to make the same tupple again and again
            let upper_ticker_ballot_name_did = (upper_ticker.clone(), ballot_name.clone(), did);

            // Check if user has already voted for this ballot or if they are voting for the first time
            if <Votes<T>>::exists(&upper_ticker_ballot_name_did) {
                //User wants to change their vote. We first need to subtract their existing vote
                let previous_votes = <Votes<T>>::get(&upper_ticker_ballot_name_did);
                <Results<T>>::mutate(&upper_ticker_ballot_name, |results| {
                    for i in 0..results.len() {
                        results[i] -= previous_votes[i];
                    }
                });
            }

            // Adding users' vote to the result
            <Results<T>>::mutate(&upper_ticker_ballot_name, |results| {
                for i in 0..results.len() {
                    results[i] += votes[i];
                }
            });

            // Storing users' vote onchain. This is needed when user wants to their change vote
            <Votes<T>>::insert(&upper_ticker_ballot_name_did, votes.clone());

            Self::deposit_event(RawEvent::VoteCast(upper_ticker, ballot_name, votes));

            Ok(())
        }

        /// Cancels a vote by setting it as expired
        ///
        /// # Arguments
        /// * `did` - DID of the token owner. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which ballot is to be cancelled
        /// * `ballot_name` - Name of the ballot
        pub fn cancel_ballot(origin, did: IdentityId, ticker: Vec<u8>, ballot_name: Vec<u8>) -> Result {
            let sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_authorized_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&upper_ticker, did),"Sender must be the token owner");

            // This avoids cloning the variables to make the same tupple again and again
            let upper_ticker_ballot_name = (upper_ticker.clone(), ballot_name.clone());

            // Ensure the existance of valid ballot
            ensure!(<Ballots<T>>::exists(&upper_ticker_ballot_name), "Ballot does not exisit");
            let ballot = <Ballots<T>>::get(&upper_ticker_ballot_name);
            let now = <timestamp::Module<T>>::get();
            ensure!(now < ballot.voting_end, "Voting already ended");

            // Clearing results
            <Results<T>>::mutate(&upper_ticker_ballot_name, |results| {
                for i in 0..results.len() {
                    results[i] = 0.into();
                }
            });

            // NB Not deleting the ballot to prevent someone from
            // deleting a ballot mid vote and creating a new one with same name to confuse voters

            // This will prevent further voting. Essentially, canceling the ballot
            <Ballots<T>>::mutate(&upper_ticker_ballot_name, |ballot_details| {
                ballot_details.voting_end = now;
            });

            Self::deposit_event(RawEvent::BallotCancelled(upper_ticker, ballot_name));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as balances::Trait>::Balance,
        Moment = <T as timestamp::Trait>::Moment,
    {
        /// A new ballot is created (Ticker, BallotName, BallotDetails)
        BallotCreated(Vec<u8>, Vec<u8>, Ballot<Moment>),

        /// A vote is cast (Ticker, BallotName, Vote)
        VoteCast(Vec<u8>, Vec<u8>, Vec<Balance>),

        /// An existing ballot is cancelled (Ticker, BallotName)
        BallotCancelled(Vec<u8>, Vec<u8>),
    }
);

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Vec<u8>, did: IdentityId) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker.as_slice());
        T::Asset::is_owner(&upper_ticker, did)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use sr_io::{with_externalities, TestExternalities};
    use sr_primitives::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        AnySignature, Perbill,
    };
    use srml_support::traits::Currency;
    use srml_support::{
        assert_err, assert_ok,
        dispatch::{DispatchError, DispatchResult},
        impl_outer_origin, parameter_types,
    };
    use std::result::Result;
    use substrate_primitives::{Blake2Hasher, H256};
    use test_client::{self, AccountKeyring};

    use crate::{
        asset::SecurityToken, balances, exemption, general_tm, identity, percentage_tm, registry,
    };

    impl_outer_origin! {
        pub enum Origin for Test {}
    }

    // For testing the module, we construct most of a mock runtime. This means
    // first constructing a configuration type (`Test`) which `impl`s each of the
    // configuration traits of modules we want to use.
    #[derive(Clone, Eq, PartialEq)]
    pub struct Test;

    parameter_types! {
        pub const BlockHashCount: u32 = 250;
        pub const MaximumBlockWeight: u32 = 4096;
        pub const MaximumBlockLength: u32 = 4096;
        pub const AvailableBlockRatio: Perbill = Perbill::from_percent(75);
    }

    type SessionIndex = u32;
    type AuthorityId = <AnySignature as Verify>::Signer;
    type BlockNumber = u64;
    type AccountId = <AnySignature as Verify>::Signer;
    type OffChainSignature = AnySignature;

    impl system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = u64;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<AccountId>;
        type Header = Header;
        type Event = ();
        type Call = ();
        type WeightMultiplierUpdate = ();
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

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type OffChainSignature = OffChainSignature;
        fn validator_id_to_account_id(v: <Self as session::Trait>::ValidatorId) -> Self::AccountId {
            v
        }
    }

    pub struct TestOnSessionEnding;
    impl session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl session::SessionHandler<AuthorityId> for TestSessionHandler {
        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
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

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

    impl registry::Trait for Test {}

    impl exemption::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl general_tm::Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    impl Trait for Test {
        type Event = ();
        type Asset = asset::Module<Test>;
    }

    type Identity = identity::Module<Test>;
    type GeneralTM = general_tm::Module<Test>;
    type Voting = Module<Test>;
    type Balances = balances::Module<Test>;
    type Asset = asset::Module<Test>;

    /// Create externalities
    fn build_ext() -> TestExternalities<Blake2Hasher> {
        system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap()
            .into()
    }

    fn make_account(
        account_id: &AccountId,
    ) -> Result<(<Test as system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        Identity::register_did(signed_id.clone(), vec![])?;
        let did = Identity::get_identity(&Key::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn add_ballot() {
        with_externalities(&mut build_ext(), || {
            let _token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_acc, token_owner_did) = make_account(&_token_owner_acc).unwrap();
            let _tokenholder_acc = AccountId::from(AccountKeyring::Bob);
            let (tokenholder_acc, tokenholder_did) = make_account(&_tokenholder_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
            };

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
            ));

            let now = Utc::now().timestamp() as u64;
            <timestamp::Module<Test>>::set_timestamp(now);

            let proposal1 = Proposal {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let proposal2 = Proposal {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                proposals: vec![proposal1.clone(), proposal2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                "sender must be a signing key for DID"
            );

            assert_err!(
                Voting::add_ballot(
                    tokenholder_acc.clone(),
                    tokenholder_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                "Sender must be the token owner"
            );

            let expired_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: 0,
                proposals: vec![proposal1.clone(), proposal2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    expired_ballot_details.clone()
                ),
                "Voting end date in past"
            );

            let invalid_date_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now + now + now,
                voting_end: now + now,
                proposals: vec![proposal1.clone(), proposal2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    invalid_date_ballot_details.clone()
                ),
                "Voting end date before voting start date"
            );

            let empty_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                proposals: vec![],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    empty_ballot_details.clone()
                ),
                "No proposal submitted"
            );

            let empty_proposal = Proposal {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![],
            };

            let no_choice_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                proposals: vec![proposal1.clone(), proposal2.clone(), empty_proposal],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    no_choice_ballot_details.clone()
                ),
                "No choice submitted"
            );

            // Adding ballot
            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone(),
                ballot_details.clone()
            ));

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                "A ballot with same name already exisits"
            );
        });
    }

    #[test]
    fn cancel_ballot() {
        with_externalities(&mut build_ext(), || {
            let _token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_acc, token_owner_did) = make_account(&_token_owner_acc).unwrap();
            let _tokenholder_acc = AccountId::from(AccountKeyring::Bob);
            let (tokenholder_acc, tokenholder_did) = make_account(&_tokenholder_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did,
                total_supply: 1_000_000,
                divisible: true,
            };

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
            ));

            let now = Utc::now().timestamp() as u64;
            <timestamp::Module<Test>>::set_timestamp(now);

            let proposal1 = Proposal {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let proposal2 = Proposal {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                proposals: vec![proposal1.clone(), proposal2.clone()],
            };

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone()
                ),
                "Ballot does not exisit"
            );

            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone(),
                ballot_details.clone()
            ));

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    token.name.clone(),
                    ballot_name.clone()
                ),
                "sender must be a signing key for DID"
            );

            assert_err!(
                Voting::cancel_ballot(
                    tokenholder_acc.clone(),
                    tokenholder_did,
                    token.name.clone(),
                    ballot_name.clone()
                ),
                "Sender must be the token owner"
            );

            <timestamp::Module<Test>>::set_timestamp(now + now + now);

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone()
                ),
                "Voting already ended"
            );

            <timestamp::Module<Test>>::set_timestamp(now);

            // Cancelling ballot
            assert_ok!(Voting::cancel_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone()
            ));
        });
    }

    #[test]
    fn vote() {
        with_externalities(&mut build_ext(), || {
            let _token_owner_acc = AccountId::from(AccountKeyring::Alice);
            let (token_owner_acc, token_owner_did) = make_account(&_token_owner_acc).unwrap();
            let _tokenholder_acc = AccountId::from(AccountKeyring::Bob);
            let (tokenholder_acc, tokenholder_did) = make_account(&_tokenholder_acc).unwrap();

            // A token representing 1M shares
            let token = SecurityToken {
                name: vec![0x01],
                owner_did: token_owner_did,
                total_supply: 1000,
                divisible: true,
            };

            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                token.name.clone(),
                token.total_supply,
                true
            ));

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                asset_rule
            ));

            assert_ok!(Asset::transfer(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                tokenholder_did,
                500
            ));

            let now = Utc::now().timestamp() as u64;
            <timestamp::Module<Test>>::set_timestamp(now);

            let proposal1 = Proposal {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let proposal2 = Proposal {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 2,
                voting_start: now,
                voting_end: now + now,
                proposals: vec![proposal1.clone(), proposal2.clone()],
            };

            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone(),
                ballot_details.clone()
            ));

            let votes = vec![100, 100, 100, 100, 100];

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    votes.clone()
                ),
                "sender must be a signing key for DID"
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    vec![0x02],
                    votes.clone()
                ),
                "Ballot does not exist"
            );

            <timestamp::Module<Test>>::set_timestamp(now - 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    votes.clone()
                ),
                "Voting hasn't started yet"
            );

            <timestamp::Module<Test>>::set_timestamp(now + now + 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    votes.clone()
                ),
                "Voting ended already"
            );

            <timestamp::Module<Test>>::set_timestamp(now + 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    votes.clone()
                ),
                "No checkpoints created"
            );

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
            ));

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    votes.clone()
                ),
                "Checkpoint has not been created yet"
            );

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
            ));

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    vec![100, 100, 100, 100]
                ),
                "Invalid vote"
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    vec![100, 100, 100, 100, 100, 100]
                ),
                "Invalid vote"
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    token.name.clone(),
                    ballot_name.clone(),
                    vec![100, 100, 100, 100, 200]
                ),
                "Not enough balance"
            );

            // Initial vote
            assert_ok!(Voting::vote(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone(),
                votes.clone()
            ));

            let mut result = Voting::results((token.name.clone(), ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [100, 100, 100, 100, 100], "Invalid result");

            // Changed vote
            assert_ok!(Voting::vote(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ballot_name.clone(),
                vec![500, 0, 0, 0, 0]
            ));

            result = Voting::results((token.name.clone(), ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [500, 0, 0, 0, 0], "Invalid result");

            // Second vote
            assert_ok!(Voting::vote(
                tokenholder_acc.clone(),
                tokenholder_did,
                token.name.clone(),
                ballot_name.clone(),
                vec![0, 500, 0, 0, 0]
            ));

            result = Voting::results((token.name.clone(), ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [500, 500, 0, 0, 0], "Invalid result");
        })
    }
}
