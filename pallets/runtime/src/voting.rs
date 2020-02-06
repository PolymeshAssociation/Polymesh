//! # Voting Module
//!
//! The Voting module provides functionality for corporate voting.
//!
//! ## Overview
//!
//! The Voting module provides functions for:
//!
//! - Creating ballots that can include multiple motions
//! - Voting on motions
//! - Cancelling ballots
//!
//! ### Terminology
//!
//! - **Ballot:** It is a collection of motions on which a tokenholder can vote.
//!     Additional parameters include voting start date, voting end date and checkpoint id.
//!     Checkpoint id is used to prevent double voting with same coins. When voting on a ballot,
//!     the total number of votes that a tokenholder can cast is equal to their balance at the checkpoint.
//!     Voters can distribute their votes accross all the motions in the ballot.
//! - **motion:** It is a suggestion or a question that can have an infinite number of choices that can be voted on.
//!     Additional parameters include title of the motion and a link from where more info can be fetched.
//!     The most common motion is of accept/reject type where the motion has two choices, yes/no.
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
    utils,
};

use polymesh_primitives::{AccountKey, IdentityId, Signatory, Ticker};
use polymesh_runtime_common::{identity::Trait as IdentityTrait, CommonTrait};
use polymesh_runtime_identity as identity;

use codec::Encode;
use frame_support::{
    decl_error, decl_event, decl_module, decl_storage, dispatch::DispatchResult, ensure,
};
use frame_system::{self as system, ensure_signed};
use sp_std::{convert::TryFrom, prelude::*, vec};

/// The module's configuration trait.
pub trait Trait:
    pallet_timestamp::Trait + frame_system::Trait + utils::Trait + IdentityTrait
{
    type Event: From<Event<Self>> + Into<<Self as frame_system::Trait>::Event>;
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

    /// Array of motions that can be voted on
    motions: Vec<Motion>,
}

/// Details about motions
#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Motion {
    /// Title of the motion
    title: Vec<u8>,

    /// Link from where more information about the motion can be fetched
    info_link: Vec<u8>,

    /// Choices for the motion excluding abstain
    /// Voting power not used is considered abstained
    choices: Vec<Vec<u8>>,
}

decl_storage! {
    trait Store for Module<T: Trait> as Voting {
        /// Mapping of ticker and ballot name -> ballot details
        pub Ballots get(fn ballots): linked_map(Ticker, Vec<u8>) => Ballot<T::Moment>;

        /// Helper data to make voting cheaper.
        /// (ticker, BallotName) -> NoOfChoices
        pub TotalChoices get(fn total_choices): map (Ticker, Vec<u8>) => u64;

        /// (Ticker, BallotName, DID) -> Vector of vote weights.
        /// weight at 0 index means weight for choice 1 of motion 1.
        /// weight at 1 index means weight for choice 2 of motion 1.
        /// User must enter 0 vote weight if they don't want to vote for a choice.
        pub Votes get(fn votes): map (Ticker, Vec<u8>, IdentityId) => Vec<T::Balance>;

        /// (Ticker, BallotName) -> Vector of current vote weights.
        /// weight at 0 index means weight for choice 1 of motion 1.
        /// weight at 1 index means weight for choice 2 of motion 1.
        pub Results get(fn results): map (Ticker, Vec<u8>) => Vec<T::Balance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Define Error type
        type Error = Error<T>;

        // Initializing events
        fn deposit_event() = default;

        /// Adds a ballot
        ///
        /// # Arguments
        /// * `did` - DID of the token owner. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which ballot is to be created
        /// * `ballot_name` - Name of the ballot
        /// * `ballot_details` - Other details of the ballot
        pub fn add_ballot(origin, did: IdentityId, ticker: Ticker, ballot_name: Vec<u8>, ballot_details: Ballot<T::Moment>) -> DispatchResult {
            let sender = Signatory::AccountKey(AccountKey::try_from(ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), Error::<T>::InvalidSigner);
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), Error::<T>::InvalidOwner);

            // This avoids cloning the variables to make the same tupple again and again.
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure the uniqueness of the ballot
            ensure!(!<Ballots<T>>::exists(&ticker_ballot_name), Error::<T>::AlreadyExists);

            let now = <pallet_timestamp::Module<T>>::get();

            ensure!(now < ballot_details.voting_end, Error::<T>::InvalidDate);
            ensure!(ballot_details.voting_end > ballot_details.voting_start, Error::<T>::InvalidDate);
            ensure!(ballot_details.motions.len() > 0, Error::<T>::NoMotions);

            // NB: Checkpoint ID is not verified here to allow creating ballots that will become active in future.
            // Voting will only be allowed on checkpoints that exist.

            let mut total_choices:usize = 0usize;

            for motion in &ballot_details.motions {
                ensure!(motion.choices.len() > 0, Error::<T>::NoChoicesInMotions);
                total_choices += motion.choices.len();
            }

            if let Ok(total_choices_u64) = u64::try_from(total_choices) {
                <TotalChoices>::insert(&ticker_ballot_name, total_choices_u64);
            } else {
                return Err(Error::<T>::InvalidChoicesType.into());
            }

            <Ballots<T>>::insert(&ticker_ballot_name, ballot_details.clone());

            let initial_results = vec![T::Balance::from(0); total_choices];
            <Results<T>>::insert(&ticker_ballot_name, initial_results);

            Self::deposit_event(RawEvent::BallotCreated(ticker, ballot_name, ballot_details));

            Ok(())
        }

        /// Casts a vote
        ///
        /// # Arguments
        /// * `did` - DID of the voter. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which vote is to be cast
        /// * `ballot_name` - Name of the ballot
        /// * `votes` - The actual vote to be cast
        pub fn vote(origin, did: IdentityId, ticker: Ticker, ballot_name: Vec<u8>, votes: Vec<T::Balance>) -> DispatchResult {
            let sender = Signatory::AccountKey( AccountKey::try_from( ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), Error::<T>::InvalidSigner);
            ticker.canonize();

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure validity the ballot
            ensure!(<Ballots<T>>::exists(&ticker_ballot_name), Error::<T>::NotExists);
            let ballot = <Ballots<T>>::get(&ticker_ballot_name);
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(ballot.voting_start <= now, Error::<T>::NotStarted);
            ensure!(ballot.voting_end > now, Error::<T>::AlreadyEnded);

            // Ensure validity of checkpoint
            ensure!(<asset::TotalCheckpoints>::exists(&ticker), Error::<T>::NoCheckpoints);
            let count = <asset::TotalCheckpoints>::get(&ticker);
            ensure!(ballot.checkpoint_id <= count, Error::<T>::NoCheckpoints);

            // Ensure vote is valid
            if let Ok(votes_len) = u64::try_from(votes.len()) {
                ensure!(votes_len == <TotalChoices>::get(&ticker_ballot_name), Error::<T>::InvalidVote);
            } else {
                return Err(Error::<T>::InvalidVote.into())
            }

            let mut total_votes: T::Balance = 0.into();
            for vote in &votes {
                total_votes += *vote;
            }
            ensure!(total_votes <= T::Asset::get_balance_at(&ticker, did, ballot.checkpoint_id), Error::<T>::InsufficientBalance);

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name_did = (ticker, ballot_name.clone(), did);

            // Check if user has already voted for this ballot or if they are voting for the first time
            if <Votes<T>>::exists(&ticker_ballot_name_did) {
                //User wants to change their vote. We first need to subtract their existing vote
                let previous_votes = <Votes<T>>::get(&ticker_ballot_name_did);
                <Results<T>>::mutate(&ticker_ballot_name, |results| {
                    for i in 0..results.len() {
                        results[i] -= previous_votes[i];
                    }
                });
            }

            // Adding users' vote to the result
            <Results<T>>::mutate(&ticker_ballot_name, |results| {
                for i in 0..results.len() {
                    results[i] += votes[i];
                }
            });

            // Storing users' vote onchain. This is needed when user wants to their change vote
            <Votes<T>>::insert(&ticker_ballot_name_did, votes.clone());

            Self::deposit_event(RawEvent::VoteCast(ticker, ballot_name, votes));

            Ok(())
        }

        /// Cancels a vote by setting it as expired
        ///
        /// # Arguments
        /// * `did` - DID of the token owner. Sender must be a signing key or master key of this DID
        /// * `ticker` - Ticker of the token for which ballot is to be cancelled
        /// * `ballot_name` - Name of the ballot
        pub fn cancel_ballot(origin, did: IdentityId, ticker: Ticker, ballot_name: Vec<u8>) -> DispatchResult {
            let sender = Signatory::AccountKey( AccountKey::try_from( ensure_signed(origin)?.encode())?);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signer_authorized(did, &sender), Error::<T>::InvalidSigner);
            ticker.canonize();
            ensure!(Self::is_owner(&ticker, did), Error::<T>::InvalidOwner);

            // This avoids cloning the variables to make the same tupple again and again
            let ticker_ballot_name = (ticker, ballot_name.clone());

            // Ensure the existance of valid ballot
            ensure!(<Ballots<T>>::exists(&ticker_ballot_name), Error::<T>::NotExists);
            let ballot = <Ballots<T>>::get(&ticker_ballot_name);
            let now = <pallet_timestamp::Module<T>>::get();
            ensure!(now < ballot.voting_end, Error::<T>::AlreadyEnded);

            // Clearing results
            <Results<T>>::mutate(&ticker_ballot_name, |results| {
                for i in 0..results.len() {
                    results[i] = 0.into();
                }
            });

            // NB Not deleting the ballot to prevent someone from
            // deleting a ballot mid vote and creating a new one with same name to confuse voters

            // This will prevent further voting. Essentially, canceling the ballot
            <Ballots<T>>::mutate(&ticker_ballot_name, |ballot_details| {
                ballot_details.voting_end = now;
            });

            Self::deposit_event(RawEvent::BallotCancelled(ticker, ballot_name));

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Balance = <T as CommonTrait>::Balance,
        Moment = <T as pallet_timestamp::Trait>::Moment,
    {
        /// A new ballot is created (Ticker, BallotName, BallotDetails)
        BallotCreated(Ticker, Vec<u8>, Ballot<Moment>),

        /// A vote is cast (Ticker, BallotName, Vote)
        VoteCast(Ticker, Vec<u8>, Vec<Balance>),

        /// An existing ballot is cancelled (Ticker, BallotName)
        BallotCancelled(Ticker, Vec<u8>),
    }
);

decl_error! {
    pub enum Error for Module<T: Trait> {
        /// sender must be a signing key for DID
        InvalidSigner,
        /// Sender must be the token owner
        InvalidOwner,
        /// A ballot with same name already exisits
        AlreadyExists,
        /// Voting end date in past / Voting end date before voting start date
        InvalidDate,
        /// No motion submitted
        NoMotions,
        /// No choice submitted
        NoChoicesInMotions,
        /// Could not decode choices
        InvalidChoicesType,
        /// Ballot does not exist
        NotExists,
        /// Voting hasn't started yet
        NotStarted,
        /// Voting ended already
        AlreadyEnded,
        /// No checkpoints created
        NoCheckpoints,
        /// Invalid vote
        InvalidVote,
        /// Not enough balance
        InsufficientBalance,
    }
}

impl<T: Trait> Module<T> {
    fn is_owner(ticker: &Ticker, did: IdentityId) -> bool {
        T::Asset::is_owner(ticker, did)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::prelude::*;
    use frame_support::traits::Currency;
    use frame_support::{
        assert_err, assert_ok, dispatch::DispatchResult, impl_outer_origin, parameter_types,
    };
    use frame_system::EnsureSignedBy;
    use sp_core::{crypto::key_types, H256};
    use sp_io::TestExternalities;
    use sp_runtime::{
        testing::{Header, UintAuthorityId},
        traits::{BlakeTwo256, ConvertInto, IdentityLookup, OpaqueKeys, Verify},
        AnySignature, KeyTypeId, Perbill,
    };
    use std::result::Result;
    use test_client::{self, AccountKeyring};

    use polymesh_runtime_balances as balances;
    use polymesh_runtime_common::traits::{
        asset::AcceptTransfer, group::GroupTrait, multisig::AddSignerMultiSig, CommonTrait,
    };
    use polymesh_runtime_group as group;
    use polymesh_runtime_identity as identity;

    use crate::{
        asset::{AssetType, SecurityToken, TickerRegistrationConfig},
        exemption, general_tm, percentage_tm, statistics,
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

    impl frame_system::Trait for Test {
        type Origin = Origin;
        type Index = u64;
        type BlockNumber = BlockNumber;
        type Hash = H256;
        type Hashing = BlakeTwo256;
        type AccountId = AccountId;
        type Lookup = IdentityLookup<AccountId>;
        type Header = Header;
        type Event = ();
        type Call = ();
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

    impl CommonTrait for Test {
        type Balance = u128;
        type CreationFee = CreationFee;
        type AcceptTransferTarget = Test;
        type BlockRewardsReserve = balances::Module<Test>;
    }

    impl AcceptTransfer for Test {
        fn accept_ticker_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }

        fn accept_token_ownership_transfer(_to_did: IdentityId, _auth_id: u64) -> DispatchResult {
            unimplemented!();
        }
    }

    impl balances::Trait for Test {
        type OnFreeBalanceZero = ();
        type OnNewAccount = ();
        type Event = ();
        type DustRemoval = ();
        type TransferPayment = ();
        type ExistentialDeposit = ExistentialDeposit;
        type TransferFee = TransferFee;
        type Identity = identity::Module<Test>;
    }

    parameter_types! {
        pub const MinimumPeriod: u64 = 3;
    }

    impl pallet_timestamp::Trait for Test {
        type Moment = u64;
        type OnTimestampSet = ();
        type MinimumPeriod = MinimumPeriod;
    }

    impl utils::Trait for Test {
        type Public = AccountId;
        type OffChainSignature = OffChainSignature;
        fn validator_id_to_account_id(
            v: <Self as pallet_session::Trait>::ValidatorId,
        ) -> Self::AccountId {
            v
        }
    }

    pub struct TestOnSessionEnding;
    impl pallet_session::OnSessionEnding<AuthorityId> for TestOnSessionEnding {
        fn on_session_ending(_: SessionIndex, _: SessionIndex) -> Option<Vec<AuthorityId>> {
            None
        }
    }

    pub struct TestSessionHandler;
    impl pallet_session::SessionHandler<AuthorityId> for TestSessionHandler {
        const KEY_TYPE_IDS: &'static [KeyTypeId] = &[key_types::DUMMY];

        fn on_new_session<Ks: OpaqueKeys>(
            _changed: bool,
            _validators: &[(AuthorityId, Ks)],
            _queued_validators: &[(AuthorityId, Ks)],
        ) {
        }

        fn on_disabled(_validator_index: usize) {}

        fn on_genesis_session<Ks: OpaqueKeys>(_validators: &[(AuthorityId, Ks)]) {}

        fn on_before_session_ending() {}
    }

    parameter_types! {
        pub const Period: BlockNumber = 1;
        pub const Offset: BlockNumber = 0;
        pub const DisabledValidatorsThreshold: Perbill = Perbill::from_percent(33);
    }

    impl pallet_session::Trait for Test {
        type OnSessionEnding = TestOnSessionEnding;
        type Keys = UintAuthorityId;
        type ShouldEndSession = pallet_session::PeriodicSessions<Period, Offset>;
        type SessionHandler = TestSessionHandler;
        type Event = ();
        type ValidatorId = AuthorityId;
        type ValidatorIdOf = ConvertInto;
        type SelectInitialValidators = ();
        type DisabledValidatorsThreshold = DisabledValidatorsThreshold;
    }

    impl pallet_session::historical::Trait for Test {
        type FullIdentification = ();
        type FullIdentificationOf = ();
    }

    parameter_types! {
        pub const One: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Two: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Three: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Four: AccountId = AccountId::from(AccountKeyring::Dave);
        pub const Five: AccountId = AccountId::from(AccountKeyring::Dave);
    }

    impl group::Trait<group::Instance1> for Test {
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
        type Proposal = Call<Test>;
        type AddSignerMultiSigTarget = Test;
        type KYCServiceProviders = Test;
        type Balances = balances::Module<Test>;
    }

    impl GroupTrait for Test {
        fn get_members() -> Vec<IdentityId> {
            unimplemented!();
        }

        fn is_member(_member_id: &IdentityId) -> bool {
            unimplemented!();
        }
    }

    impl AddSignerMultiSig for Test {
        fn accept_multisig_signer(_: Signatory, _: u64) -> DispatchResult {
            unimplemented!()
        }
    }

    impl asset::Trait for Test {
        type Event = ();
        type Currency = balances::Module<Test>;
    }

    impl statistics::Trait for Test {}

    impl percentage_tm::Trait for Test {
        type Event = ();
    }

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
    fn build_ext() -> TestExternalities {
        let mut t = frame_system::GenesisConfig::default()
            .build_storage::<Test>()
            .unwrap();
        asset::GenesisConfig::<Test> {
            asset_creation_fee: 0,
            ticker_registration_fee: 0,
            ticker_registration_config: TickerRegistrationConfig {
                max_ticker_length: 12,
                registration_length: Some(10000),
            },
            fee_collector: AccountKeyring::Dave.public().into(),
        }
        .assimilate_storage(&mut t)
        .unwrap();
        sp_io::TestExternalities::new(t)
    }

    fn make_account(
        account_id: &AccountId,
    ) -> Result<(<Test as frame_system::Trait>::Origin, IdentityId), &'static str> {
        let signed_id = Origin::signed(account_id.clone());
        Balances::make_free_balance_be(&account_id, 1_000_000);
        let _ = Identity::register_did(signed_id.clone(), vec![]);
        let did = Identity::get_identity(&AccountKey::try_from(account_id.encode())?).unwrap();
        Ok((signed_id, did))
    }

    #[test]
    fn add_ballot() {
        build_ext().execute_with(|| {
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
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                AssetType::default(),
                vec![],
                None
            ));

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
            ));

            let now = Utc::now().timestamp() as u64;
            <pallet_timestamp::Module<Test>>::set_timestamp(now);

            let motion1 = Motion {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let motion2 = Motion {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                motions: vec![motion1.clone(), motion2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    ticker,
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                Error::<Test>::InvalidSigner
            );

            assert_err!(
                Voting::add_ballot(
                    tokenholder_acc.clone(),
                    tokenholder_did,
                    ticker,
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                Error::<Test>::InvalidOwner
            );

            let expired_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: 0,
                motions: vec![motion1.clone(), motion2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    expired_ballot_details.clone()
                ),
                Error::<Test>::InvalidDate
            );

            let invalid_date_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now + now + now,
                voting_end: now + now,
                motions: vec![motion1.clone(), motion2.clone()],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    invalid_date_ballot_details.clone()
                ),
                Error::<Test>::InvalidDate
            );

            let empty_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                motions: vec![],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    empty_ballot_details.clone()
                ),
                Error::<Test>::NoMotions
            );

            let empty_motion = Motion {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![],
            };

            let no_choice_ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                motions: vec![motion1.clone(), motion2.clone(), empty_motion],
            };

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    no_choice_ballot_details.clone()
                ),
                Error::<Test>::NoChoicesInMotions
            );

            // Adding ballot
            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone(),
                ballot_details.clone()
            ));

            assert_err!(
                Voting::add_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    ballot_details.clone()
                ),
                Error::<Test>::AlreadyExists
            );
        });
    }

    #[test]
    fn cancel_ballot() {
        build_ext().execute_with(|| {
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
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                AssetType::default(),
                vec![],
                None
            ));

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
            ));

            let now = Utc::now().timestamp() as u64;
            <pallet_timestamp::Module<Test>>::set_timestamp(now);

            let motion1 = Motion {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let motion2 = Motion {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 1,
                voting_start: now,
                voting_end: now + now,
                motions: vec![motion1.clone(), motion2.clone()],
            };

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone()
                ),
                Error::<Test>::NotExists
            );

            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone(),
                ballot_details.clone()
            ));

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    ticker,
                    ballot_name.clone()
                ),
                Error::<Test>::InvalidSigner
            );

            assert_err!(
                Voting::cancel_ballot(
                    tokenholder_acc.clone(),
                    tokenholder_did,
                    ticker,
                    ballot_name.clone()
                ),
                Error::<Test>::InvalidOwner
            );

            <pallet_timestamp::Module<Test>>::set_timestamp(now + now + now);

            assert_err!(
                Voting::cancel_ballot(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone()
                ),
                Error::<Test>::AlreadyEnded
            );

            <pallet_timestamp::Module<Test>>::set_timestamp(now);

            // Cancelling ballot
            assert_ok!(Voting::cancel_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone()
            ));
        });
    }

    #[test]
    fn vote() {
        build_ext().execute_with(|| {
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
                asset_type: AssetType::default(),
            };
            let ticker = Ticker::from_slice(token.name.as_slice());
            // Share issuance is successful
            assert_ok!(Asset::create_token(
                token_owner_acc.clone(),
                token_owner_did,
                token.name.clone(),
                ticker,
                token.total_supply,
                true,
                AssetType::default(),
                vec![],
                None
            ));

            let asset_rule = general_tm::AssetRule {
                sender_rules: vec![],
                receiver_rules: vec![],
            };

            // Allow all transfers
            assert_ok!(GeneralTM::add_active_rule(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                asset_rule
            ));

            assert_ok!(Asset::transfer(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                tokenholder_did,
                500
            ));

            let now = Utc::now().timestamp() as u64;
            <pallet_timestamp::Module<Test>>::set_timestamp(now);

            let motion1 = Motion {
                title: vec![0x01],
                info_link: vec![0x01],
                choices: vec![vec![0x01], vec![0x02]],
            };
            let motion2 = Motion {
                title: vec![0x02],
                info_link: vec![0x02],
                choices: vec![vec![0x01], vec![0x02], vec![0x03]],
            };

            let ballot_name = vec![0x01];

            let ballot_details = Ballot {
                checkpoint_id: 2,
                voting_start: now,
                voting_end: now + now,
                motions: vec![motion1.clone(), motion2.clone()],
            };

            assert_ok!(Voting::add_ballot(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone(),
                ballot_details.clone()
            ));

            let votes = vec![100, 100, 100, 100, 100];

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    tokenholder_did,
                    ticker,
                    ballot_name.clone(),
                    votes.clone()
                ),
                Error::<Test>::InvalidSigner
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    vec![0x02],
                    votes.clone()
                ),
                Error::<Test>::NotExists
            );

            <pallet_timestamp::Module<Test>>::set_timestamp(now - 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    votes.clone()
                ),
                Error::<Test>::NotStarted
            );

            <pallet_timestamp::Module<Test>>::set_timestamp(now + now + 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    votes.clone()
                ),
                Error::<Test>::AlreadyEnded
            );

            <pallet_timestamp::Module<Test>>::set_timestamp(now + 1);

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    votes.clone()
                ),
                Error::<Test>::NoCheckpoints
            );

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
            ));

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    votes.clone()
                ),
                Error::<Test>::NoCheckpoints
            );

            assert_ok!(Asset::create_checkpoint(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
            ));

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    vec![100, 100, 100, 100]
                ),
                Error::<Test>::InvalidVote
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    vec![100, 100, 100, 100, 100, 100]
                ),
                Error::<Test>::InvalidVote
            );

            assert_err!(
                Voting::vote(
                    token_owner_acc.clone(),
                    token_owner_did,
                    ticker,
                    ballot_name.clone(),
                    vec![100, 100, 100, 100, 200]
                ),
                Error::<Test>::InsufficientBalance
            );

            // Initial vote
            assert_ok!(Voting::vote(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone(),
                votes.clone()
            ));

            let mut result = Voting::results((ticker, ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [100, 100, 100, 100, 100], "Invalid result");

            // Changed vote
            assert_ok!(Voting::vote(
                token_owner_acc.clone(),
                token_owner_did,
                ticker,
                ballot_name.clone(),
                vec![500, 0, 0, 0, 0]
            ));

            result = Voting::results((ticker, ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [500, 0, 0, 0, 0], "Invalid result");

            // Second vote
            assert_ok!(Voting::vote(
                tokenholder_acc.clone(),
                tokenholder_did,
                ticker,
                ballot_name.clone(),
                vec![0, 500, 0, 0, 0]
            ));

            result = Voting::results((ticker, ballot_name.clone()));
            assert_eq!(result.len(), 5, "Invalid result len");
            assert_eq!(result, [500, 500, 0, 0, 0], "Invalid result");
        })
    }
}
