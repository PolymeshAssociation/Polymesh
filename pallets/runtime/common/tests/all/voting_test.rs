use super::{
    storage::{make_account, TestStorage},
    ExtBuilder,
};
use pallet_asset::{self as asset, AssetType, SecurityToken};
use pallet_compliance_manager as compliance_manager;
use polymesh_primitives::Ticker;
use polymesh_runtime_common::voting::{self, Ballot, Motion};

use chrono::prelude::Utc;
use frame_support::{assert_err, assert_ok};
use std::convert::TryFrom;
use test_client::AccountKeyring;

type Asset = asset::Module<TestStorage>;
type ComplianceManager = compliance_manager::Module<TestStorage>;
type Voting = voting::Module<TestStorage>;
type Error = voting::Error<TestStorage>;

#[test]
fn add_ballot() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_acc, token_owner_did) =
            make_account(AccountKeyring::Alice.public()).unwrap();
        let (tokenholder_acc, _) = make_account(AccountKeyring::Bob.public()).unwrap();

        // A token representing 1M shares
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        // Share issuance is successful
        assert_ok!(Asset::create_asset(
            token_owner_acc.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            AssetType::default(),
            vec![],
            None,
            None,
        ));

        assert_ok!(Asset::create_checkpoint(token_owner_acc.clone(), ticker,));

        let now = Utc::now().timestamp() as u64;
        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now);

        let motion1 = Motion {
            title: vec![0x01].into(),
            info_link: vec![0x01].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into()],
        };
        let motion2 = Motion {
            title: vec![0x02].into(),
            info_link: vec![0x02].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into(), vec![0x03].into()],
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
                tokenholder_acc.clone(),
                ticker,
                ballot_name.clone(),
                ballot_details.clone()
            ),
            Error::InvalidOwner
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
                ticker,
                ballot_name.clone(),
                expired_ballot_details.clone()
            ),
            Error::InvalidDate
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
                ticker,
                ballot_name.clone(),
                invalid_date_ballot_details.clone()
            ),
            Error::InvalidDate
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
                ticker,
                ballot_name.clone(),
                empty_ballot_details.clone()
            ),
            Error::NoMotions
        );

        let empty_motion = Motion {
            title: vec![0x02].into(),
            info_link: vec![0x02].into(),
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
                ticker,
                ballot_name.clone(),
                no_choice_ballot_details.clone()
            ),
            Error::NoChoicesInMotions
        );

        // Adding ballot
        assert_ok!(Voting::add_ballot(
            token_owner_acc.clone(),
            ticker,
            ballot_name.clone(),
            ballot_details.clone()
        ));

        assert_err!(
            Voting::add_ballot(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                ballot_details.clone()
            ),
            Error::AlreadyExists
        );
    });
}

#[test]
fn cancel_ballot() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_acc, token_owner_did) =
            make_account(AccountKeyring::Alice.public()).unwrap();
        let (tokenholder_acc, _) = make_account(AccountKeyring::Bob.public()).unwrap();

        // A token representing 1M shares
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_did,
            total_supply: 1_000_000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        // Share issuance is successful
        assert_ok!(Asset::create_asset(
            token_owner_acc.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            AssetType::default(),
            vec![],
            None,
            None,
        ));

        assert_ok!(Asset::create_checkpoint(token_owner_acc.clone(), ticker,));

        let now = Utc::now().timestamp() as u64;
        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now);

        let motion1 = Motion {
            title: vec![0x01].into(),
            info_link: vec![0x01].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into()],
        };
        let motion2 = Motion {
            title: vec![0x02].into(),
            info_link: vec![0x02].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into(), vec![0x03].into()],
        };

        let ballot_name = vec![0x01];

        let ballot_details = Ballot {
            checkpoint_id: 1,
            voting_start: now,
            voting_end: now + now,
            motions: vec![motion1.clone(), motion2.clone()],
        };

        assert_err!(
            Voting::cancel_ballot(token_owner_acc.clone(), ticker, ballot_name.clone()),
            Error::NotExists
        );

        assert_ok!(Voting::add_ballot(
            token_owner_acc.clone(),
            ticker,
            ballot_name.clone(),
            ballot_details.clone()
        ));

        assert_err!(
            Voting::cancel_ballot(tokenholder_acc.clone(), ticker, ballot_name.clone()),
            Error::InvalidOwner
        );

        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now + now + now);

        assert_err!(
            Voting::cancel_ballot(token_owner_acc.clone(), ticker, ballot_name.clone()),
            Error::AlreadyEnded
        );

        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now);

        // Cancelling ballot
        assert_ok!(Voting::cancel_ballot(
            token_owner_acc.clone(),
            ticker,
            ballot_name.clone()
        ));
    });
}

#[test]
fn vote() {
    ExtBuilder::default().build().execute_with(|| {
        let (token_owner_acc, token_owner_did) =
            make_account(AccountKeyring::Alice.public()).unwrap();
        let (tokenholder_acc, tokenholder_did) =
            make_account(AccountKeyring::Bob.public()).unwrap();

        // A token representing 1M shares
        let token = SecurityToken {
            name: vec![0x01].into(),
            owner_did: token_owner_did,
            total_supply: 1000,
            divisible: true,
            asset_type: AssetType::default(),
            ..Default::default()
        };
        let ticker = Ticker::try_from(token.name.as_slice()).unwrap();
        // Share issuance is successful
        assert_ok!(Asset::create_asset(
            token_owner_acc.clone(),
            token.name.clone(),
            ticker,
            token.total_supply,
            true,
            AssetType::default(),
            vec![],
            None,
            None,
        ));

        let sender_rules = vec![];
        let receiver_rules = vec![];

        // Allow all transfers
        assert_ok!(ComplianceManager::add_active_rule(
            token_owner_acc.clone(),
            ticker,
            sender_rules,
            receiver_rules
        ));

        assert_ok!(Asset::unsafe_transfer(
            token_owner_did,
            &ticker,
            token_owner_did,
            tokenholder_did,
            500
        ));

        let now = Utc::now().timestamp() as u64;
        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now);

        let motion1 = Motion {
            title: vec![0x01].into(),
            info_link: vec![0x01].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into()],
        };
        let motion2 = Motion {
            title: vec![0x02].into(),
            info_link: vec![0x02].into(),
            choices: vec![vec![0x01].into(), vec![0x02].into(), vec![0x03].into()],
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
            ticker,
            ballot_name.clone(),
            ballot_details.clone()
        ));

        let votes = vec![100, 100, 100, 100, 100];

        assert_err!(
            Voting::vote(token_owner_acc.clone(), ticker, vec![0x02], votes.clone()),
            Error::NotExists
        );

        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now - 1);

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                votes.clone()
            ),
            Error::NotStarted
        );

        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now + now + 1);

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                votes.clone()
            ),
            Error::AlreadyEnded
        );

        <pallet_timestamp::Module<TestStorage>>::set_timestamp(now + 1);

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                votes.clone()
            ),
            Error::NoCheckpoints
        );

        assert_ok!(Asset::create_checkpoint(token_owner_acc.clone(), ticker,));

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                votes.clone()
            ),
            Error::NoCheckpoints
        );

        assert_ok!(Asset::create_checkpoint(token_owner_acc.clone(), ticker,));

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                vec![100, 100, 100, 100]
            ),
            Error::InvalidVote
        );

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                vec![100, 100, 100, 100, 100, 100]
            ),
            Error::InvalidVote
        );

        assert_err!(
            Voting::vote(
                token_owner_acc.clone(),
                ticker,
                ballot_name.clone(),
                vec![100, 100, 100, 100, 200]
            ),
            Error::InsufficientBalance
        );

        // Initial vote
        assert_ok!(Voting::vote(
            token_owner_acc.clone(),
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
            ticker,
            ballot_name.clone(),
            vec![0, 500, 0, 0, 0]
        ));

        result = Voting::results((ticker, ballot_name.clone()));
        assert_eq!(result.len(), 5, "Invalid result len");
        assert_eq!(result, [500, 500, 0, 0, 0], "Invalid result");
    })
}
