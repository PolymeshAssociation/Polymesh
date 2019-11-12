use crate::{
    asset::AssetTrait,
    identity,
    utils,
};
use primitives::{IdentityId, Key};
use codec::Encode;
use rstd::{convert::TryFrom, prelude::*};
use sr_primitives::traits::{CheckedAdd, CheckedDiv, CheckedMul, CheckedSub};
use srml_support::traits::Currency;
use srml_support::{decl_event, decl_module, decl_storage, dispatch::Result, ensure};
use system::{self, ensure_signed};

/// The module's configuration trait.
pub trait Trait: timestamp::Trait + system::Trait + utils::Trait + identity::trait {
    type Event: From<Event<Self>> + Into<<Self as system::Trait>::Event>;
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Ballot<V> {
    checkpoint_id: u64,
    voting_start: V,
    voting_end: V,
    proposals: Vec<Proposal>,
}

#[derive(codec::Encode, codec::Decode, Default, Clone, PartialEq, Eq, Debug)]
pub struct Proposal {
    title: Vec<u8>,
    info_link: Vec<u8>,
    choices: Vec<Vec<u8>>, //Choices excluding abstain. Voting power not used is considered abstained.
}

decl_storage! {
    trait Store for Module<T: Trait> as Voting {
        // Mapping of ticker and ballot name -> ballot details
        pub Ballots get(ballots): map (Vec<u8>, Vec<u8>) -> Ballot<T::Moment>;
        // Mapping from ticker to vector of Ballot names. Helper data for the UI.
        pub BallotNames get(ballot_names): map Vec<u8> -> Vec<Vec<u8>>;
        // Helper data to make voting cheaper.
        // (ticker, BallotName) -> NoOfChoices
        pub TotalChoices get(total_choices): map (Vec<u8>, Vec<u8>) -> u64;
        // (Ticker, BallotName, DID) -> Vector of vote weights.
        // weight at 0 index means weight for choice 1 of proposal 1.
        // weight at 1 index means weight for choice 2 of proposal 1.
        // User must enter 0 vote weight if they don't want to vote for a choice.
        pub Votes get(votes): map (Vec<u8>, Vec<u8>, IdentityId) -> Vec<T:TokenBalance>;
        // (Ticker, BallotName) -> Vector of current vote weights.
        // weight at 0 index means weight for choice 1 of proposal 1.
        // weight at 1 index means weight for choice 2 of proposal 1.
        pub Results get(results): map (Vec<u8>, Vec<u8>) -> Vec<T:TokenBalance>;
    }
}

decl_module! {
    /// The module declaration.
    pub struct Module<T: Trait> for enum Call where origin: T::Origin {
        // Initializing events
        // this is needed only if you are using events in your module
        fn deposit_event() = default;

        /// Adds a ballot
        ///
        /// # Arguments
        /// * `did` - DID of the token owner. Sender must be a signing key or master key of this DID
        pub fn add_ballot(origin, did: IdentityId, ticker: Vec<u8>, ballot_name: Vec<u8>, ballot_details: Ballot) -> Result {
            let sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");
            ensure!(Self::is_owner(&upper_ticker, &did),"Sender must be the token owner");

            // Ensure the uniqueness of the ballot
            ensure!(!<Ballots>::exists(&upper_ticker, &ballot_name), "A ballot with same name already exisits");

            let now = <timestamp::Module<T>>::get();

            ensure!(now < ballot_details.voting_end, "Voting end date in past");
            ensure!(ballot_details.voting_end < ballot_details.voting_start, "Voting end date before voting start date");
            ensure!(ballot_details.proposals.len() > 0, "No proposal submitted");

            // NB: Checkpoint ID is not verified here to allow creating ballots that will become active in future.
            // Voting will only be allowed on checkpoints that exist.

            let mut total_choices:u64 = 0u64;

            for proposal in ballot_details.proposals {
                ensure!(proposal.choices.len() > 0, "No choice submitted");
                total_choices += proposal.choices.len();
            }

            <Ballots>::insert((&upper_ticker, &ballot_name), ballot_details.clone());
            <BallotNames>::insert(&upper_ticker, ballot_name.clone());
            <TotalChoices>::insert((&upper_ticker, &ballot_name), total_choices.clone());

            let initial_results = vec![<T as utils::Trait>::as_tb(0); total_choices];
            <Results>::insert((&upper_ticker, &ballot_name), initial_results);

            Self::deposit_event(RawEvent::BallotCreated(upper_ticker, ballot_name, ballot_details));

            Ok(())
        }

        pub fn vote(origin, did: IdentityId, ticker: Vec<u8>, ballot_name: Vec<u8>, votes: Vec<T:TokenBalance>) -> Result {
            let sender = ensure_signed(origin)?;
            let upper_ticker = utils::bytes_to_upper(&ticker);

            // Check that sender is allowed to act on behalf of `did`
            ensure!(<identity::Module<T>>::is_signing_key(did, &Key::try_from(sender.encode())?), "sender must be a signing key for DID");

            // Ensure validity the ballot
            ensure!(<Ballots>::exists((&upper_ticker, &ballot_name)), "Ballot does not exist");
            let ballot = <Ballots>::get((&upper_ticker, &ballot_name));
            let now = <timestamp::Module<T>>::get();
            ensure!(ballot.voting_start >= now, "Voting hasn't started yet");
            ensure!(ballot.voting_end <= now, "Voting ended already");

            let count = <asset::TotalCheckpoints>::get(&upper_ticker);
            ensure!(ballot.checkpoint_id <= count, "Checkpoint has not be created yet");

            // Ensure vote is valid
            ensure!(votes.len() == <TotalChoices>::get((&upper_ticker, &ballot_name), "Invalid vote");

            let total_votes = <T as utils::Trait>::as_tb(0);
            for vote in votes {
                total_votes += vote;
            }

            ensure!(total_votes <= <asset::Module<T>>::get_balance_at(&ticker, did, ballot.checkpoint_id), "Not enough balance");

            if <Votes>::exists(&upper_ticker, &ballot_name, &did) {
                //User wants to change their vote. We first need to subtract their existing vote.
                let previous_votes = <Votes>::get((&upper_ticker, &ballot_name, &did));
                <Results>::mutate((&upper_ticker, &ballot_name), |results| {
                    for i in 0..results.len() {
                        results[i] -= previous_votes[i];
                    }
                });
            }

            <Results>::mutate((&upper_ticker, &ballot_name), |results| {
                for i in 0..results.len() {
                    results[i] += votes[i];
                }
            });

            <Votes>::insert((&upper_ticker, &ballot_name, &did), votes);

            Ok(())
        }
    }
}

decl_event!(
    pub enum Event<T>
    where
        Ballot = T::Ballot,
    {
        // (Ticker, BallotName, BallotDetails)
        BallotCreated(Vec<u8>, Vec<u8>, Ballot),
    }
);

impl<T: Trait> Module<T> {
    pub fn is_owner(ticker: &Vec<u8>, did: &Vec<u8>) -> bool {
        let upper_ticker = utils::bytes_to_upper(ticker.as_slice());
        T::Asset::is_owner(&upper_ticker, did)
    }
}

/// tests for this module
#[cfg(test)]
mod tests {

}
